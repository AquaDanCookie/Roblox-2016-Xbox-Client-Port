use std::fs::{self, File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

use fs2::FileExt;

pub struct InstanceGuard {
    file: Option<File>,
}

impl InstanceGuard {
    pub fn acquire(state_dir: &Path) -> Option<Self> {
        // An old build that predates the advisory lock would not be caught by
        // the lock file. Scan /proc so any live copy of the shell blocks a
        // second launch (two copies were audible as two menu themes).
        if other_instance_running() {
            return None;
        }
        if let Err(error) = fs::create_dir_all(state_dir) {
            log::warn!(
                "Could not create the state directory {}: {error}",
                state_dir.display()
            );
            return Some(Self { file: None });
        }

        let path = state_dir.join("instance.lock");
        let legacy_pid = fs::read_to_string(&path)
            .ok()
            .and_then(|text| text.trim().parse::<u32>().ok());
        let mut file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
        {
            Ok(file) => file,
            Err(error) => {
                log::warn!("Could not open {}: {error}", path.display());
                return Some(Self { file: None });
            }
        };

        // An OS-held advisory lock is released automatically when the
        // process exits, so a second launch cannot race the PID write the
        // way the old create_new/read-PID guard could. Two instances playing
        // the menu theme at once was the source of the doubled music.
        if file.try_lock_exclusive().is_err() {
            return None;
        }

        // Also refuse the older PID-only lock format. This catches an
        // already-running copy from before the advisory-lock fix even though
        // that process does not hold an flock.
        if let Some(pid) = legacy_pid
            && pid != std::process::id()
            && Path::new(&format!("/proc/{pid}")).exists()
        {
            let comm = fs::read_to_string(format!("/proc/{pid}/comm")).unwrap_or_default();
            if comm.trim().starts_with("roblox-2016-xbo") {
                let _ = file.unlock();
                return None;
            }
        }

        let _ = file.set_len(0);
        let _ = file.seek(SeekFrom::Start(0));
        let _ = writeln!(file, "{}", std::process::id());
        let _ = file.sync_all();
        Some(Self { file: Some(file) })
    }
}

fn other_instance_running() -> bool {
    let current = std::process::id();
    let Ok(entries) = fs::read_dir("/proc") else {
        return false;
    };
    for entry in entries.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u32>().ok())
        else {
            continue;
        };
        if pid == current {
            continue;
        }
        let comm = fs::read_to_string(format!("/proc/{pid}/comm")).unwrap_or_default();
        if comm.trim().starts_with("roblox-2016-xbo") {
            log::warn!("Refusing to start: another shell process {pid} is running");
            return true;
        }
    }
    false
}

impl Drop for InstanceGuard {
    fn drop(&mut self) {
        if let Some(file) = &self.file {
            let _ = file.unlock();
        }
    }
}
