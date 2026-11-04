use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaunchRuntime {
    Sober,
    Cordial,
    None,
}

pub fn uri_for_place(place_id: u64) -> String {
    format!("roblox://experiences/start?placeId={place_id}")
}

pub fn spawn_launch(place_id: u64, runtime: LaunchRuntime) -> Result<Child, String> {
    let uri = uri_for_place(place_id);
    let mut command = match runtime {
        LaunchRuntime::Sober => {
            let mut command = Command::new("flatpak");
            command.args(["run", "org.vinegarhq.Sober"]);
            command
        }
        LaunchRuntime::Cordial => Command::new("xdg-open"),
        LaunchRuntime::None => {
            return Err("No Roblox runtime is installed. Install Sober or Cordial.".to_owned());
        }
    };
    command
        .arg(uri)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("Could not start the Roblox runtime: {error}"))
}

pub fn open_sober() -> Result<Child, String> {
    Command::new("flatpak")
        .args(["run", "org.vinegarhq.Sober"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("Could not open Sober: {error}"))
}

pub struct LaunchLog {
    latest: PathBuf,
    path: PathBuf,
    offset: usize,
    saw_join: bool,
    saw_loaded_marker: bool,
    saw_loaded_place: bool,
    saw_crash: bool,
}

impl LaunchLog {
    pub fn snapshot(dir: &Path) -> Self {
        let latest = dir.join("latest.log");
        let path = std::fs::canonicalize(&latest).unwrap_or_else(|_| latest.clone());
        let offset = std::fs::metadata(&path)
            .map(|metadata| metadata.len() as usize)
            .unwrap_or(0);
        Self {
            latest,
            path,
            offset,
            saw_join: false,
            saw_loaded_marker: false,
            saw_loaded_place: false,
            saw_crash: false,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn poll(&mut self, place_id: u64) -> bool {
        let resolved = std::fs::canonicalize(&self.latest).unwrap_or_else(|_| self.latest.clone());
        if resolved != self.path {
            self.path = resolved;
            self.offset = 0;
            self.saw_join = false;
            self.saw_loaded_marker = false;
            self.saw_loaded_place = false;
            self.saw_crash = false;
        }

        let bytes = std::fs::read(&self.path).unwrap_or_default();
        let end = bytes.len();
        let start = self.offset.min(end);
        let chunk = String::from_utf8_lossy(&bytes[start..end]);
        self.offset = end;

        let lower = chunk.to_ascii_lowercase();
        self.saw_crash |= lower.contains("roblox has crashed")
            || lower.contains("out of memory")
            || lower.contains("invalid device")
            || lower.contains("rbxcrash");
        self.saw_join |= chunk.contains(&format!("placeid:{place_id}"));
        self.saw_loaded_marker |= chunk.contains("onGameLoaded");
        self.saw_loaded_place |= chunk.contains(&format!("placeId:{place_id}"));

        self.saw_join && self.saw_loaded_marker && self.saw_loaded_place
    }

    pub fn crashed(&self) -> bool {
        self.saw_crash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_dir() -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "roblox-2016-launch-test-{}-{id}",
            std::process::id()
        ))
    }

    fn append(path: &Path, text: &str) {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        file.write_all(text.as_bytes()).unwrap();
    }

    #[test]
    fn uri_uses_the_verified_form() {
        assert_eq!(
            uri_for_place(189707),
            "roblox://experiences/start?placeId=189707"
        );
    }

    #[test]
    fn join_needs_the_place_marker_and_loaded_marker() {
        let dir = test_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("latest.log");
        std::fs::write(&path, "").unwrap();
        let mut log = LaunchLog::snapshot(&dir);

        append(&path, "FLog::GameJoinLoadTime placeid:189707\n");
        assert!(!log.poll(189707));
        assert!(!log.crashed());

        append(&path, "onGameLoaded() placeId:189707\n");
        assert!(log.poll(189707));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_different_place_does_not_join() {
        let dir = test_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("latest.log");
        std::fs::write(&path, "").unwrap();
        let mut log = LaunchLog::snapshot(&dir);

        append(
            &path,
            "placeid:999\nonGameLoaded() placeId:999\nplaceid:999\n",
        );
        assert!(!log.poll(189707));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn crash_markers_are_detected() {
        for marker in [
            "Roblox has crashed",
            "out of memory",
            "Invalid device",
            "RBXCRASH: OutOfMemoryGraphics",
        ] {
            let dir = test_dir();
            std::fs::create_dir_all(&dir).unwrap();
            let path = dir.join("latest.log");
            std::fs::write(&path, "").unwrap();
            let mut log = LaunchLog::snapshot(&dir);
            append(&path, marker);
            assert!(!log.poll(189707));
            assert!(log.crashed(), "{marker} should be a crash");
            std::fs::remove_dir_all(&dir).ok();
        }
    }
}
