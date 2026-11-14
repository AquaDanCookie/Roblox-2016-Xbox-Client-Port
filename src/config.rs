use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_volume")]
    pub master_volume: f32,
    #[serde(default = "default_volume")]
    pub music_volume: f32,
    #[serde(default)]
    pub muted: bool,
    #[serde(default)]
    pub fullscreen: bool,
    #[serde(default = "default_true")]
    pub mute_on_focus_loss: bool,
    #[serde(default = "default_true")]
    pub ui_sounds: bool,
    #[serde(default)]
    pub reduced_motion: bool,
    #[serde(default)]
    pub settings_tab: usize,
    #[serde(default = "default_game_scroll_speed")]
    pub game_scroll_speed: f32,
    #[serde(default)]
    pub window_maximized: bool,
}

fn default_volume() -> f32 {
    0.8
}

fn default_true() -> bool {
    true
}

fn default_game_scroll_speed() -> f32 {
    1.0
}

impl Default for Config {
    fn default() -> Self {
        Self {
            master_volume: 0.8,
            music_volume: 0.8,
            muted: false,
            fullscreen: false,
            mute_on_focus_loss: true,
            ui_sounds: true,
            reduced_motion: false,
            settings_tab: 0,
            game_scroll_speed: 1.0,
            window_maximized: false,
        }
    }
}

impl Config {
    pub fn load(paths: &Paths) -> Self {
        let path = paths.config_file();
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        match toml::from_str::<Self>(&text) {
            Ok(config) => config,
            Err(error) => {
                log::warn!("Ignoring invalid config {}: {error}", path.display());
                Self::default()
            }
        }
    }

    pub fn save(&self, paths: &Paths) {
        let path = paths.config_file();
        if let Some(parent) = path.parent()
            && let Err(error) = std::fs::create_dir_all(parent)
        {
            log::warn!(
                "Could not create config directory {}: {error}",
                parent.display()
            );
            return;
        }
        match toml::to_string_pretty(self) {
            Ok(text) => {
                if let Err(error) = std::fs::write(&path, text) {
                    log::warn!("Could not write config {}: {error}", path.display());
                }
            }
            Err(error) => log::warn!("Could not serialize config: {error}"),
        }
    }
}

#[derive(Clone)]
pub struct Paths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub state_dir: PathBuf,
    pub project_dir: PathBuf,
}

