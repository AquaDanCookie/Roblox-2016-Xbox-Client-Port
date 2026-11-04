# Phase 0 Re-verification, 2026-11-02

**Status:** Phase 0 recon re-run on this machine. No Phase 1 code was written or modified at the time.
**Verdict:** Feasible as a 2016-style Rust shell with real catalog data, real login, and a Sober or Cordial launch handoff. Not feasible as a standalone game runtime.

> **Phase 1 update:** the scaffold described in section 7 was later repaired and wired, and the shell is now complete. See [`phase-1-report.md`](phase-1-report.md).

This document replaces the stale "Phase 0 complete" claims where the machine now says otherwise. It does not delete the earlier recon; it records what is still true, what changed, and what is blocked.

---

## 1. Method

Read-only work only:

- inventory and source reading of `XboxClient`;
- live HTTP probes against Roblox endpoints;
- inspection of the installed Sober Flatpak, desktop entry, sandbox permissions, APK, cookie names, and logs;
- inspection of the open-source Cordial project's documentation and launch code;
- build and run checks of the existing Rust scaffold on this machine.

No tokens, cookie values, join tickets, usernames, or passwords are recorded here.

---

## 2. Provided tree

`XboxClient` is **50 files, 656 KB**. It is the Durango/Xbox platform shim, not the client.

| Item | Evidence |
|---|---|
| 2016 API hosts | `XboxService.cpp:90-91`: `https://www.roblox.com/` and `https://api.roblox.com/` |
| 2016 auth endpoints | `XboxService.cpp:106-120`: `/xboxlive/connect`, `/xboxlive/get-roblox-userInfo`, `/xboxlive/link-existing-user`, `/xboxlive/has-linked-account`, `/device/initialize` |
| 2016 join URL | `XboxService.cpp:1300-1322`: `/Game/PlaceLauncher.ashx?request=RequestGame&placeId=...&gamerTag=...` |
| 2016 test place | `XboxService.cpp:713-722`: place `248409235` ("Cube Eat Cube") |
| Protocol activation | `main.cpp:107-113`: `ActivationKind::Protocol` forwarded to `onProtocolActivated` |
| Branding | `Package.appxmanifest`: `BackgroundColor="#000040"`, `DisplayName="ROBLOX"`, Xbox Live `TitleId="1465F7BC"` |
| Controller keys | `XboxGameController.cpp:321-342`: A/B/X/Y, L1/R1, L2/R2, L3/R3, both sticks, D-pad, Select, Start |

The tree contains **no UI scenes, no engine, no menu music, no 3D shell background, and no controller prompt art**. The only visual assets are four PNGs and one tiny particle place:

- `XboxClient/img/roblox.png` (1920x1080 RGBA, the 2016 splash with "Powering Imagination")
- `XboxClient/img/robloxWide.png` (480x480)
- `XboxClient/img/logo208.png` (208x208)
- `XboxClient/img/logo100.png` (100x100)
- `XboxClient/Durango/Layout/Image/Loose/particles7.rbxl` (13 KB, not the shell background)

The splash image was checked directly. It is the title screen, not a full UI kit.

---

## 3. UI surfaces to preserve

The 2016 shell is a 10-foot, 16:9 interface. The reference screens and the implementation targets are:

| Surface | 2016 reference | Implementation note |
|---|---|---|
| Title | Centered wordmark, "Powering Imagination", "Press Any Button To Begin" | Use `XboxClient/img/roblox.png`; any controller button, Enter, Space, or click advances |
| Sign-in choice | Logo, tagline, grey create-account and blue sign-in buttons | Modern replacement: Roblox web login; keep the 2016 hierarchy |
| Home | Top nav `HOME AVATAR GAMES FRIENDS ROBUX`; avatar card; `FEATURED` 4x2 grid; friend activity | Live sorts and thumbnails fill the grid; friends need an authenticated API |
| Games | Horizontal carousels `FEATURED`, `POPULAR`, `TOP RATED` | Map to live sorts; labels may follow the live sort names |
| Search | Not shown in the 2016 capture | Explicit adaptation: same typography, focus ring, and results grid; reachable with Y or a magnifier |
| Game details | Back, title, large thumbnail, rating and description, game images, green `PLAY`, grey `FAVORITE` | Live metadata, votes, media, and thumbnails |
| Launch | Dim overlay, thin spinner, "Joining ..." | Honest wait state; success only after Sober log evidence |
| Settings | Account, audio, video, controls, diagnostics | `X` on Home, Menu, or Esc |

