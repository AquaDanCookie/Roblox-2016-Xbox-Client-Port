# Roblox 2016 Xbox Client — Rust Reimplementation

**Status:** Phase 4 complete; 2016 fidelity, UI, and audio pass complete, 2026-11-03.  
**Original recon:** 2026-10-28  
**Re-verification:** 2026-11-02  
**Phase 1 report:** [`docs/phase-1-report.md`](docs/phase-1-report.md)  
**Phase 2 report:** [`docs/phase-2-report.md`](docs/phase-2-report.md)  
**Phase 3 report:** [`docs/phase-3-report.md`](docs/phase-3-report.md)  
**Phase 4 report:** [`docs/phase-4-report.md`](docs/phase-4-report.md)  
**Fidelity pass:** [`docs/fidelity-pass.md`](docs/fidelity-pass.md)  
**UI/audio pass:** [`docs/ui-audio-pass-2026-11-03.md`](docs/ui-audio-pass-2026-11-03.md)  
**Host:** CachyOS Linux, KDE Plasma Wayland, NVIDIA RTX 3060 (driver 615.78.08)  
**Reference UI:** Roblox Xbox One “V1” layout (late 2015 – mid 2017)  
**Runtime target:** Sober Flatpak 1.8.0, Roblox Android 2.741.1061  

---

## 0. Current state (2026-11-03)

Phase 0 was re-run, Phase 1 and Phase 2 are complete, and Phase 3 launch scope is complete. Phase 4 is also complete. The short version:

- The provided tree is still a 50-file platform shim with no UI, music, or background.
- The 2016 backend is still gone. Modern public APIs still work.
- Sober is installed and has a live authenticated session.
- Cordial is an open-source second runtime and the best public source for the Android launch/auth contract. It is not installed.
- Phase 1 is complete: the shell runs on live public data, builds cleanly, and has screenshots and a navigation demo. See [`docs/phase-1-report.md`](docs/phase-1-report.md).
- Phase 2 is complete: real WebKitGTK login helper, read-only Sober session import, Secret Service storage, auto-restore, sign-out, real avatar headshot, friends, presence, and Robux. See [`docs/phase-2-report.md`](docs/phase-2-report.md).
- Phase 3 is complete for launch: the release binary joined Natural Disaster Survival (`189707`) through Search → Details → Play and DONT SPLASH (`119958473240233`) through Home → Details → Play. Joined is shown only after Sober logs `GameJoinLoadTime` and `onGameLoaded` for the requested place. Crash and timeout states are implemented. See [`docs/phase-3-report.md`](docs/phase-3-report.md).
- Phase 4 is complete: user-supplied looping video background, user-supplied menu music with launch fade, generated foggy fallback, no-root install/uninstall, persistent app log, signed-out Sober branch, 15 tests, and the toast overlay bug fix. See [`docs/phase-4-report.md`](docs/phase-4-report.md).
- The 2016 fidelity pass is complete on this machine: the real 2016 background recording and menu theme are installed locally, Source Sans Pro is bundled, the title/sign-in/home layouts are measured against the captures, and the avatar is a full-body render. The recordings are not in the repository. See [`docs/fidelity-pass.md`](docs/fidelity-pass.md).
- The UI/audio pass is complete (2026-11-03): the shell now follows the square-tile 2016 layout with the measured nav, typography, avatar card, unboxed friend activity, details, and avatar composition. The theme is decoded once, the 76 ms lead silence is trimmed, and a sample-exact in-memory loop replaces the decoder loop. A single-instance lock prevents a second copy of the music. The analog output measured −16.3 dB mean on the previous pass; the loop itself is unit-tested. See [`docs/ui-audio-pass-2026-11-03.md`](docs/ui-audio-pass-2026-11-03.md).
- The app's stored session was removed after testing. A diagnostic command exposed the Sober token in the terminal; the Phase 2 report explains the required rotation.
- The 2016 captures were re-fetched to `/tmp/opencode/refs` for analysis. Side-by-side comparisons are in `docs/screenshots/compare`; reference frames are local-only and not redistributed.
- The former scaffold is repaired and wired. `cargo check`, `cargo test` (15 passed), and `cargo build --release` pass with no warnings.
- No controller is connected, so controller hardware acceptance remains unverified.

Phase 0 evidence and the feasibility memo: [`docs/phase-0-reverification-2026-11-02.md`](docs/phase-0-reverification-2026-11-02.md).

**Gate:** All four phases are complete. The remaining work is optional hardware testing, token rotation, and supplying the user's own 2016 video/music files.

---

