# Phase 0 — Reconnaissance and Feasibility Evidence

**Date:** 2026-10-28  
**Host:** CachyOS Linux, KDE Plasma Wayland  
**Purpose:** Establish what the provided 2016 tree contains, what the 2016 client did, what the modern backend requires, and what can honestly be built on this machine.

> **2026-11-02 note:** this is the original recon. See [`phase-0-reverification-2026-11-02.md`](phase-0-reverification-2026-11-02.md) for current evidence and [`phase-1-report.md`](phase-1-report.md) for the completed shell. The captures were re-fetched to `/tmp/opencode/refs` for analysis. The 2,131-line scaffold was repaired and wired in Phase 1.

---

## 1. Method

Phase 0 used only read-only inspection and live unauthenticated HTTP probes:

- inventory and source reading of `XboxClient`;
- inspection of the installed Sober Flatpak, its state, logs, APK, and desktop registration;
- live probes against public Roblox endpoints;
- frame extraction and OCR from the 2016 gameplay capture;
- environment checks for GPU, Vulkan, audio, WebKitGTK, fonts, and input devices.

No backend code, UI code, or credentials were written or modified. No token values are recorded in this document.

## 2. Provided tree inventory

`XboxClient` is **50 files, ~656 KB**. It is the Durango/Xbox platform integration layer, not the Roblox client.

### 2.1 Files that matter for the reimplementation

| File | Size | Role |
|---|---:|---|
| `XboxClient/XboxService.cpp` | 105,884 B | 2016 Xbox Live auth, account linking, catalog, PlaceLauncher, teleport, session polling |
| `XboxClient/XboxService.h` | 8,500 B | Platform service interface |
| `XboxClient/main.cpp` | 8,793 B | Durango `IFrameworkView`, protocol activation |
| `XboxClient/XboxGameController.cpp` | 12,581 B | Gamepad discovery, RBX gamepad mapping, vibration |
| `XboxClient/ControllerBuffer.*` | 6,278 B | Input history buffering |
| `XboxClient/XboxMultiplayerManager.cpp` | 33,630 B | Xbox Live multiplayer/party |
| `XboxClient/VoiceChat*.cpp/h` | ~40 KB | Xbox voice chat |
| `XboxClient/p2p.cpp/h` | 26,310 B | Xbox peer-to-peer networking |
| `XboxClient/UserTranslator.cpp` | 6,671 B | Xbox user ID translation |
| `XboxClient/XbLiveUtils.cpp` | 15,463 B | Xbox Live helpers |
| `XboxClient/Package.appxmanifest` | 4,285 B | 2016 branding, `#000040` background, Xbox Live title `1465F7BC` |
| `XboxClient/img/roblox.png` | 98,189 B | 1920×1080 RGBA splash/logo |
| `XboxClient/img/robloxWide.png` | 30,698 B | 480×480 logo tile |
| `XboxClient/img/logo208.png` | 10,357 B | 208×208 logo |
| `XboxClient/img/logo100.png` | 4,293 B | 100×100 logo |
| `XboxClient/Durango/Layout/Image/Loose/particles7.rbxl` | 13,374 B | Tiny particle place, not the shell background |

### 2.2 What is absent

The tree does **not** contain:

- UI scenes, layout definitions, or XAML;
- the Roblox engine or renderer;
- catalog/game data;
- the blurred 3D shell background;
- menu music or UI sound effects;
- nav icons or controller prompt art;
- the 2016 avatar/catalog assets.

The four PNGs and `particles7.rbxl` are the only visual assets. Any claim that the tree contains the 2016 UI or music is false.

### 2.3 2016 behavior confirmed in source

- `main.cpp` handles `ActivationKind::Protocol` and forwards the URI to `XboxPlatform::onProtocolActivated`. This confirms the 2016 client was protocol-activatable.
- `Package.appxmanifest` uses `BackgroundColor="#000040"`, `DisplayName="ROBLOX"`, and Xbox Live `TitleId="1465F7BC"`.
- `XboxService.cpp` hardcodes the test place `248409235` (“Cube Eat Cube”) around line 722.
- `XboxService.cpp` uses `https://www.roblox.com/` and `https://api.roblox.com/`.
- The 2016 auth endpoints are `/xboxlive/connect`, `/xboxlive/get-roblox-userInfo`, `/xboxlive/link-existing-user`, `/xboxlive/set-roblox-username-password`, `/xboxlive/has-linked-account`, `/xboxlive/has-set-username-password`, and `/device/initialize`.
- The 2016 join URL is `/Game/PlaceLauncher.ashx?request=RequestGame&placeId=<id>&gamerTag=<tag>`.
- The 2016 controller advertises A/B/X/Y, L1/R1, L2/R2, L3/R3, both thumbsticks, D-pad, Select, and Start (`XboxGameController.cpp`, `findAvailableGamepadKeyCodesAndSet`).

