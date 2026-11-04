# Phase 3 Report: Game Launch and Runtime Handoff

**Status:** Phase 3 launch scope complete on 2026-11-02. Phase 4 was completed later that day; see [`phase-4-report.md`](phase-4-report.md).
**Host:** CachyOS, KDE Wayland, NVIDIA RTX 3060 12 GB.
**Runtime:** Sober Flatpak 1.8.0, Roblox Android 2.741.1061.

Read the security notice at the top of [`phase-2-report.md`](phase-2-report.md) before re-authenticating. The exposed Sober session still needs to be rotated.

---

## 1. What was built

### 1.1 Launch contract

The verified URI form is:

```text
roblox://experiences/start?placeId=<rootPlaceId>
```

`launch.rs` builds the URI from the numeric place ID and spawns:

```bash
flatpak run org.vinegarhq.Sober 'roblox://experiences/start?placeId=<rootPlaceId>'
```

The URI is passed as a separate process argument, never through a shell string.

- Sober is used when its Flatpak is installed.
- Cordial is the fallback, launched through `xdg-open` with the same URI. It is detected but was not exercised because it is not installed on this machine.
- If neither runtime is available, the launch fails with an actionable message instead of claiming success.
- `roblox://placeId=<id>` and the other APK forms remain documented fallbacks in [`backend.md`](backend.md); they were not needed after the first form worked.

### 1.2 Session alignment

Before handing off, the app reads Sober's cookie file read-only, validates `.ROBLOSECURITY` with `GET https://users.roblox.com/v1/users/authenticated`, and compares the result with the shell account.

- Same account: launch.
- Different account: show both names, with "Launch as Sober" and "Cancel".
- No readable or valid Sober session: show the error, with "Launch anyway" and "Open Sober".
- The app does not write to Sober's cookie file or pass the token to any command line.

### 1.3 Launch state machine

The launch screen has explicit states instead of a timer or a fake toast:

| State | Meaning |
|---|---|
| `Checking` | Validating the Sober session |
| `Launching` | Starting the runtime |
| `Waiting` | Polling Sober's log for the requested place |
| `Joined` | Both the join record and `onGameLoaded` matched the place |
| `SessionUnverified` | Sober's session could not be read or validated |
| `Mismatch` | Shell account and Sober account differ |
| `Failed` | Runtime error, crash, or 90-second timeout |

`LaunchLog` snapshots `~/.var/app/org.vinegarhq.Sober/data/sober/sober_logs/latest.log`, follows the symlink when Sober rotates the log, and reads only new bytes. A join requires all three markers:

- `placeid:<requested id>` in `GameJoinLoadTime`
- `onGameLoaded`
- `placeId:<requested id>`

Crash markers are `Roblox has crashed`, `out of memory`, `Invalid device`, and `RBXCRASH`. When one appears, the UI switches to `Failed` immediately instead of waiting for the timeout.

### 1.4 Launch UI

- 2016-style dark panel, spinner, and centered status text.
- Place name, current phase, and an honest explanation.
- On failure: Retry, Open Sober, and Back.
- On success: Return to shell.
- On mismatch: Launch as Sober or Cancel.
- On an unverified session: Launch anyway, Open Sober, or Back.
- The Sober log path is shown in waiting and failure states so the user can inspect it.

### 1.5 Integration

- `details.rs` passes the live `rootPlaceId` to `App::open_launch`.
- `settings.rs` reports whether the Cordial fallback is installed.
- Test-only flags `--demo-launch`, `--demo-play`, and `--demo-search-play` exercise the real UI path. They do not bypass the state machine.

---

## 2. Verification

### 2.1 Build

| Check | Result |
|---|---|
| `cargo fmt` | clean |
| `cargo check` | 0 warnings |
| `cargo test` | 8 passed, 0 failed |
| `cargo build --release` | 34 MB binary |
| Release binary smoke test | sign-in screen rendered, exit 0 at 13:49 |

The release binary was then used for both launch tests below.

### 2.2 First URI form

At 13:22 the first form was tested directly:

```bash
flatpak run org.vinegarhq.Sober 'roblox://experiences/start?placeId=189707'
```

The Sober log contained:

```text
[FLog::GameJoinLoadTime] ... placeid:189707, universeid:65241
rbx.jni: onGameLoaded() SessionReporterState_GameLoaded placeId:189707
```

The plain `roblox://placeId=` form was not needed.

### 2.3 Search → Details → Play

Run on the release binary at 13:49 with `--screen search --demo-search-play`:

```text
demo action: Navigate(Down)
demo action: Navigate(Left)
demo action: Confirm
demo action: Confirm
Sober account check ok for launch
launching place 189707 with Sober
launch joined place 189707
```

