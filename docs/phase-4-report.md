# Phase 4 Report: Fidelity, Packaging, and Hardening

**Status:** Complete on 2026-11-02.
**Host:** CachyOS, KDE Wayland, NVIDIA RTX 3060 12 GB.
**Runtime:** Sober Flatpak 1.8.0, Roblox Android 2.741.1061.
**Fidelity pass:** [`fidelity-pass.md`](fidelity-pass.md) documents the exact local 2016 background/theme, bundled Source Sans Pro, measured title/home layout, and full-body avatar render.

Read the security notice at the top of [`phase-2-report.md`](phase-2-report.md) before re-authenticating. The exposed Sober session still needs to be rotated.

---

## 1. The fidelity request

The running shell can now use the 2016 menu video and music, but the app does not ship them.

The original shell background was a real-time 3D scene, not a video file, and the 2016 soundtrack is copyrighted. Neither asset is present in the provided `XboxClient` tree. Bundling either would violate the project constraints, so the app provides the exact presentation layer and lets the user supply the assets:

- A looping video background from `~/.local/share/roblox-2016-xbox/background/`.
- A looping menu track from `~/.local/share/roblox-2016-xbox/music/`.
- A generated foggy fallback that matches the 2016 mood when no video is installed.

This is the closest honest reproduction. It is not the original engine scene or the original soundtrack.

---

## 2. What was built

### 2.1 Video background

`src/video_background.rs` decodes a user-supplied video in a worker thread and uploads frames to an egui texture.

- Accepted names: `menu.mp4`, `menu.webm`, `menu.mkv`, `menu.mov`, `background.mp4`, or the first video file in the folder.
- ffmpeg command: looped input, real-time pacing, no audio, scale/crop to 960×540, 30 fps, RGBA rawvideo.
- The path is passed as a process argument, never through a shell.
- The decoder uses a one-frame channel, so a slow UI drops frames instead of growing memory.
- The video is muted; the menu music is the audio source.
- Reduced motion disables the video and uses the generated fallback.
- If ffmpeg is missing or the file is corrupt, the app keeps the fallback and shows an honest error in Settings.
- Dropping the video kills the ffmpeg process and reaps it.

### 2.2 Generated fallback scene

`src/background.rs` now generates a 960×540 scene instead of the previous flat blue texture:

- dark blue-gray gradient and fog lobes;
- three layered mountain ridges;
- a soft building silhouette with a few lit windows and an antenna;
- blurred edges, bokeh particles, vignette, and grain.

It is generated in code at startup. No frames from the YouTube capture are bundled.

### 2.3 Menu music

`src/audio.rs` still uses `rodio` and a user-supplied file, now with the intended fades:

- Path: `~/.local/share/roblox-2016-xbox/music/menu.mp3` (also `.wav`, `.flac`). OGG Vorbis is not accepted because rodio 0.23 decoded the 2016 theme to zero samples.
- Loop: `repeat_infinite`.
- Fade in: 1.5 seconds.
- Fade out: 0.8 seconds while the launch screen is active.
- Resume: fades back in on return to the shell.
- Focus loss: optional immediate mute, restored on focus.
- Settings: status, path, Open music folder, Reload music.

No 2016 track is bundled or downloaded.

### 2.4 Title and sign-in composition

The title screen now matches the capture used for the reference:

- centered 2016 wordmark from the provided tree;
- two-line tagline “Community Created Gaming.” / “Limitless Possibilities.”;
- “Press Any Button To Begin” at the same vertical position;
- adaptive scrims: strong over a bright user video, light over the generated fallback;
- ghost actions have a subtle dark backdrop so they remain readable over video.

Sign-in keeps the same composition with the real WebView, read-only Sober import, and guest browsing.

### 2.5 Toast overlay fix

A real rendering bug was found while driving the new Reload video button. `draw_toast` created a second `Canvas`, and `Canvas::new` painted a full-screen black rectangle before drawing the toast. The result was a black screen for the four-second toast lifetime.

`Canvas` now has an `overlay` constructor that skips the clear. The Settings screen remains visible under the “Background video reloaded.” toast, and the same fix applies to every toast in the app.

### 2.6 Persistent app log

`src/main.rs` now tees logging to stderr and `~/.local/state/roblox-2016-xbox/app.log`.

- Rotates at 2 MB.
- Default filter: `roblox_2016_xbox=info,warn`, so dependency noise is suppressed.
- Diagnostics shows the app log path and an Open log folder button.
- A token scan of the log found no `.ROBLOSECURITY`, `RBXSESSION`, or authentication-ticket values.