### 3.1 Assets and music

- The provided splash and logos are usable locally. They are Roblox trademarks, not redistributable as a product identity.
- **No menu music is present.** The 2016 track is copyrighted and absent. The build reads a user-supplied file from `~/.local/share/roblox-2016-xbox/music/menu.{ogg,mp3,flac,wav}`. If missing, the UI says so; it does not substitute another track.
- The Roblox Wiki says the 2016 title screen was built by asimo3089, with Tomarty animating the camera and background. The background scene is not in the tree. The Rust shell uses a procedural dark blue scene, with an optional user-supplied still image.

### 3.2 Reference capture gap

The earlier plan cites local captures under `/tmp/opencode/refs/`. **That directory no longer exists.** Home, games, details, and motion comparison cannot be done from the current machine state.

This blocks the Phase 1 acceptance item "side-by-side screenshots" until one of these happens:

1. the user restores the captures;
2. the user gives permission to re-fetch the public 2016 gameplay capture for local analysis only, with no bundling;
3. the user accepts a text-only comparison against the written UI spec and the Fandom stills.

The 2016 title screen is available on the Roblox Wiki:
<https://roblox.fandom.com/wiki/Roblox_(Xbox)>

---

## 4. Backend contract

### 4.1 The 2016 backend is gone

Live probes on 2026-11-02:

| Probe | Result |
|---|---|
| `https://api.roblox.com/` | DNS failure, `curl: (6) Could not resolve host` |
| `https://www.roblox.com/Game/PlaceLauncher.ashx?request=RequestGame&placeId=248409235&gamerTag=test` | HTTP 404, HTML page, not a join script |

The 2016 Xbox Live identity (`1465F7BC`, `xbl-authz-actor-10`) cannot be used on Linux. A literal 2016 backend port is impossible.

### 4.2 Modern public APIs work

Live unauthenticated probes on 2026-11-02, with a fresh UUID `sessionId`:

| Purpose | Endpoint | Status | Observed shape |
|---|---|---|---|
| Search | `https://apis.roblox.com/search-api/omni-search?searchQuery=natural%20disaster&sessionId=<uuid>&pageToken=&pageType=all` | 200 | 40 game results; first result `universeId=65241`, `rootPlaceId=189707`, `name="Natural Disaster Survival"`, `creatorName="Stickmasterluke"` |
| Search without session | same URL without `sessionId` | 200 | 0 results; the session ID is required in practice |
| Home sorts | `https://apis.roblox.com/explore-api/v1/get-sorts?sessionId=<uuid>&device=console&country=all` | 200 | `top-trending`, `up-and-coming`, `top-playing-now`, `fun-with-friends`, `top-revisited`, plus a `filters_v5` pill group |
| Sort content | `https://apis.roblox.com/explore-api/v1/get-sort-content?sessionId=<uuid>&sortId=top-trending&device=console` | 200 | 98 game entries |
| Game details | `https://games.roblox.com/v1/games?universeIds=65241` | 200 | `rootPlaceId=189707`, `playing=5532`, `visits=4534997125`, `maxPlayers=30`, `favoritedCount=11933247` |
| Votes | `https://games.roblox.com/v1/games/votes?universeIds=65241` | 200 | up/down counts |
| Favorites | `https://games.roblox.com/v1/games/65241/favorites/count` | 200 | count |
| Thumbnails | `https://thumbnails.roblox.com/v1/games/multiget/thumbnails?universeIds=65241&size=768x432&format=Png&isCircular=false` | 200 | CDN image URL |
| Place to universe | `https://apis.roblox.com/universes/v1/places/920587237/universe` | 200 | universe ID |
| Game media | `https://games.roblox.com/v2/games/65241/media` | 200 | 10 image entries |
| Authenticated user, no cookie | `https://users.roblox.com/v1/users/authenticated` | 401 | correct unauthenticated behavior |
| Login, no credentials | `POST https://auth.roblox.com/v2/login` | 403 | returns `x-csrf-token`; retry needs that header |