The app selected Natural Disaster Survival from live search results, opened its details, and joined place `189707`. The Sober log confirmed `GameJoinLoadTime` and `onGameLoaded` for that place. Screenshot: [`screenshots/launch.png`](screenshots/launch.png).

### 2.4 Home → Details → Play, second game

Run on the release binary at 13:50 to prove the path is not hardcoded to one game:

```text
demo action: Navigate(Down)
demo action: Navigate(Right)
demo action: Confirm
demo action: Confirm
Sober account check ok for launch
launching place 119958473240233 with Sober
launch joined place 119958473240233
```

This is the first featured game, DONT SPLASH. The Sober log confirmed `placeid:119958473240233` and `onGameLoaded` for the same place. Screenshot: [`screenshots/launch-dontsplash.png`](screenshots/launch-dontsplash.png).

### 2.5 Failure path

While Assassin's Creed III Remastered was holding 10.4 GiB of the 12 GiB VRAM, launches failed intermittently. Two concrete failures:

- `OutOfMemoryGraphics (Failed to allocate memory. size = 524288, alignment = 64)` for place `189707`.
- `[Vulkan Loader] ERROR: vkGetDeviceQueue: Invalid device` for DONT SPLASH.

The app detected both and showed "Launch failed" with the GPU explanation, the Sober log path, and Retry / Open Sober / Back at 13:40:31. It did not show "Joined". Screenshot: [`screenshots/launch-failed.png`](screenshots/launch-failed.png).

This was environmental: after closing the other game, free VRAM rose from 56 MiB to 10.5 GiB and both joins passed. The release shell itself could not allocate a wgpu device while VRAM was exhausted; it launched normally after the game closed.

---

## 3. Acceptance criteria

| Criterion | Status |
|---|---|
| Search → details → PLAY launches the selected place in Sober and the game loads | **Pass** for place `189707`; the Sober log and app log both confirm the join |
| Launch verification uses runtime evidence, not a timer or toast | **Pass**: `GameJoinLoadTime` + `onGameLoaded` + matching place ID |
| Missing or signed-out Sober is explained honestly | **Implemented, not exercised end-to-end**: Sober is signed in, so the unverified and mismatch branches could not be triggered without changing Sober's state |
| Loading, cancel, error, and return states | **Loading, error, and joined states verified**; cancel and return buttons are implemented but were not driven in a demo |
| Visual comparison with 2016 references | Carried from Phase 1 (`screenshots/compare/`); the new launch states are documented here, not re-compared screen by screen |
| User-supplied menu music loops | Verified in Phase 1; not re-run this phase. No track is currently installed |
| Settings persist across restarts | Implemented from Phase 1; not re-verified this phase |
| Controller navigation verified on hardware | **Unverified**: no controller is connected |
| Release build runs from a clean launch | **Pass**: release binary rendered and completed both launch flows |

---

## 4. Limitations and deferred work

- The game renders in Sober's window, not inside the shell. This project is a shell and launch handoff, not an in-process Roblox client.
- Join confirmation depends on Sober's proprietary log format and path. If Sober changes either, the app reports a timeout instead of guessing.
- Crash markers are deliberately narrow. A new crash string that does not match them will end in the 90-second timeout, not a false "Joined".
- The unverified and account-mismatch branches are implemented but untested against a signed-out or different Sober account. Testing them requires changing Sober's session, which I did not do.
- Cordial was not exercised because it is not installed.
- Controller hardware acceptance remains blocked.
- Launch music fade, subtle UI SFX, frame-time profiling, and desktop packaging were not implemented in this phase. They are deferred to Phase 4 rather than claimed as done.
- The failure screenshot was captured under severe GPU contention. It documents the error UI, not the normal result on a free GPU.

---

## 5. Files changed in Phase 3

- `src/launch.rs`: URI, runtime spawn, log snapshot/polling, crash detection.
- `src/app.rs`: launch state, session check, process polling, retry/leave/continue actions, test sequences.
- `src/screens/launch.rs`: launch states, spinner, buttons, log path.
- `src/screens/details.rs`: Play passes the real root place ID.
- `src/screens/settings.rs`: Cordial fallback status.
- `src/api.rs`: authenticated-account check for Sober.
- `src/config.rs`: Sober log directory.
- `src/main.rs`: `--demo-launch`, `--demo-play`, `--demo-search-play`.
- `docs/backend.md`: verified URI and log markers.
- `docs/screenshots/launch.png`, `launch-dontsplash.png`, `launch-failed.png`.
- `PLAN.md`, `README.md`, `docs/screenshots/README.md`.