### 2.7 Packaging

- `install.sh` installs the release binary to `~/.local/bin`, the desktop file to `~/.local/share/applications`, and the provided icon to the local icon theme.
- `uninstall.sh` removes those files. `--purge` also removes the app data, music, and background videos.
- No root is required.
- The desktop entry passes `desktop-file-validate`.

### 2.8 2016 fidelity pass

After the launch/packaging work, the shell was matched to the captures:

- The real 2016 background recording and menu theme are installed in local user data. They are not in the repository.
- Source Sans Pro is bundled from the 2016 source under the SIL OFL.
- Title, sign-in, and home positions were measured against `t0`, `t20`, and `t65`.
- The avatar card uses a full-body render from the Roblox thumbnail API.
- Dimming and title scrims adapt to the video luminance.

Full details: [`fidelity-pass.md`](fidelity-pass.md).

---

## 3. Verification

### 3.1 Build and tests

| Check | Result |
|---|---|
| `cargo fmt` | clean |
| `cargo check` | no warnings |
| `cargo test` | 12 passed at the phase boundary; 13 after the audio pass; 15 after the 2026-11-03 UI/audio pass, 0 failed |
| `cargo build --release` | success, 34 MB |
| Release launch flow | joined `189707` after the final build |

The new tests cover the verified URI form, the join marker requirements, wrong-place rejection, and all crash markers.

### 3.2 Video background

A temporary H.264 test video was installed, decoded, and removed.

- Release log: `background video: /home/aqua/.local/share/roblox-2016-xbox/background/menu.mp4`.
- Settings showed `Background video playing`.
- The frame appeared behind the title, home, and settings screens.
- Reduced motion produced no video process and used the fallback.
- A corrupt `menu.mp4` produced `Background video unavailable: The background video ended before producing a frame.`
- The Reload video button was driven through the real focus system:
  ```text
  focus activate settings:reload-background (click=false, confirm=true)
  background video: ...
  ```
  The toast now overlays the Settings screen instead of erasing it.

### 3.3 Music

A temporary OGG was installed to exercise the status path. The status said `Playing menu music`, but a later audible check found that rodio’s Vorbis decode produced zero samples for the real theme. The theme was converted to a normalized MP3; the analog output monitor then measured −16.3 dB mean. The fade volumes are applied from the app loop (`stable_dt`, 1.5 s in, 0.8 s out).

### 3.4 Release launch flows

| Flow | Result |
|---|---|
| Search → Details → Play, Natural Disaster Survival | Joined `189707` at 14:33 on the final release build |
| Home → Details → Play, DONT SPLASH | Joined `119958473240233` at 14:37 on the final release build |
| Sober logs | `GameJoinLoadTime` and `onGameLoaded` matched the requested place |
| Signed-out Sober branch | Isolated HOME showed “Sober session could not be verified” with Launch anyway / Open Sober / Back |

### 3.5 Packaging test

An isolated HOME was used:

- `install.sh` installed the 34 MB binary, desktop file, and icon.
- `desktop-file-validate` passed.
- The installed binary rendered the sign-in screen and exited 0.
- `uninstall.sh --purge` removed the installed app and its data.

### 3.6 Visual comparison

Updated side-by-side images:

- [`screenshots/compare/title.png`](screenshots/compare/title.png)
- [`screenshots/compare/signin.png`](screenshots/compare/signin.png)
- [`screenshots/compare/home.png`](screenshots/compare/home.png)
- [`screenshots/compare/games.png`](screenshots/compare/games.png)
- [`screenshots/compare/details.png`](screenshots/compare/details.png)

The title composition, tagline, prompt, nav hierarchy, featured grid, and details layout match the reference closely. The generated background is a mood approximation, not a reconstruction of the engine scene.

### 3.7 Security scan

- App log scanned for token patterns: no matches.
- Source and docs scanned for literal `.ROBLOSECURITY` or ticket values: no matches.
- Video and audio paths are passed as arguments, not shell strings.
- Launch place IDs are numeric.
- The WebView remains ephemeral.

The earlier diagnostic exposure still requires the user to rotate the Roblox session.

---

## 4. Acceptance criteria