These are the endpoints the shell can use for home, search, details, and thumbnails without inventing content.

### 4.3 Authentication

The recommended path remains a real Roblox login page in WebKitGTK, because captcha and 2FA are handled by Roblox:

1. Open `https://www.roblox.com/login` in a dedicated WebKitGTK window.
2. User completes the login, captcha, and any 2FA challenge.
3. Read `.ROBLOSECURITY` from the WebKit cookie manager for `https://www.roblox.com`.
4. Validate it with `GET https://users.roblox.com/v1/users/authenticated`.
5. Store it in the Secret Service via `keyring`.
6. Clear WebView data on sign-out.

Do not build a native username/password form as the primary path. A raw form can hit captcha and 2FA states that are not reasonable to reimplement, and a silent failure is worse than an honest WebView.

A read-only import of Sober's existing session is a useful fallback:

```text
~/.var/app/org.vinegarhq.Sober/data/sober/cookies
```

The file is mode `0600`. The host user can read it. Cookie names observed: `RBXEventTrackerV2`, `.ROBLOSECURITY`, `GuestData`. Values were not recorded. Import must be explicit, read-only, validated, and never logged.

### 4.4 Android-compatible identity

Sober runs the official Roblox Android APK:

- package `com.roblox.client`
- installed app version `2.741.1061` (Sober state file)
- ABI `x86_64`
- base APK at `~/.var/app/org.vinegarhq.Sober/data/sober/packages/x86_64/com.roblox.client/base.apk`

Cordial's ground-truth trace of the same APK records the platform shape:

- User-Agent: `... ROBLOX Android App 2.732.1043 Tablet Hybrid GooglePlayStore RobloxApp/2.732.1043`
- `OS Ver. = 13, Lvl = 33`
- `Build = googleProdRelease`
- `Arch = X86_64`
- `BaseUrl = www.roblox.com/`

So the Android client presents **Android 13, API level 33**, as a tablet hybrid on Google Play. Sober's installed APK is newer (`2.741.1061`), so the version token in its User-Agent will differ. There is no separate "API version" header for the web endpoints; the web API is path-versioned (`/v1`, `/v2`). The shell should use a truthful user agent of its own, not impersonate the Android client.

Sources:
- Cordial trace README: <https://github.com/luohoa97/cordial/blob/main/docs/traces/README.md>
- Cordial `SDK_INT = 33` comment: <https://github.com/luohoa97/cordial/blob/main/native/android_classes.cpp>

### 4.5 Join ticket

The Android runtime mints and redeems its own join ticket. A live Sober log shows the join script URL:

```text
https://assetgame.roblox.com/Game/Join.ashx?ticketVersion=2&ticket=<redacted>
```

The front-end does not mint that ticket and does not need it. It hands a place ID to Sober, and Sober joins with its own session.

### 4.6 Launch URI

The APK's own strings include:

```text
roblox://placeId=%d&callId=%s
roblox://placeId=%d&gameInstanceId=%s&callId=%s
roblox://placeId=%d&reservedServerAccessCode=%s&callId=%s
roblox://experiences/start
roblox://navigation/game_details
```

Cordial measured this exact form reaching the engine's `Game.launch` message:

```text
roblox://experiences/start?placeId=1818
```

Community Sober documentation also lists:

```text
roblox://experience?placeId=<place-id>
placelauncherurl:https://www.roblox.com/Game/PlaceLauncher.ashx?request=RequestGame&placeId=<place-id>
```

Sober's own CLI usage line is:

```text
Usage: sober [--bootstrap] [launch_uri | config]
```

**The exact CLI handoff has not been tested on this machine.** Test order for Phase 3:

1. `flatpak run org.vinegarhq.Sober 'roblox://experiences/start?placeId=<id>'`
2. `flatpak run org.vinegarhq.Sober 'roblox://placeId=<id>'`
3. `flatpak run org.vinegarhq.Sober 'roblox://experience?placeId=<id>'`
4. `xdg-open 'roblox://...'` as the registered-handler fallback

Do not claim any form works until a Sober log shows the join.

---

## 5. Sober Linux integration

| Item | Value |
|---|---|
| Flatpak ID | `org.vinegarhq.Sober` |
| Installed version | `1.8.0`, branch `stable`, from Flathub |
| Runtime | `org.gnome.Platform/x86_64/51` |
| Desktop entry | `/var/lib/flatpak/exports/share/applications/org.vinegarhq.Sober.desktop` |
| MIME types | `x-scheme-handler/roblox`, `x-scheme-handler/roblox-player` |
| Exec line | `/usr/bin/flatpak run --branch=stable --arch=x86_64 --command=sober --file-forwarding org.vinegarhq.Sober -- @@u %u @@` |
| Default handler | `xdg-mime query default` returns `org.vinegarhq.Sober.desktop` for both schemes |
| Sandbox | `shared=ipc;network;`, sockets `fallback-x11;pulseaudio;wayland;`, device `dri`; no host home filesystem |
| Logs | `~/.var/app/org.vinegarhq.Sober/data/sober/sober_logs/` |
| Session jar | `~/.var/app/org.vinegarhq.Sober/data/sober/cookies` |

### 5.1 What the handoff can pass

- A URI as one separate argv argument: `flatpak run org.vinegarhq.Sober '<uri>'`.
- The registered-handler route: `xdg-open '<uri>'`, which uses the desktop file's `--file-forwarding` and `@@u %u @@`.

### 5.2 What the handoff cannot pass

- No environment variables, no file descriptors, and no files from the host home. The sandbox has no host home filesystem.
- No auth cookie or ticket through the simple place URI. The Android engine accepts `roblox://` and `robloxmobile://` for game links, not the desktop `roblox-player://` ticket format. Cordial's measured translation drops the desktop `gameinfo` ticket by default.
- No guarantee that Sober is signed into the same account as the shell. The shell must read Sober's cookie jar, validate the account, and warn on a mismatch.
- No reliable Wayland focus restoration after Sober exits. The shell can minimize and restore best-effort, but the compositor decides.

### 5.3 Launch verification

A launch is successful only when the Sober log shows both:

- `FLog::GameJoinLoadTime` with `placeid:<requested-place>`
- `rbx.jni: onGameLoaded()` with `placeId:<requested-place>`

Record the log size or offset before launch, then poll for new lines. On timeout, show the log path and an honest failure. Do not use a timer or a toast as proof.

### 5.4 Cordial as a second runtime

Cordial is an open-source Roblox Android runtime for Linux:

- repository: <https://github.com/luohoa97/cordial>
- Flatpak ID: `io.github.luohoa97.Cordial`
- same official Android APK, loaded natively
- registered for `roblox://` and `roblox-player://`
- CLI join path: `cordial-run --join-url 'roblox://experiences/start?placeId=1818'`
- its engine network stack is untouched; it talks to Roblox directly

Cordial is **not installed** on this machine. It is not the primary target. It is a credible fallback or secondary runtime if Sober changes or breaks, and its documentation is the best public evidence for the Android launch and auth contract.