## 1. Objective

Build a native Rust desktop client that reproduces the 2016 Roblox Xbox One UI/UX and performs a real flow:

**sign in → home → search → place details → launch a real Roblox experience**

The client runs on CachyOS without root. It is not a C++ port. The 2016 `XboxClient` tree is the UI/UX and behavior reference; Sober is the Android-compatible runtime/launch reference.

## 2. Non-negotiable constraints

- No fake data, fake login, fake launch, or silent fallbacks.
- Every “works” claim must be backed by a run on this machine.
- No root requirement for the client.
- If a step requires the official Android binary or Sober internals, document the limitation and implement the closest honest alternative.
- Do not bundle or download copyrighted menu music or 2016 UI assets that are not in the provided tree.
- Do not bypass anti-cheat, DRM, or account security.
- Keep this file updated at every phase boundary.

## 3. Source-of-truth references

| Area | Reference |
|---|---|
| 2016 platform/auth/launch behavior | `XboxClient/XboxService.cpp` |
| 2016 entry point / protocol activation | `XboxClient/main.cpp` |
| 2016 branding / colors / title ID | `XboxClient/Package.appxmanifest` |
| 2016 controller capabilities | `XboxClient/XboxGameController.cpp`, `ControllerBuffer.*` |
| 2016 logo/splash | `XboxClient/img/roblox.png`, `logo208.png`, `logo100.png`, `robloxWide.png` |
| 2016 UI captures | `/tmp/opencode/refs/k2016/*.jpg`, `/tmp/opencode/refs/xbox2016_title.png` |
| 2016 gameplay capture | `/tmp/opencode/refs/xbox2016_pmObfZGsKIE.mkv` |
| Sober runtime / launch evidence | `~/.var/app/org.vinegarhq.Sober/data/sober/`, Flatpak desktop file |
| Backend/API contract | `docs/backend.md` |
| Detailed recon evidence | `docs/phase-0-recon.md` |
| Re-verification evidence (2026-11-02) | `docs/phase-0-reverification-2026-11-02.md` |
| Phase 1 build and acceptance report | `docs/phase-1-report.md` |
| Phase 2 auth and account report | `docs/phase-2-report.md` |
| UI specification | `docs/ui-reference.md` |
| Reference index / attribution | `docs/references.md` |

## 4. Phase 0 findings (summary)

### 4.1 The provided tree is a platform shim, not the client

`XboxClient` is ~656 KB and 50 files. It contains the Durango/Xbox Live platform integration, controller plumbing, voice chat, networking, and four logo PNGs plus `particles7.rbxl`. It does **not** contain:

- UI scenes or layout definitions
- the Roblox engine
- game catalog assets
- menu music
- the 3D background used by the Xbox shell

The actual 2016 shell must be reconstructed from the reference captures and the platform behavior in `XboxService.cpp`.

### 4.2 The 2016 backend path is retired

- `api.roblox.com` no longer resolves (`curl: (6) Could not resolve host`).
- `/xboxlive/connect`, `/xboxlive/get-roblox-userInfo`, `/xboxlive/has-linked-account`, and `/device/initialize` are therefore unreachable.
- `www.roblox.com/Game/PlaceLauncher.ashx?...` now returns HTTP 404 with an HTML page, not a join script.
- The 2016 Xbox Live title `1465F7BC` and `xbl-authz-actor-10` headers are not usable on Linux.

The modern path is the Android-compatible Roblox web API plus Sober’s authenticated Android runtime.

### 4.3 Sober is a working Android-compatible runtime on this machine

- Sober Flatpak `1.8.0`, runtime `org.gnome.Platform/x86_64/51`, app version `2.741.1061`.
- Desktop file registers `x-scheme-handler/roblox` and `x-scheme-handler/roblox-player`; `xdg-mime query default` resolves both to Sober.
- CLI usage: `sober [--bootstrap] [launch_uri | config]`.
- Sober logs show a real authenticated join (`placeid:15317947079`) and a teleport to another live place (`placeid:5643672124`).
- The APK contains the deep-link forms `roblox://placeId=%d&callId=%s`, `roblox://experiences/start`, and `roblox://navigation/game_details`.

**Conclusion:** Sober is the only proven runtime path on this machine; the exact `roblox://placeId=` CLI handoff is still to be tested in Phase 2. In-process game rendering and direct join are out of scope.

### 4.4 Public catalog APIs work without authentication

Verified live on 2026-10-28:

