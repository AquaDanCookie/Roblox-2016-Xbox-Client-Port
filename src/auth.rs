use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use secrecy::SecretString;

use crate::config::Paths;
use crate::models::{AuthenticatedUser, Friend, Load, Presence};

const KEYRING_SERVICE: &str = "roblox-2016-xbox";
const KEYRING_USER: &str = "session";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthState {
    SignedOut,
    Checking,
    SigningIn,
    SignedIn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionStorage {
    Keyring,
    File,
    None,
}

impl SessionStorage {
    pub fn label(self) -> &'static str {
        match self {
            SessionStorage::Keyring => "Secret Service",
            SessionStorage::File => "Protected file",
            SessionStorage::None => "This session only",
        }
    }
}

pub struct Session {
    pub token: SecretString,
    pub user: AuthenticatedUser,
    pub storage: SessionStorage,
    pub headshots: HashMap<u64, String>,
    pub avatar: Option<String>,
    pub friends: Load<Vec<Friend>>,
    pub presence: HashMap<u64, Presence>,
    pub robux: Load<u64>,
}

pub struct LoadedSession {
    pub token: String,
}

#[derive(Clone)]
pub struct SessionStore {
    paths: Paths,
}

impl SessionStore {
    pub fn new(paths: Paths) -> Self {
        Self { paths }
    }

    pub fn load(&self) -> Result<Option<LoadedSession>, String> {
        if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER) {
            match entry.get_password() {
                Ok(token) if !token.trim().is_empty() => {
                    return Ok(Some(LoadedSession {
                        token: token.trim().to_owned(),
                    }));
                }
                Ok(_) => {}
                Err(error) => log::debug!("Secret Service session lookup: {error}"),
            }
        }

        let path = self.paths.data_dir.join("session");
        match std::fs::read_to_string(&path) {
            Ok(token) if !token.trim().is_empty() => {
                log::warn!("Using the fallback session file at {}", path.display());
                Ok(Some(LoadedSession {
                    token: token.trim().to_owned(),
                }))
            }
            Ok(_) => Ok(None),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(format!("could not read the session file: {error}")),
        }
    }

    pub fn save(&self, token: &str) -> SessionStorage {
        if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
            && entry.set_password(token).is_ok()
        {
            return SessionStorage::Keyring;
        }

        log::warn!("Secret Service is unavailable; using a protected session file.");
        match write_private(&self.paths.data_dir.join("session"), token) {
            Ok(()) => SessionStorage::File,
            Err(error) => {
                log::warn!("Could not write the fallback session file: {error}");
                SessionStorage::None
            }
        }
    }

    pub fn clear(&self) {
        if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
            && let Err(error) = entry.delete_credential()
        {
            log::debug!("Secret Service session delete: {error}");
        }
        let _ = std::fs::remove_file(self.paths.data_dir.join("session"));
    }
}

pub fn read_sober_token(paths: &Paths) -> Result<String, String> {
    let path = paths.sober_cookie_file();
    let metadata = std::fs::metadata(&path)
        .map_err(|_| "No Sober session found. Sign in to Sober first.".to_owned())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            log::warn!(
                "Sober cookie file is readable by other users: {}",
                path.display()
            );
        }
    }
    let raw = std::fs::read_to_string(&path)
        .map_err(|error| format!("Could not read the Sober session: {error}"))?;
    for part in raw.split(';') {
        let part = part.trim();
        if let Some(token) = part.strip_prefix(".ROBLOSECURITY=")
            && !token.is_empty()
        {
            return Ok(token.to_owned());
        }
    }
    Err("Sober is installed but not signed in.".to_owned())
}

pub struct LoginProcess {
    pub child: Child,
    pub out: PathBuf,
}

impl Drop for LoginProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self.out);
    }
}

pub fn spawn_login_helper(paths: &Paths) -> Result<LoginProcess, String> {
    let out = paths
        .data_dir
        .join(format!("login-{}.token", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let exe = std::env::current_exe()
        .map_err(|error| format!("Could not find the app binary: {error}"))?;
    let child = Command::new(exe)
        .arg("--login-helper")
        .arg("--out")
        .arg(&out)
        .env("WEBKIT_DISABLE_DMABUF_RENDERER", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("Could not open the Roblox sign-in window: {error}"))?;
    Ok(LoginProcess { child, out })
}

fn write_private(path: &Path, value: &str) -> std::io::Result<()> {
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(value.as_bytes())?;
    file.sync_all()?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(())
}
