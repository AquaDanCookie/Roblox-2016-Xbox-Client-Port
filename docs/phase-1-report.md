# Phase 1 Report: UI Shell and Live Catalog

**Status:** Complete on 2026-11-02. Phase 2 has not started.
**Binary:** `target/release/roblox-2016-xbox` (33 MB, release build).
**Host:** CachyOS, KDE Wayland, NVIDIA RTX 3060, driver 615.78.08.

> **Phase 2 update:** the sign-in stub, account tab, and Avatar/Friends/Robux placeholders were replaced with real authentication and account data. See [`phase-2-report.md`](phase-2-report.md). This report remains the Phase 1 snapshot.

---

## 1. What was built

The existing 2,131-line scaffold was repaired and wired into a runnable app. The build now has no compiler warnings.

- `eframe`/`egui` shell with a 1920x1080 virtual canvas that scales to the window.
- `wgpu` renderer on Vulkan, `winit` windowing, native Wayland.
- Screens: boot/title, sign-in, home, games, search, game details, launch preview, settings, and honest placeholders for Avatar, Friends, and Robux.
- Live catalog data: explore sorts, sort content, omni-search, game details, votes, favorites, media, and thumbnails.
- Focus graph with directional navigation, keyboard and mouse support, and controller code through `gilrs`.
- Menu music pipeline through `rodio`/`symphonia`, with volume, mute, focus-loss mute, reload, and an honest missing-file state.
- Settings persistence through `~/.config/roblox-2016-xbox/config.toml`.
- Procedural dark blue background, Segoe UI and Segoe UI Light typography, 2016 color tokens, focus rings, and square buttons.
- Dev CLI for deterministic screenshots and navigation tests.

### 1.1 Screens

| Screen | State |
|---|---|
| Boot/title | Provided 2016 splash, "Press Any Button To Begin" |
| Sign-in | 2016 hierarchy, Create Account opens the Roblox sign-up page, guest browsing, Phase 2 sign-in stub |
| Home | Live featured grid, avatar placeholder, honest friend-activity empty state |
| Games | Three live sort rows, paging through the `...` controls, four tiles per row |
| Search | Live omni-search, field focus, debounce/Enter, loading, empty, and error states |
| Details | Live title, creator, genre, rating bar, description, metadata, three media images, Play, Favorite, overflow |
| Launch | 2016 loading composition, spinner, explicit Phase 3 label, Back |
| Settings | Account, Audio, Video, Controls, Diagnostics |
| Avatar/Friends/Robux | Honest "not available in this build" panels, no fake data |

### 1.2 Input

| Input | Action |
|---|---|
| Arrows / WASD | Move focus |
| Enter / Space | Select |
| Esc / Backspace | Back or open Settings from a top-level screen |
| Tab / Shift+Tab | Cycle focus |
| Y | Search |
| X | Settings |
| M | Mute |
| F12 | Save a screenshot |
| Controller stick / D-pad | Move focus, with repeat |
| A / B / X / Y | Select, back, settings, search |
| LB / RB | Previous or next top-level tab |
| LT / RT | Page the active game row |
| Menu | Settings |

Controller code compiles and the mapping is in place. No controller is connected on this machine, so hardware acceptance is **unverified**.

### 1.3 Live data

Verified during the build:

- Home uses the live `top-trending` sort and loads eight thumbnails.
- Games uses the first three live sorts.
- Search returned 40 results for "natural disaster" and displays the first eight.
- Details loaded Natural Disaster Survival (universe 65241) with 4.4M up votes, 450.2K down votes, 4.5B visits, 11.9M favorites, and three media images.
- Thumbnails require the live thumbnail API and are cached under `~/.cache/roblox-2016-xbox/thumbnails/`.

### 1.4 Audio

A generated temporary OGG was placed at `~/.local/share/roblox-2016-xbox/music/menu.ogg` to prove the pipeline. The Audio settings screen reported "Playing menu music" and showed the path. The test file was removed afterward, so the shipped state is "Menu music not installed" until the user supplies a track. No copyrighted music is bundled.

---

## 2. Acceptance evidence

| Check | Result |
|---|---|
| `cargo test` | 7 passed, 0 failed |
| `cargo build --release` | Success, 33 MB binary |
| Release binary launch | Runs on KDE Wayland and the NVIDIA Vulkan adapter |
| Boot/title | Provided splash renders |
| Live home | Eight live thumbnails render |
| Live search | 40 live results for "natural disaster" |
| Live details | Metadata, votes, favorites, and three images render |
| Keyboard navigation | Demo flow reached Home, Details, Launch, Search, and Settings |
| Focus visibility | Focus line or ring visible on every focused control |
| Offline state | `HTTPS_PROXY=http://127.0.0.1:9` produced "Could not reach Roblox" and a Retry button |
| Audio | Temporary OGG played and looped |
| Controller | Not hardware-tested |
| Screenshots | Captured at 1280x720 logical; see `docs/screenshots/` |