- `https://apis.roblox.com/search-api/omni-search` → HTTP 200
- `https://apis.roblox.com/explore-api/v1/get-sorts` → HTTP 200
- `https://apis.roblox.com/explore-api/v1/get-sort-content` → HTTP 200
- `https://games.roblox.com/v1/games?universeIds=65241` → HTTP 200
- `https://games.roblox.com/v1/games/votes?universeIds=65241` → HTTP 200
- `https://games.roblox.com/v1/games/65241/favorites/count` → HTTP 200
- `https://thumbnails.roblox.com/v1/games/multiget/thumbnails` → HTTP 200

Authenticated endpoints correctly return 401/403 without a session. This gives Phase 1 real home/search/details data without inventing content.

### 4.5 Login is possible, but captcha/2FA make a native form unreliable

- `POST https://auth.roblox.com/v2/login` returns HTTP 403 with `x-csrf-token` on the first request, as expected.
- `https://users.roblox.com/v1/users/authenticated` returns 401 without a session.
- `https://auth.roblox.com/v1/authentication-ticket` returns 401 without a session.
- A raw username/password form can hit captcha and 2FA challenges that are not reasonably reimplemented.

**Recommended approach:** embedded WebKitGTK login window (`wry` exposes `cookies()` / `cookies_for_url()`), so the user completes captcha/2FA in a real Roblox page; the app captures only `.ROBLOSECURITY`, validates it, and stores it in the Secret Service. A read-only “Use Sober session” import is the fallback because Sober already has a working authenticated session on this machine.

### 4.6 Environment is suitable, with one hardware gap

- Rust 1.99.1 / Cargo 1.99.1.
- NVIDIA RTX 3060, proprietary 615.78.08; Vulkan 1.4.351.
- KDE Wayland; PipeWire 1.6.10 / PulseAudio 15.0.0.
- WebKitGTK 2.54.2 and GTK 3.24.52 are installed.
- Segoe UI is available locally; Noto Sans is available as a redistributable fallback.
- **No controller is connected.** `/dev/input/by-id` contains only keyboard/mouse. Controller acceptance testing is blocked until hardware is available. Keyboard/mouse fallback will be tested now.

## 5. Architecture decision

### 5.1 Stack

| Layer | Choice | Reason |
|---|---|---|
| Window/input | `winit` | Mature Wayland/X11 support, works with wgpu/egui |
| GPU | `wgpu` | Vulkan on NVIDIA, custom background shader |
| UI | `egui` + `egui-wgpu` + `egui-winit` | Precise 10-foot 2D layout, fast iteration, custom focus graph |
| HTTP | `reqwest` + `tokio` + `rustls` | Async, TLS without OpenSSL surprises |
| JSON | `serde` / `serde_json` | Standard |
| Controller | `gilrs` | Linux gamepad hotplug and standard mapping |
| Audio | `rodio` + `symphonia` | Loopable menu music, MP3/WAV/FLAC; OGG Vorbis decodes to silence under rodio 0.23 |
| Images | `image` | Thumbnail decode/cache |
| Login WebView | `wry` (WebKitGTK) | Real Roblox login, captcha/2FA, cookie extraction |
| Secret storage | `keyring` (Secret Service) | No plaintext token by default |
| Logging | `tracing` | Structured logs with redaction |
| CLI/config | `clap`, `directories`, `toml` | Normal Linux app behavior |

Verified 2026-11-02: eframe 0.37 / wgpu 30 runs on the RTX 3060 Vulkan adapter under Wayland. The scaffold is repaired and wired; the shell builds without warnings.

### 5.2 Why not Bevy

Bevy is a good engine, but this project is a 2D, UI-heavy shell. The 2016 3D scene/assets are absent, so Bevy’s main strengths would not be used. `egui` + a custom wgpu background pass gives faster iteration and tighter control over console focus/navigation. If a 3D avatar scene becomes a hard requirement, a Bevy render pass can be added later.

### 5.3 Intended module layout

```text
src/
  main.rs
  app.rs
  config.rs
  ui/
    theme.rs
    focus.rs
    screens/{boot, signin, home, games, search, details, settings, launch}.rs
    widgets/{nav, carousel, game_card, button, prompt, dialog}.rs
  backend/
    http.rs
    auth.rs
    catalog.rs
    thumbnails.rs
    session.rs
    launch.rs
  input/
    gamepad.rs
    keyboard.rs
  audio/
    music.rs
    sfx.rs
  assets.rs
```

## 6. Phase plan

### Phase 0 — Recon and backend contract (re-verified 2026-11-02)

**Deliverables**