| Criterion | Status |
|---|---|
| User-supplied video loops in the background | **Pass** on debug and release, then removed |
| Reduced motion disables video | **Pass** |
| Missing or corrupt video is reported honestly | **Pass** |
| User-supplied music loops | **Pass**; `menu.mp3` plays and loops, confirmed on the analog output monitor |
| Music fades on launch and resumes | **Implemented**; player output measured, fade timing not separately instrumented |
| Title/sign-in match the 2016 composition | **Pass** in the updated comparisons |
| Search → Details → Play joins | **Pass**, `189707` on the final release |
| Second game joins | **Pass**, DONT SPLASH `119958473240233` |
| Signed-out Sober branch | **Pass** |
| No-root install/uninstall | **Pass** |
| Persistent app log and diagnostics | **Pass** |
| Toast overlays do not erase the UI | **Pass** |
| Controller hardware | **Unverified**, no controller connected |
| Original 2016 video and soundtrack bundled | **Not in the repo**; the real recordings are installed in local user data on this machine |
| Full-body avatar render | **Pass** via `thumbnails.roblox.com/v1/users/avatar` |
| Source Sans Pro shell font | **Pass**; bundled from the 2016 source under the SIL OFL |
| UI SFX | **Pass**: original 2016 source set installed locally; generated WAV fallback embedded; Settings toggle |
| Frame-time/1080p60 profiling | **Not performed** |
| Account mismatch, cancel/return, Cordial | **Implemented, not exercised** |

---

## 5. Limitations

- The exact 2016 background scene and soundtrack are not included in the repository. The real recordings are installed only in this machine's user data; another machine gets the generated fallback and no music until files are supplied.
- Without a user video, the fallback is a still image. This machine has the real recording installed, so the running app has motion.
- The original background was a real-time 3D scene. A video loop is a substitute, not an engine reconstruction.
- The actual `ScaledWorldv4.7.rbxl` intro place was not found in the provided tree or the upstream content.
- The music is audibly verified, but the fade timing itself was not separately measured.
- Controller hardware remains unverified.
- The account-mismatch branch needs a second signed-in account to test safely.
- Cordial is not installed, so its fallback path is unexercised.
- Cancel/return buttons are implemented but were not driven by the screenshot harness.
- Frame-time profiling is not implemented. UI SFX is implemented with the local 2016 set and embedded WAV fallbacks.
- The avatar is a full-body Roblox thumbnail render, not an interactive 3D character; friends are limited to the first 20.
- WebView credential login was not completed end-to-end because the agent does not have the account credentials.
- The Secret Service fallback file was not exercised because Secret Service worked.
- The exposed Sober token still needs rotation.

---

## 6. Files changed in Phase 4

- `src/video_background.rs`: new ffmpeg-backed looping video texture.
- `src/background.rs`: generated foggy fallback scene.
- `src/audio.rs`: fade in/out, focus mute, launch fade.
- `src/canvas.rs`: overlay mode, vertical gradients, video-active flag.
- `src/app.rs`: video lifecycle, reload/stop, adaptive title/sign-in, demo hook, toast overlay.
- `src/main.rs`: persistent log tee, `--demo-settings-video`.
- `src/screens/boot.rs`, `signin.rs`: title/sign-in composition and scrims.
- `src/screens/settings.rs`: video status, open/reload background, app log, ghost actions.
- `src/screens/mod.rs`: ghost button backdrop, stronger bottom gradient.
- `src/launch.rs`: four new launch/log tests.
- `install.sh`, `uninstall.sh`, `packaging/roblox-2016-xbox.desktop`, `packaging/roblox-2016-xbox.png`.
- `README.md`, `PLAN.md`, `docs/ui-reference.md`, `docs/screenshots/*`, `docs/phase-4-report.md`.

---

## 7. Design gate

| Check | Result |
|---|---|
| R-02 copy | Pass: no em dashes in the new UI strings |
| R-17 real numbers | Pass: diagnostics uses real versions and paths, no invented metrics |
| R-23 assets | Pass: provided logo/icon; Source Sans Pro is OFL and bundled; the full-body avatar comes from the public API; the 2016 recordings stay in local user data |
| R-25 contrast | Pass on the generated fallback and the 2016-style dark video; bright user video is dimmed and scrimmed, though unusually bright footage can still reduce small-text contrast |
| R-26 controls | Pass: Reload video driven; folder buttons use the real `open_path` handler |
| R-27 states | Pass: video not installed/loading/playing/unavailable; music states; diagnostics |
| R-32 keyboard | Pass: new controls register in the focus graph; Reload video was activated by Confirm |
| R-35 run | Pass: release built, launch joins, unverified branch, install smoke, screenshots |
| R-37 direction | Pass: 2016 captures and `docs/ui-reference.md`; calm 10-foot composition |
| R-38 honest content | Pass: no fake data; missing assets are labeled, not simulated |

The folder-open buttons were not clicked in the screenshot harness because they spawn an external file manager. They call the same `open_path` handler as the existing config, music, and log folder actions.