---

## 6. Environment audit

| Item | Value |
|---|---|
| OS | CachyOS Linux |
| Session | Wayland, KDE Plasma |
| GPU | NVIDIA GeForce RTX 3060, 12 GB |
| Driver | 615.78.08 proprietary |
| Vulkan | 1.4.351 NVIDIA; 1.4.363 llvmpipe |
| Audio | PipeWire 1.6.10 through PulseAudio 15.0.0 |
| Rust | 1.99.1 |
| Cargo | 1.99.1, rustup stable toolchain |
| WebKitGTK | 2.54.2 (`webkit2gtk-4.1` and `webkitgtk-6.0`) |
| GTK | 3.24.52 |
| Secret Service | `secret-tool` present; D-Bus session present |
| Controller | **None detected**. `/dev/input/by-id` has keyboard, mouse, and microphone only |
| Reference captures | `/tmp/opencode/refs` is gone |

### 6.1 Rendering smoke test

The existing 31-line eframe stub was built and run for 25 seconds, then killed by `timeout` (exit 124). wgpu selected the NVIDIA RTX 3060 Vulkan adapter. The app did not crash.

One local warning: the Vulkan loader reports a broken implicit layer at `/usr/share/vulkan/implicit_layer.d/../../../lib/liblsfg-vk-layer.so` (`vkGetInstanceProcAddr` missing). wgpu skips it. It is a Lossless Scaling frame-generation layer, not part of this project. It can be ignored or disabled for cleaner logs.

### 6.2 Controller gap

No gamepad is connected. Controller code can be written and unit-tested, but hardware acceptance is **blocked**. Keyboard and mouse acceptance can proceed now.

---

## 7. Existing Rust scaffold

`src/` contains **2,131 lines across 10 modules** that `PLAN.md` does not mention:

```text
api.rs  audio.rs  canvas.rs  config.rs  focus.rs
images.rs  main.rs  models.rs  theme.rs  util.rs
```

What is actually true:

- `main.rs` is a 31-line eframe stub. It declares none of the other modules.
- `cargo check` on the binary passes because only `main.rs` is compiled.
- The stub ran on Wayland/Vulkan as described above.
- When the modules are wired in a temporary copy, the build fails with **21 errors**, including:
  - `egui` is used directly but is not a direct dependency (eframe re-exports it);
  - `reqwest 0.13` needs the `query` feature for `RequestBuilder::query`;
  - `tokio::join!(...).await` is wrong; `tokio::join!` already awaits and returns a tuple;
  - type mismatches in the theme/canvas code.

So the scaffold is a sketch, not Phase 1 completion. It should be reviewed, repaired, or replaced. It is not deleted.

There are no screen modules, no app state machine, no controller loop, no auth, and no launch code yet.

---

## 8. Feasibility memo

**Preserve as-is:** the title screen, dark blue 10-foot visual language, top nav labels, focus ring, green `PLAY` and grey secondary buttons, controller prompt grammar, and the story of the screens. The provided splash is usable. The 2016 layout is documented well enough to rebuild.

**Reimplement:** the UI shell, focus graph, live catalog, WebView login, session storage, and launch handoff. The engine and renderer are not reimplemented.

**Blocked:** in-process game rendering, a direct game join without Sober or Cordial, the retired 2016 `api.roblox.com` backend, Xbox Live identity, the exact 2016 3D background, the exact 2016 soundtrack, and controller hardware acceptance.

**Stack recommendation:** `eframe` plus `egui` for the shell, with `wgpu` and `winit` underneath. This is justified by the audit: the app is a 2D, UI-heavy shell with no 3D assets, the eframe/wgpu path already runs on this NVIDIA Wayland machine, and custom painting gives exact control over the 2016 layout and focus behavior. Bevy would add a 3D engine whose main strength is unused. If a 3D avatar scene becomes mandatory, a Bevy render pass can be added later.