## 3. 2016 UI reference

### 3.1 Sources

- Roblox Wiki (Fandom), “Roblox (Xbox)”: <https://roblox.fandom.com/wiki/Roblox_(Xbox)>
- 2016 gameplay capture: <https://www.youtube.com/watch?v=pmObfZGsKIE>
- Additional captures used during recon:
  - `/tmp/opencode/refs/k2016/t0.jpg` — title screen
  - `/tmp/opencode/refs/k2016/t20.jpg` — sign-in choice
  - `/tmp/opencode/refs/k2016/t65.jpg` — home
  - `/tmp/opencode/refs/k2016/t85.jpg` — games
  - `/tmp/opencode/refs/k2016/gd160.jpg` — game details
  - `/tmp/opencode/refs/xbox2016_title.png` — 2016 title screen
  - `/tmp/opencode/refs/xbox2016_pmObfZGsKIE.mkv` — 867 s gameplay capture

### 3.2 Visual language

- Dark, blurred blue 3D environment behind all shell screens.
- White/light-blue thin text; uppercase section labels with letter spacing.
- Red/white Roblox wordmark at top-left on shell screens.
- Top nav: `HOME AVATAR GAMES FRIENDS ROBUX`.
- Focus: bright light-blue border and glow; selected nav item is white/underlined.
- Green `PLAY` button; grey secondary buttons; orange price tags.
- Controller prompts use a circled letter and a label, e.g. `X Settings`.
- Layout is 16:9, 10-foot UI, generous margins, large targets.

### 3.3 Screens confirmed

1. **Title** — centered logo, “Powering Imagination™”, “Press Any Button To Begin”.
2. **Sign-in choice** — logo, “Community Created Gaming. Limitless Possibilities.”, grey “SIGN UP USING LOST AFFECT”, blue “SIGN IN”.
3. **Home** — top nav; `LOST AFFECT` avatar card; `FEATURED` 4×2 grid; `FRIEND ACTIVITY` list; bottom-right `X Settings`.
4. **Games** — top nav with `GAMES` selected/underlined; horizontal carousels `FEATURED`, `POPULAR`, `TOP RATED`; `...` overflow buttons.
5. **Game details** — `Home` back button; title; large thumbnail; `RATING AND DESCRIPTION` with thumbs and vote bar; `GAME IMAGES` column; green `PLAY`; grey `FAVORITE`; `...` overflow.
6. **Avatar** — large 3D avatar, `MY COLLECTION`, `CATALOG` with orange `R$` prices.
7. **Sign-in / account** — the 2016 app offered create-account, sign-in, or Xbox gamertag paths. The Linux reimplementation cannot use Xbox gamertag identity.

### 3.4 Search

The 2016 capture does not show a dedicated search screen. The top nav has no `SEARCH` item. The reimplementation will add a 2016-consistent search surface reachable from `GAMES` and a `Y` controller shortcut, using the same typography, focus ring, and results grid. This is an explicit adaptation, not a claim of exact 2016 parity.

## 4. Sober runtime evidence

### 4.1 Installation

| Item | Value |
|---|---|
| Flatpak | `org.vinegarhq.Sober` |
| Version | `1.8.0` |
| Branch | `stable` |
| Runtime | `org.gnome.Platform/x86_64/51` |
| Commit | `7b6430c703e8178baa536062099fc5a1e0c7cf213c0789695fa7660b62cb9d26` |
| Roblox app version | `2.741.1061` |
| APK | `~/.var/app/org.vinegarhq.Sober/data/sober/packages/x86_64/com.roblox.client/base.apk` |

### 4.2 Deep-link registration

`/var/lib/flatpak/exports/share/applications/org.vinegarhq.Sober.desktop`:

```ini
MimeType=x-scheme-handler/roblox;x-scheme-handler/roblox-player;
Exec=/usr/bin/flatpak run --branch=stable --arch=x86_64 --command=sober --file-forwarding org.vinegarhq.Sober -- @@u %u @@
```

`xdg-mime query default x-scheme-handler/roblox` and `x-scheme-handler/roblox-player` both return `org.vinegarhq.Sober.desktop`.

CLI usage printed by Sober:

```text
Usage: /proc/self/exe [--bootstrap] [launch_uri | config]
```

### 4.3 APK deep links

Strings found in the cached official APK:

```text
roblox://experiences/start
roblox://navigation/game_details
roblox://placeId=%d&callId=%s
roblox://placeId=%d&gameInstanceId=%s&callId=%s
roblox://placeId=%d&reservedServerAccessCode=%s&callId=%s
```

The simple `roblox://placeId=<id>` form is the planned launch URI. The exact CLI handoff will be tested in Phase 2; it is **not** claimed as verified yet.

### 4.4 Real join evidence

Sober logs from 2026-10-28 show an authenticated session and real joins:

```text
[FLog::ClientRunInfo] The base url is https://www.roblox.com/
[FLog::GameJoinLoadTime] ... placeid:15317947079 ... universeid:5283170248 ...
rbx.jni: onGameLoaded() SessionReporterState_GameLoaded placeId:15317947079
[FLog::UgcExperienceController] doTeleport: joinScriptUrl https://assetgame.roblox.com/Game/Join.ashx?ticketVersion=2&ticket=<redacted>...
rbx.jni: onGameLoaded() SessionReporterState_GameLoaded placeId:5643672124
```

This proves the Android runtime, authentication, join, and teleport paths work on this machine. It does **not** prove the `roblox://placeId=` CLI handoff; that is a Phase 2 test.

### 4.5 Sober session storage

`~/.var/app/org.vinegarhq.Sober/data/sober/cookies` is a single-line semicolon-delimited string. Cookie names observed:

- `GuestData`
- `.ROBLOSECURITY`
- `RBXEventTrackerV2`

The `.ROBLOSECURITY` value is present and was **not** printed or copied into project files. Any session import must be explicit, read-only by default, and must never log the value.

`~/.var/app/org.vinegarhq.Sober/data/sober/state` reports `v1.app_version = 2.741.1061`.

## 5. Live API probes

All probes were unauthenticated and read-only on 2026-10-28.

### 5.1 Public catalog

| Endpoint | Status | Notes |
|---|---|---|
| `https://apis.roblox.com/search-api/omni-search?searchQuery=adopt%20me&sessionId=<uuid>&pageToken=&pageType=all` | 200 | Returns `searchResults` with `universeId`, `name`, `playerCount`, votes, etc. |
| `https://apis.roblox.com/explore-api/v1/get-sorts?sessionId=<uuid>&device=console&country=all` | 200 | Returns `sorts` including filters and sort IDs |
| `https://apis.roblox.com/explore-api/v1/get-sort-content?sessionId=<uuid>&sortId=top-trending&device=console` | 200 | Returns `games` with `universeId`, `rootPlaceId`, `name`, `playerCount`, votes |
| `https://games.roblox.com/v1/games?universeIds=65241` | 200 | Returns name, description, creator, playing, visits, maxPlayers, genre, timestamps |
| `https://games.roblox.com/v1/games/votes?universeIds=65241` | 200 | `{"data":[{"id":65241,"upVotes":4424302,"downVotes":449986}]}` |
| `https://games.roblox.com/v1/games/65241/favorites/count` | 200 | `{"favoritesCount":11924741}` |
| `https://thumbnails.roblox.com/v1/games/multiget/thumbnails?universeIds=65241&size=768x432&format=Png&isCircular=false` | 200 | Returns `imageUrl` on `tr.rbxcdn.com` |

### 5.2 Authentication

| Endpoint | Status | Notes |
|---|---|---|
| `POST https://auth.roblox.com/v2/login` | 403 | First request returns `x-csrf-token`; retry with header is required |
| `GET https://users.roblox.com/v1/users/authenticated` | 401 | Correct unauthenticated behavior |
| `POST https://auth.roblox.com/v1/authentication-ticket` | 401 | Correct unauthenticated behavior; needs session + CSRF |
| `GET https://auth.roblox.com/` | 404 | Root 404 is normal; endpoint paths matter |

### 5.3 2016 endpoints

| Endpoint | Result | Meaning |
|---|---|---|
| `https://api.roblox.com/` | DNS failure (`curl: (6) Could not resolve host`) | The entire 2016 API host is gone |
| `https://api.roblox.com/xboxlive/has-linked-account` | DNS failure | 2016 Xbox link flow is gone |
| `https://api.roblox.com/xboxlive/get-roblox-userInfo` | DNS failure | 2016 user-info flow is gone |
| `https://api.roblox.com/device/initialize` | DNS failure | 2016 browser-tracker flow is gone |
| `https://www.roblox.com/Game/PlaceLauncher.ashx?request=RequestGame&placeId=248409235&gamerTag=test` | 404 + HTML | No longer returns a join script |