Demo flow excerpt:

```text
screen -> Home
demo action: Navigate(Down)
demo action: Navigate(Right)
demo action: Confirm
screen -> Details
demo action: Confirm
screen -> Launch
demo action: Back
screen -> Details
demo action: Back
screen -> Home
demo action: Search
screen -> Search
demo action: Back
demo action: Back
screen -> Home
demo action: Settings
screen -> Settings
demo action: Back
screen -> Home
```

The demo uses the same action and focus path as keyboard and controller input. It does not replace hardware controller testing.

---

## 3. Screenshots

Our renders:

- `docs/screenshots/title.png`
- `docs/screenshots/signin.png`
- `docs/screenshots/home.png`
- `docs/screenshots/games.png`
- `docs/screenshots/search.png`
- `docs/screenshots/details.png`
- `docs/screenshots/launch.png`
- `docs/screenshots/settings-account.png`
- `docs/screenshots/settings-audio.png`
- `docs/screenshots/settings-video.png`
- `docs/screenshots/settings-controls.png`
- `docs/screenshots/settings-diagnostics.png`

Side-by-side comparisons with the 2016 capture:

- `docs/screenshots/compare/title.png`
- `docs/screenshots/compare/signin.png`
- `docs/screenshots/compare/home.png`
- `docs/screenshots/compare/games.png`
- `docs/screenshots/compare/details.png`

The comparison images contain 2016 reference frames. They are local analysis material only and must not be redistributed. The app does not bundle them.

---

## 4. Known gaps

These are the Phase 1 gaps, kept as history. Phase 2 resolved sign-in and account data, Phase 3 resolved launch, and Phase 4 added the video/music paths, the capture tagline, and the generated background. The 3D avatar remains a gap.

- **No 3D avatar.** The 2016 avatar was rendered by the Roblox engine. The shell uses a labeled silhouette. The 3D scene and avatar assets are not in the provided tree.
- **Background approximation.** The 2016 shell used a blurred 3D scene. The shell uses a generated dark blue scene with a soft light beam.
- **Games layout.** The shell fits three full rows with slightly smaller tiles. The 2016 capture cuts off the third row and places the overflow buttons below each row. The shell puts them to the right for usability.
- **Sign-in is Phase 2.** The sign-in button shows an honest Phase 2 message. Guest browsing reaches the live catalog. Create Account opens the real Roblox sign-up page.
- **Launch is Phase 3.** The launch screen is a labeled preview of the 2016 loading state. It does not claim a join.
- **No menu music supplied.** The audio path is implemented and was tested with a temporary tone. The user chooses the track.
- **Title tagline.** The provided splash uses "Powering Imagination". The 2016 capture used "Community Created Gaming. Limitless Possibilities." The sign-in screen uses the community tagline; Phase 4 moved the title screen to it as well.
- **No controller hardware.** Controller mapping is implemented and compiled but not accepted on hardware.
- **Window size.** KWin kept the window at 1280x720 logical (1500x843 physical) despite the 1920x1080 initial request. The shell's virtual canvas is 1920x1080 and scales correctly. The window can be resized by the user. The in-app fullscreen toggle produced a black frame under KWin Wayland, so it was removed instead of shipped broken; fullscreen is left to the window manager.
- **Settings layout.** Settings is a full-screen panel rather than the 2016 overlay. The controls are real; the visual treatment is an approximation.
- **Friend, Avatar, and Robux data.** These need authenticated APIs and are placeholders in Phase 1.

---

## 5. Next phase

Phase 2, matching the prompt's phase plan:

1. WebKitGTK login window at `https://www.roblox.com/login`.
2. Capture and validate `.ROBLOSECURITY`.
3. Store it in the Secret Service through `keyring`.
4. Add the read-only Sober session import with explicit consent.
5. Show the authenticated account on Home and in Settings.
6. Add sign-out and session clearing.
7. Implement favorite actions with CSRF handling if the API supports it.
8. Add friends and presence where the account permits; keep honest empty states.
9. Leave launch to Phase 3.

The Phase 1 public catalog work already covers the prompt's Phase 2 "home feed, search, place details" requirement. Phase 2 therefore focuses on authentication and account-dependent data.

---

## 6. Reproduce

```bash
cargo test
cargo build --release
target/release/roblox-2016-xbox
target/release/roblox-2016-xbox --screen home --lock-input --screenshot /tmp/home.png --wait 10
target/release/roblox-2016-xbox --screen home --demo --lock-input
```

The reference comparison images are generated from local 2016 captures and are not part of the app.