- Tree/asset inventory.
- 2016 UI screen inventory and layout specification.
- Documented 2016-vs-modern API comparison.
- Documented Android-compatible auth/session/launch path.
- Environment audit and feasibility memo.
- This plan and the supporting docs.

**Gate:** no backend code; wait for approval.

### Phase 1 — UI shell and live public data (complete 2026-11-02)

**Scope**

- Create the Rust project and CI-friendly build.
- Implement the design system: palette, typography, focus rings, spacing, prompts.
- Implement boot/title, sign-in choice (visual shell), home, games, search, details, settings.
- Use real public APIs for home sorts, search, details, votes, favorites, thumbnails.
- Implement keyboard/mouse navigation and the gamepad focus graph.
- Implement procedural blue background and optional user-supplied background image.
- Implement audio scaffold and user-supplied menu music path.
- Capture side-by-side screenshots against the 2016 references.

**Acceptance criteria**

- `cargo build --release` succeeds on this machine.
- App runs at 1920×1080 on KDE Wayland without root.
- Home loads live sorts/featured content; search returns live results; details load live metadata and thumbnails.
- Keyboard navigation reaches every interactive element; focus is always visible.
- No mock game data in normal operation; network errors show honest error states.
- Controller code builds, but hardware acceptance is marked **unverified** until a controller is connected.
- Screenshot comparison documented; known fidelity gaps listed.

**Result (2026-11-02):** complete. The release binary builds, the live home/search/details flow works, the navigation demo passes, and the audio path was tested with a temporary track. Controller hardware acceptance is unverified. Sign-in and launch are labeled Phase 2/3 stubs. KWin kept the window at 1280x720 logical despite the 1920x1080 request; the virtual canvas is 1920x1080 and scales. See [`docs/phase-1-report.md`](docs/phase-1-report.md).

### Phase 2 — Real authentication and account data (complete 2026-11-02)

**Scope**

- Implement WebKitGTK login window; capture `.ROBLOSECURITY`; validate with `users/authenticated`.
- Store session in Secret Service; sign-out clears it; no password storage.
- Implement optional read-only “Use Sober session” import with explicit consent.
- Show authenticated user in home/nav.
- Add account-dependent data where the API permits: favorite action, friends, presence, Robux.
- Keep honest empty states where the account or API does not permit data.

**Acceptance criteria**

- A real account signs in through the Roblox login page on this machine.
- Authenticated home shows the real account.
- Sign-out removes the stored session and returns to the sign-in screen.
- No cookie/token appears in logs or UI.
- Account-dependent screens show real data or an honest limitation, never fake data.

**Result (2026-11-02):** complete. The Sober import validated the real account; Home, Avatar, Friends, Robux, and Settings Account rendered real data. Auto-restore from the Secret Service and sign-out were verified. The WebKitGTK helper rendered the real Roblox login page under Xvfb; the credential login itself was not completed because the agent does not have the account credentials. The Secret Service fallback file was not exercised. A diagnostic command exposed the Sober token; see the security notice in [`docs/phase-2-report.md`](docs/phase-2-report.md).

### Phase 3 — Game launch, 2016 fidelity, audio, and polish

**Scope**

- Implement launch handoff: `flatpak run org.vinegarhq.Sober 'roblox://experiences/start?placeId=<rootPlaceId>'`, with fallback URI forms.
- Detect Sober presence/session; show actionable errors if Sober is missing or signed out.
- Keep Cordial as a secondary runtime and launch fallback.
- Verify launch by tailing Sober logs for a new `GameJoinLoadTime`/`onGameLoaded` entry for the requested place, plus user-visible confirmation.
- Implement loading/cancel/error states.
- Motion: screen fades/slides, carousel inertia, focus scale, 180–250 ms easing.
- Audio: looping menu music, fade on launch, volume/mute, optional subtle UI SFX.
- Settings: video scale, reduced motion, contrast, audio, account, diagnostics.
- Avatar/Friends/Robux screens: implement only what real APIs support; otherwise honest “not available” states.
- Controller hardware acceptance if a controller is available.
- Performance pass: 1080p60 target, texture cache limits, frame-time logging.
- Packaging: desktop file, icon, install/uninstall notes.

**Acceptance criteria**

- Search → details → PLAY launches the selected place in Sober and the game loads.
- Launch verification is based on runtime evidence, not a timer or a toast.
- If Sober is not signed in, the app explains the limitation and offers to open Sober; it does not claim success.
- Visual comparison against 2016 references is documented screen by screen.
- Menu music plays from a user-supplied file and loops; missing file is reported, not faked.
- Settings persist across restarts.
- Controller navigation verified on hardware, or explicitly marked unverified.
- Release build runs from a clean launch with no developer terminal.