Supporting crates:

| Layer | Choice | Reason |
|---|---|---|
| HTTP | `reqwest` with `rustls` and `query` | Async, TLS without OpenSSL surprises |
| JSON | `serde` / `serde_json` | Standard, matches the live API shapes |
| Async | `tokio` | Required by reqwest and the image loader |
| Controller | `gilrs` | evdev/udev, no XInput assumption |
| Audio | `rodio` + `symphonia` | PipeWire through the PulseAudio compatibility layer |
| Login | `wry` / WebKitGTK | Real Roblox page for captcha and 2FA |
| Secrets | `keyring` | Secret Service, no plaintext token by default |
| Logging | `tracing` or `log` | Redacted diagnostics |

Wayland is the primary path. `winit` can fall back to X11 if needed. The Sober handoff is a separate process, so the shell does not need to share a window or event loop with it. Focus restoration is best-effort under Wayland.

The GPU is NVIDIA proprietary Vulkan, not Mesa. `wgpu` selected it successfully. The llvmpipe adapter is present but should not be chosen for the app.

**Closest honest alternative if the launch path fails:** keep the 2016 UI as the front-end, hand the place ID to Sober or Cordial, and report the real result from their logs. Do not fake a join.

---

## 9. Decisions needed before Phase 1

1. **antislop mode:** during or after?
2. **Phase 1 go-ahead:** approve the shell scope, or revise it?
3. **Login source:** WebView, read-only Sober import, or both?
4. **Menu music:** user-supplied file, or skip until supplied?
5. **Reference captures:** restore them, allow a local re-fetch for analysis only, or accept a text-only comparison?
6. **Controller:** proceed keyboard/mouse and mark controller unverified, or wait for hardware?
7. **Cordial:** keep it as a secondary runtime, or stay Sober-only?

---

## 10. Verification log

| Date | Check | Result |
|---|---|---|
| 2026-11-02 | `XboxClient` inventory | 50 files, 656 KB; platform shim only |
| 2026-11-02 | 2016 `api.roblox.com` | DNS failure |
| 2026-11-02 | 2016 `PlaceLauncher.ashx` | HTTP 404 HTML |
| 2026-11-02 | Public search/sorts/details/votes/favorites/thumbnails/media | HTTP 200; live shapes recorded above |
| 2026-11-02 | `users/authenticated` without cookie | HTTP 401 |
| 2026-11-02 | `auth.roblox.com/v2/login` probe | HTTP 403 with `x-csrf-token` |
| 2026-11-02 | Sober Flatpak | 1.8.0, stable, runtime GNOME 51 |
| 2026-11-02 | Sober MIME handlers | both `roblox` and `roblox-player` resolve to Sober |
| 2026-11-02 | Sober sandbox | network, ipc, wayland/x11, pulseaudio, dri; no host home |
| 2026-11-02 | Sober live join evidence | `placeid:7041939546` and `onGameLoaded` in the 2026-11-02 log |
| 2026-11-02 | Sober join ticket evidence | `assetgame.roblox.com/Game/Join.ashx?ticketVersion=2&ticket=<redacted>` |
| 2026-11-02 | Android identity | APK 2.741.1061; Cordial trace says Android 13 / API 33, tablet hybrid |
| 2026-11-02 | Rust toolchain | 1.99.1 / 1.99.1 |
| 2026-11-02 | GPU/Vulkan | RTX 3060, 615.78.08, Vulkan 1.4.351 |
| 2026-11-02 | Session | Wayland, KDE |
| 2026-11-02 | Audio | PipeWire 1.6.10 / PulseAudio 15.0.0 |
| 2026-11-02 | Controller | none detected |
| 2026-11-02 | eframe/wgpu stub | ran on NVIDIA Vulkan; killed by 25 s timeout, exit 124 |
| 2026-11-02 | Existing scaffold wired in a temp copy | 21 compile errors; not Phase 1 complete |