/// Locates the folder that contains the runtime `assets` directory.
///
/// Release builds ship `assets/` next to the binary. Development runs from
/// `target/debug` or `target/release`, so the repository root is two levels up.
/// The compile-time manifest path is only a last resort.
fn runtime_project_dir() -> PathBuf {
    if let Ok(executable) = std::env::current_exe() {
        if let Some(exe_dir) = executable.parent() {
            for candidate in [exe_dir.to_path_buf(), exe_dir.join("../..")] {
                if candidate.join("assets").is_dir() {
                    return candidate.canonicalize().unwrap_or(candidate);
                }
            }
        }
    }

    if let Ok(current_dir) = std::env::current_dir() {
        if current_dir.join("assets").is_dir() {
            return current_dir;
        }
    }

    std::env::current_exe()
        .ok()
        .and_then(|executable| executable.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}

impl Paths {
    pub fn new() -> Self {
        let project_dir = runtime_project_dir();
        let project = directories::ProjectDirs::from("io", "local", "roblox-2016-xbox");
        if let Some(project) = project {
            return Self {
                config_dir: project.config_dir().to_path_buf(),
                data_dir: project.data_dir().to_path_buf(),
                cache_dir: project.cache_dir().to_path_buf(),
                state_dir: project
                    .state_dir()
                    .map(std::path::Path::to_path_buf)
                    .unwrap_or_else(|| project.data_dir().to_path_buf()),
                project_dir,
            };
        }

        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            config_dir: home.join(".config/roblox-2016-xbox"),
            data_dir: home.join(".local/share/roblox-2016-xbox"),
            cache_dir: home.join(".cache/roblox-2016-xbox"),
            state_dir: home.join(".local/state/roblox-2016-xbox"),
            project_dir,
        }
    }

    pub fn config_file(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }

    pub fn project_assets_dir(&self) -> PathBuf {
        self.project_dir.join("assets")
    }

    pub fn music_dir(&self) -> PathBuf {
        self.project_assets_dir().join("music")
    }

    fn fallback_music_dir(&self) -> PathBuf {
        self.data_dir.join("music")
    }

    pub fn music_file(&self) -> Option<PathBuf> {
        for dir in [self.music_dir(), self.fallback_music_dir()] {
            if let Some(path) = ["menu.mp3", "menu.wav", "menu.flac"]
                .iter()
                .map(|name| dir.join(name))
                .find(|path| path.is_file())
            {
                return Some(path);
            }
        }
        None
    }

    pub fn sounds_dir(&self) -> PathBuf {
        self.project_assets_dir().join("sounds")
    }

    fn fallback_sounds_dir(&self) -> PathBuf {
        self.data_dir.join("sounds")
    }

    pub fn sfx_file(&self, name: &str) -> Option<PathBuf> {
        for dir in [self.sounds_dir(), self.fallback_sounds_dir()] {
            if let Some(path) = ["mp3", "wav", "flac"]
                .iter()
                .map(|extension| dir.join(format!("{name}.{extension}")))
                .find(|path| path.is_file())
            {
                return Some(path);
            }
        }
        None
    }

    pub fn background_dir(&self) -> PathBuf {
        self.project_assets_dir().join("background")
    }

    fn fallback_background_dir(&self) -> PathBuf {
        self.data_dir.join("background")
    }

    pub fn background_video_file(&self) -> Option<PathBuf> {
        let preferred = [
            "menu.mp4",
            "menu.webm",
            "menu.mkv",
            "menu.mov",
            "background.mp4",
        ];
        for dir in [self.background_dir(), self.fallback_background_dir()] {
            if let Some(path) = preferred
                .iter()
                .map(|name| dir.join(name))
                .find(|path| path.is_file())
            {
                return Some(path);
            }
            let mut entries = std::fs::read_dir(dir)
                .ok()
                .into_iter()
                .flatten()
                .filter_map(|entry| entry.ok().map(|entry| entry.path()))
                .filter(|path| {
                    path.is_file()
                        && path
                            .extension()
                            .and_then(|extension| extension.to_str())
                            .is_some_and(|extension| {
                                matches!(
                                    extension.to_ascii_lowercase().as_str(),
                                    "mp4" | "webm" | "mkv" | "mov"
                                )
                            })
                })
                .collect::<Vec<_>>();
            entries.sort();
            if let Some(path) = entries.into_iter().next() {
                return Some(path);
            }
        }
        None
    }

    pub fn thumbnail_dir(&self) -> PathBuf {
        self.cache_dir.join("thumbnails")
    }

    pub fn outfits_file(&self, user_id: u64) -> PathBuf {
        self.data_dir
            .join("outfits")
            .join(format!("{user_id}.json"))
    }

    pub fn screenshot_dir(&self) -> PathBuf {
        self.state_dir.join("screenshots")
    }

    /// Phase 2: read-only Sober session import.
    pub fn sober_cookie_file(&self) -> PathBuf {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        home.join(".var/app/org.vinegarhq.Sober/data/sober/cookies")
    }

    pub fn sober_log_dir(&self) -> PathBuf {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        home.join(".var/app/org.vinegarhq.Sober/data/sober/sober_logs")
    }

    pub fn ensure_directories(&self) {
        for path in [
            self.config_dir.clone(),
            self.data_dir.clone(),
            self.fallback_music_dir(),
            self.fallback_sounds_dir(),
            self.fallback_background_dir(),
            self.cache_dir.clone(),
            self.thumbnail_dir(),
            self.state_dir.clone(),
        ] {
            if let Err(error) = std::fs::create_dir_all(&path) {
                log::warn!("Could not create {}: {error}", path.display());
            }
        }
    }
}

pub fn open_path(path: &Path) -> Result<(), String> {
    std::process::Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("could not open {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::Paths;

    #[test]
    fn runtime_assets_are_found_next_to_the_project() {
        let paths = Paths::new();
        assert!(paths.project_assets_dir().join("music/menu.mp3").is_file());
        assert!(
            paths
                .project_assets_dir()
                .join("background/menu.mp4")
                .is_file()
        );

        let music = paths.music_file().expect("bundled menu music");
        assert!(music.starts_with(paths.project_assets_dir()));
        let video = paths.background_video_file().expect("bundled background");
        assert!(video.starts_with(paths.project_assets_dir()));
    }
}