**Result (2026-11-02):** launch scope complete. The release binary joined Natural Disaster Survival (`189707`) through Search → Details → Play and DONT SPLASH (`119958473240233`) through Home → Details → Play. Both joins were confirmed from Sober's `GameJoinLoadTime` and `onGameLoaded` lines. The first URI form, `roblox://experiences/start?placeId=<id>`, was sufficient. Crash detection (`Roblox has crashed`, `out of memory`, `Invalid device`, `RBXCRASH`) and a 90-second timeout show an honest failure state. The signed-out and account-mismatch branches are implemented but were not exercised because Sober is signed in. Cordial was not exercised because it is not installed. Launch music fade, UI SFX, frame-time profiling, and desktop packaging were deferred to Phase 4. See [`docs/phase-3-report.md`](docs/phase-3-report.md).

### Phase 4 — Hardening and release

**Scope**

- Exercise the signed-out Sober branch, account mismatch, cancel/return, and Cordial fallback.
- Fidelity: user-supplied looping video background, user-supplied menu music, music fade, generated fallback scene, title composition against the 2016 capture.
- Packaging: desktop file, icon, install/uninstall scripts, no root.
- Persistent app log and diagnostics path.
- Offline/error/rate-limit behavior.
- API drift guards and contract tests.
- Security review: token handling, WebView data, subprocess arguments, log redaction.
- Crash reporting to local logs only.
- Final QA matrix and release notes.

**Acceptance criteria**

- No known token leakage.
- All core flow steps pass on this machine.
- Deferred launch polish, fallback paths, and packaging are either verified or explicitly re-scoped.
- Documented limitations and legal notes.
- `PLAN.md` and docs updated.

**Result (2026-11-02):** complete. The shell loops a user-supplied video background and menu track, falls back to a generated foggy scene, and fades the music out on launch. The title and sign-in composition match the 2016 capture more closely. The no-root installer and uninstaller were tested in a temporary HOME. The app writes a persistent log and exposes it in Diagnostics. The signed-out Sober branch was verified. A latent toast bug that painted a black overlay over the whole UI was found and fixed. `cargo test` had 13 tests at the end of this phase and 15 after the 2026-11-03 UI/audio pass. See [`docs/phase-4-report.md`](docs/phase-4-report.md).

**Fidelity pass (2026-11-02):** the real 2016 background recording and menu theme are installed in the local app data folder; they are not in the repository. Source Sans Pro is bundled under the SIL OFL. The title, sign-in, and home layouts were measured against the captures, luma-adaptive dimming keeps the dark footage visible, and the avatar is a full-body render. See [`docs/fidelity-pass.md`](docs/fidelity-pass.md).

**Audio pass (2026-11-02):** the original 2016 UI sound set (navigate, select, back, toggle, launch, notice) is installed locally, wired to focus activation, navigation, back, toggle, launch, and toast notices, with a Settings toggle. rodio 0.23 silently decoded the OGG theme to zero samples; converting the theme to a normalized MP3 restored playback, and the analog output monitor measured −16.3 dB mean / −5.4 dB max. Music and UI-sound players are created with `Player::new()` and their queues added only after the source is appended, fixing a race where the mixer discarded empty queues. UI sounds use the local MP3s when present and embedded WAV fallbacks otherwise. The recordings are not in the repository.

**UI/audio fidelity pass (2026-11-03):** the shell was measured again against 1080p real-client captures. The game grids now use square `games/icons` images, the Home grid is 4×2 at 300×300 with 12 px gaps, the avatar card is 450×290 with an upper-body crop, friend activity is unboxed, the details icon is 540×540, and nav/section/title/button/prompt typography follows the 2016 proportions. The menu theme now decodes once, trims 76 ms of lead silence, and loops with a sample-exact in-memory source; a single-instance lock prevents duplicate players. Offscreen Xvfb captures were used so no window was opened on the user's desktop. See [`docs/ui-audio-pass-2026-11-03.md`](docs/ui-audio-pass-2026-11-03.md).

## 7. Key risks