**Conclusion:** a literal 2016 backend port is impossible. The reimplementation must use modern web APIs for data/auth and Sober for launch.

## 6. Environment audit

| Item | Value |
|---|---|
| OS | CachyOS Linux |
| Session | Wayland, KDE Plasma |
| GPU | NVIDIA GeForce RTX 3060 |
| Driver | 615.78.08 (proprietary) |
| Vulkan | 1.4.351 (NVIDIA), 1.4.363 (llvmpipe) |
| Audio | PipeWire 1.6.10 / PulseAudio 15.0.0 |
| Rust | 1.99.1 |
| Cargo | 1.99.1 |
| WebKitGTK | 2.54.2 |
| GTK | 3.24.52 |
| Wayland client | 1.26.0 |
| Fonts | Segoe UI present; Noto Sans present |
| Controller | **None detected**; `/dev/input/by-id` has keyboard/mouse only |
| `DISPLAY` / `WAYLAND_DISPLAY` | `:0` / `wayland-1` |

### 6.1 Controller gap

No `/dev/input/js*` and no gamepad in `/dev/input/by-id`. The controller implementation can be built and unit-tested, but hardware acceptance is **blocked**. Phase 1 will test keyboard/mouse and mark controller navigation unverified.

### 6.2 WebKitGTK suitability

WebKitGTK 2.54.2 and GTK 3.24.52 are installed. `wry` exposes cookie access on Linux and uses WebKitGTK. This makes a real Roblox login window feasible without reimplementing captcha/2FA. The login window must be tested on NVIDIA/Wayland in Phase 2; a fallback to read-only Sober session import remains available.

## 7. Feasibility memo

### 7.1 Achievable on this machine

- 2016-style boot/title, sign-in choice, home, games, search, details, settings.
- Real public catalog data (sorts, search, metadata, votes, favorites, thumbnails).
- Real authentication through a Roblox WebView, with captcha/2FA handled by Roblox.
- Secure local session storage via Secret Service.
- Optional read-only import of Sober’s existing session, with explicit consent.
- Real launch by handing a `roblox://placeId=` URI to Sober.
- Menu music from a user-supplied file, looped and faded.
- Keyboard/mouse navigation now; gamepad support built and hardware-tested later.

### 7.2 Not achievable without the official Android binary / Sober

- In-process Roblox game rendering.
- Direct game join without Sober.
- Xbox Live identity/auth (`1465F7BC`, `xbl-authz-actor-10`).
- The retired 2016 `api.roblox.com` backend.
- Exact 2016 3D shell background (assets absent).
- Exact 2016 menu music (copyrighted, absent).
- A literal 1:1 C++ port.

### 7.3 Closest honest alternative

A native Rust **shell** that uses modern Roblox web APIs for discovery/auth and Sober’s official Android runtime for launch. The app owns the 2016 UI/UX; Sober owns game execution. The handoff is visible and verified, not hidden behind a fake “playing” state.

## 8. Reproducible checks

Safe commands used during recon:

```bash
# Tree inventory
find XboxClient -type f -printf '%10s  %p\n' | sort -k2

# 2016 endpoint status
curl -sS -o /dev/null -w '%{http_code}\n' --max-time 15 \
  'https://www.roblox.com/Game/PlaceLauncher.ashx?request=RequestGame&placeId=248409235&gamerTag=test'

# Public catalog
curl -s --max-time 15 'https://games.roblox.com/v1/games?universeIds=65241'
curl -s --max-time 15 'https://games.roblox.com/v1/games/votes?universeIds=65241'
curl -s --max-time 15 'https://thumbnails.roblox.com/v1/games/multiget/thumbnails?universeIds=65241&size=768x432&format=Png&isCircular=false'

# Auth probe (no credentials)
curl -s -o /dev/null -D - --max-time 15 -X POST \
  -H 'Content-Type: application/json' \
  -d '{"ctype":"Username","cvalue":"__probe__","password":"__probe__"}' \
  'https://auth.roblox.com/v2/login'

# Sober registration
xdg-mime query default x-scheme-handler/roblox
flatpak info org.vinegarhq.Sober | grep -E 'Version|Branch|Runtime'
```

Do not run cookie-dumping commands in shared logs. If inspecting Sober’s cookie jar, print names only and redact values.

## 9. Phase 0 conclusion

The project is feasible as a **2016-style Rust shell + Sober launch handoff**, with real data and real authentication. It is not feasible as a standalone game client. The next phase should build the shell against live public APIs, then wire authentication and launch in Phase 2.

**Phase 0 gate:** no Phase 1 code has been started. Awaiting approval.