| Risk | Impact | Mitigation |
|---|---|---|
| Roblox login blocks WebView / captcha | Cannot sign in natively | Use real WebView; fallback to read-only Sober session import |
| Sober proprietary internals change | Launch/session detection breaks | Isolate in `launch.rs`; detect and report; no private API dependency beyond CLI/cookie/logs |
| Roblox API changes/rate limits | Home/search/details break | Contract tests, caching, backoff, honest error states |
| No controller on this machine | Controller UX unverified | Keyboard/mouse acceptance now; hardware gate later |
| 2016 intro place absent | Background is a recording, not the engine scene | Public 2016 background recording in local user data; generated fallback; document |
| Local 2016 assets are copyrighted | Distribution risk | Keep them in user data only; no repo copies; docs screenshots use the fallback |
| Copyright/trademark | Distribution risk | Code license only; do not redistribute Roblox assets/music; local fan project |
| Roblox ToS | Account risk | No anti-cheat bypass; use Sober; document unofficial status |
| NVIDIA/Wayland WebKitGTK issues | Login window fails | Test early in Phase 2; fallback to X11/portal or Sober session |

## 8. Decisions

**Resolved 2026-11-02:** antislop during; repair the scaffold; WebView plus read-only Sober import; re-fetch references locally for analysis; keyboard/mouse acceptance now; user-supplied menu music; keep Cordial as a secondary runtime.

**Phase 3 decisions (resolved 2026-11-02):** use `roblox://experiences/start?placeId=<rootPlaceId>` as the primary launch form; require Sober log evidence (`GameJoinLoadTime` plus `onGameLoaded`) before showing Joined; treat the documented crash markers as failures; keep the Sober account check read-only; keep Cordial as an unexercised fallback; add test-only demo flags that drive the real UI.

**Phase 4 decisions (resolved 2026-11-02):** user-supplied video and music only, no bundled 2016 assets; decode video with ffmpeg at 960×540/30 fps; generated foggy scene as the fallback; no in-app fullscreen toggle; install to `~/.local/bin` without root; write a persistent app log; fix the toast overlay by giving `Canvas` an overlay mode.

**Fidelity decisions (resolved 2026-11-02):** install the public 2016 background and theme recordings in local user data only; bundle Source Sans Pro under the SIL OFL; use the full-body avatar thumbnail endpoint; adapt dim/scrim to video luminance; measure title/sign-in/home positions against `t0`, `t20`, and `t65`.

**Audio decisions (resolved 2026-11-02):** use the verified MP3 theme rather than OGG Vorbis; normalize the theme to −16 LUFS; install the original 2016 UI sound set in local user data and keep copyright-free WAV fallbacks embedded; enable UI sounds by default; keep focus-loss muting off by default; default master and music volume to 80%.

All phase decisions are resolved. The numbered questions below are the original Phase 1 decision list and are kept for the record.

1. **Antislop mode:** during (apply while building) or after (audit a finished UI)?
2. **Login source:** embedded Roblox WebView (recommended), read-only Sober session import (fallback), or both?
3. **Menu music:** user-supplied file (recommended), or explicitly approve a licensed replacement? Do not download the copyrighted 2016 track.
4. **Sober session sync:** read-only import only, or also allow explicit write-back so one login covers both apps? Write-back is invasive and needs a separate security review.
5. **Controller:** proceed with keyboard/mouse and mark controller unverified, or wait for hardware before Phase 1 acceptance?
6. **Reference captures:** restore the old `/tmp/opencode/refs` set, allow a local re-fetch of the public 2016 capture for analysis only, or accept a text-only comparison?
7. **Cordial:** keep it as a secondary runtime and launch fallback, or stay Sober-only?
8. **Existing scaffold:** repair the 2,131-line `src/` sketch or replace it with a clean Phase 1 shell?

## 9. Verification log

| Date | Check | Result |
|---|---|---|
| 2026-10-28 | `XboxClient` inventory | 50 files, ~656 KB; platform shim only |
| 2026-10-28 | 2016 `api.roblox.com` endpoints | NXDOMAIN / unreachable |
| 2026-10-28 | 2016 `PlaceLauncher.ashx` | HTTP 404 HTML, not a join script |
| 2026-10-28 | Public search/sorts/games/votes/thumbnails | HTTP 200 |
| 2026-10-28 | `auth.roblox.com/v2/login` CSRF probe | HTTP 403 + `x-csrf-token` |
| 2026-10-28 | `users/authenticated` unauthenticated | HTTP 401 |
| 2026-10-28 | Sober Flatpak/version | 1.8.0, app 2.741.1061 |
| 2026-10-28 | Sober deep-link registration | `roblox` and `roblox-player` → Sober |
| 2026-10-28 | Sober real join evidence | `placeid:15317947079`, teleport `placeid:5643672124` |
| 2026-10-28 | Controller presence | None; keyboard/mouse only |
| 2026-10-28 | WebKitGTK/GTK | 2.54.2 / 3.24.52 installed |
| 2026-10-28 | GPU/Vulkan | RTX 3060, 615.78.08, Vulkan 1.4.351 |
| 2026-10-28 | Audio | PipeWire 1.6.10, PulseAudio 15.0.0 |
| 2026-11-02 | Re-verification: 2016 API host | DNS failure |
| 2026-11-02 | Re-verification: public APIs | search/sorts/details/votes/favorites/thumbnails/media all HTTP 200 |
| 2026-11-02 | Re-verification: Sober session | live join `placeid:7041939546` in the 2026-11-02 log |
| 2026-11-02 | Re-verification: Sober sandbox | network/ipc/wayland/x11/pulseaudio/dri, no host home |
| 2026-11-02 | Re-verification: Android identity | APK 2.741.1061; Cordial trace Android 13 / API 33 |
| 2026-11-02 | Re-verification: eframe/wgpu | ran on RTX 3060 Vulkan under Wayland |
| 2026-11-02 | Re-verification: scaffold wired | 21 compile errors; not Phase 1 complete |
| 2026-11-02 | Re-verification: captures | `/tmp/opencode/refs` missing |
| 2026-11-02 | Re-verification: controller | none detected |
| 2026-11-02 | Phase 1: tests | `cargo test` 7 passed, 0 failed |
| 2026-11-02 | Phase 1: release build | success, 33 MB binary |
| 2026-11-02 | Phase 1: live data | home, search, and details loaded from live APIs |
| 2026-11-02 | Phase 1: navigation demo | Home → Details → Launch → Details → Home → Search → Home → Settings → Home |
| 2026-11-02 | Phase 1: offline state | proxy failure produced an error panel and Retry |
| 2026-11-02 | Phase 1: audio | temporary OGG played and looped; test file removed |
| 2026-11-02 | Phase 1: controller | code builds; hardware acceptance unverified |
| 2026-11-02 | Phase 1: screenshots | `docs/screenshots/` and `docs/screenshots/compare/` |
| 2026-11-02 | Phase 2: build | 8 tests pass; release 34 MB; no warnings |
| 2026-11-02 | Phase 2: auth endpoints | authenticated 200; friends 20; presence 20; currency 200; headshot 200 |
| 2026-11-02 | Phase 2: Sober import | real account validated; Home, Avatar, Friends, Robux populated |
| 2026-11-02 | Phase 2: auto-restore | Secret Service session validated on restart |
| 2026-11-02 | Phase 2: sign-out | keyring entry removed; screen returned to Sign In |
| 2026-11-02 | Phase 2: WebView helper | real Roblox login page rendered under Xvfb |
| 2026-11-02 | Phase 2: token exposure | diagnostic command printed the secret; app keyring cleared; rotation required |
| 2026-11-02 | Phase 3: URI form | `roblox://experiences/start?placeId=189707` produced `GameJoinLoadTime` and `onGameLoaded` |
| 2026-11-02 | Phase 3: Search → Details → Play | release binary joined `189707`; app and Sober logs agree |
| 2026-11-02 | Phase 3: Home → Details → Play | release binary joined DONT SPLASH `119958473240233` |
| 2026-11-02 | Phase 3: failure path | OOM and `Invalid device` detected; Failed UI shown instead of Joined |
| 2026-11-02 | Phase 3: GPU contention | ACIII held 10.4 GiB VRAM; both joins passed after it closed |
| 2026-11-02 | Phase 3: tests | `cargo test` 8 passed; `cargo check` clean |
| 2026-11-02 | Phase 3: release build | 34 MB; sign-in smoke test and both launch flows passed |
| 2026-11-02 | Phase 3: deferred | launch music fade, UI SFX, frame-time profiling, desktop packaging |
| 2026-11-02 | Phase 4: video background | release decoded a looped H.264 test video; status showed playing |
| 2026-11-02 | Phase 4: reduced motion | video disabled; generated fallback used |
| 2026-11-02 | Phase 4: video error | corrupt file reported unavailable; no fake success |
| 2026-11-02 | Phase 4: music | test OGG loaded and looped; status showed playing |
| 2026-11-02 | Phase 4: signed-out Sober | isolated HOME showed the unverified branch on the release binary |
| 2026-11-02 | Phase 4: toast bug | fixed the full-screen black overlay; reload-button toast verified over Settings |
| 2026-11-02 | Phase 4: install | temp HOME install, desktop-file validate, release smoke, purge |
| 2026-11-02 | Phase 4: app log | persistent log written; diagnostics path shown; no token matches |
| 2026-11-02 | Phase 4: release joins | `189707` and `119958473240233` joined after the Phase 4 changes |
| 2026-11-02 | Phase 4: tests | `cargo test` 12 passed at the phase boundary; 13 after the audio pass; `cargo check` clean |
| 2026-11-02 | Fidelity: font | Source Sans Pro bundled from the 2016 source; app log confirms |
| 2026-11-02 | Fidelity: background | real 2016 recording installed locally; release decoded it |
| 2026-11-02 | Fidelity: music | real 2016 theme installed locally; settings showed playing |
| 2026-11-02 | Fidelity: avatar | full-body avatar loaded; home card rendered it |
| 2026-11-02 | Fidelity: layout | title/sign-in/home measured against t0/t20/t65 |
| 2026-11-02 | Fidelity: release joins | release build joined 189707 and 119958473240233; later layout changes do not touch launch.rs |
| 2026-11-02 | Fidelity: repo assets | most screenshots use the generated fallback; Games/Search layout captures use the local background; recordings are not in the app |
| 2026-11-02 | App menu shortcut | installed, `desktop-file-validate` pass, `gtk-launch roblox-2016-xbox` started the app |
| 2026-11-02 | App icon | old 2016 logo embedded as the window icon and installed at 16–256 px hicolor sizes |
| 2026-11-02 | Audio: root cause | rodio 0.23 decoded `menu.ogg` to zero samples (`empty=true len=0 pos=0`); MP3 and WAV played |
| 2026-11-02 | Audio: theme | normalized `menu.mp3` installed; analog output monitor measured −16.3 dB mean / −5.4 dB max |
| 2026-11-02 | Audio: UI sounds | original 2016 source set installed locally; six events wired; embedded WAV fallback decode test passes |
| 2026-11-02 | Audio: settings | UI Sounds toggle present and on by default; master/music defaults set to 80% |
| 2026-11-02 | Audio: tests | `cargo test` 13 passed; release build clean |
| 2026-11-02 | Audio: Settings capture | retaken at 1920×1080; shows the UI Sounds toggle, 80% volumes, and `Playing menu music` |
| 2026-11-03 | UI reference | 1080p real-client frames (`GhCUQpPJDww`, `Xmbbrp1TmHM`) plus the original `pmObfZGsKIE` 720p capture and Fandom title still |
| 2026-11-03 | UI layout | square 300×300 icons, 4×2 Home grid, three 2×2 Games carousels, 450×290 avatar card, unboxed friend rows, measured nav and typography |
| 2026-11-03 | UI details | 540×540 game icon, narrower media column, exact vote counts, 232×80 PLAY and 220×80 FAVORITE |
| 2026-11-03 | Audio loop | one decode per launch; 76 ms lead silence trimmed; sample-exact `LoopSource`; no decoder at the loop point |
| 2026-11-03 | Audio loop playback | 3 s test track played through 3 loop points for 9 s; no silence gap at −50 dB / 20 ms |
| 2026-11-03 | Audio real theme | 8,981,378 frames / 187.112 s; EasyEffects monitor measured −15.8 dB mean / −2.4 dB max |
| 2026-11-03 | Audio duplicate guard | a second normal launch exits with `Roblox 2016 Xbox is already running.` |
| 2026-11-03 | UI screenshots | refreshed title, sign-in, home, games, details, settings audio, and comparisons via offscreen Xvfb |
| 2026-11-03 | Tests/build | `cargo test` 15 passed; `cargo build --release` clean |

## 10. Next step

All four phases and the 2016 fidelity pass are complete. The remaining work is optional and documented in [`docs/fidelity-pass.md`](docs/fidelity-pass.md):

- Rotate the exposed Roblox session. This is required, not optional.
- This machine has the real 2016 background recording and menu theme installed in local user data. They are not in the repository. Other machines use the generated fallback until the user supplies files.
- The actual `ScaledWorldv4.7.rbxl` intro place was not found; the background is a recording of the scene, not an engine render.
- Re-run controller acceptance if hardware becomes available.
- Exercise the account-mismatch and Cordial fallback branches with a controlled setup.
- After signing in, re-check the Home avatar bust crop, friend rows, and Avatar tab against the 2016 reference.
- Confirm the theme and UI clicks are audible on the physical speakers, not just the analog monitor; if not, check EasyEffects routing.
- Add a frame-time profiling pass if the project continues.
- Run a final distribution review for Roblox trademark and asset licensing.

Before re-authenticating, rotate the exposed Roblox session as described in [`docs/phase-2-report.md`](docs/phase-2-report.md). The app's stored copy has been removed.
