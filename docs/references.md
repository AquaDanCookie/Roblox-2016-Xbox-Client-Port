# Reference Index

This file records where the 2016 UI/behavior references came from. The captures are **not** part of the project and must not be redistributed. The 2016 Xbox platform shim itself is also not included; paths beginning with `XboxClient/` refer to the local reference copy used during analysis.

## 2016 UI / behavior

| Reference | Location | Notes |
|---|---|---|
| Roblox Wiki (Fandom), “Roblox (Xbox)” | <https://roblox.fandom.com/wiki/Roblox_(Xbox)> | 2016 title screen and UI stills |
| 2016 gameplay capture | <https://www.youtube.com/watch?v=pmObfZGsKIE> | Primary motion/UI reference; local copy at `/tmp/opencode/refs/xbox2016_pmObfZGsKIE.mkv` |
| Real 2016 client running in 2026 | <https://www.youtube.com/watch?v=GhCUQpPJDww> | 1080p title/home/avatar/details frames; primary 2026-11-03 layout reference |
| 2016 client title build | <https://www.youtube.com/watch?v=Xmbbrp1TmHM> | 1080p title-screen composition |
| 2025 2016-UI recreation | <https://www.youtube.com/watch?v=u6Sc53ztspQ> | Secondary cross-check only, not source of truth |
| Xbox UI Launcher trailer | <https://www.youtube.com/watch?v=nD_OScrernE> | Secondary cross-check only, not source of truth |
| Additional local captures | `/tmp/opencode/refs/xboxui_*.webm` | Local reference only; source URLs were not recorded |
| 2016 source tree | Local reference, not included | Platform shim; auth/controller/protocol behavior |
| 2016 upstream content | <https://github.com/Julien-Rodot/Roblox-2016-> | `content/fonts`, `content/ui` textures, and CoreScripts used for font and UI analysis |
| 2016 Source Sans Pro | `assets/fonts/` | Bundled under the SIL OFL 1.1; copied from the 2016 source |
| 2016 intro place name | `XboxClient/XboxService.cpp` | `kIntroPlace = "rbxasset://ScaledWorldv4.7.rbxl"`; the actual intro place is not in the tree |
| 2016 particle test place | `XboxClient/Durango/Layout/Image/Loose/particles7.rbxl` | Older/commented-out intro place; used for scene analysis only |
| 2016 background recording | local user data only | Public 2016 background animation; installed locally, not redistributed |
| 2016 menu theme | local user data only | Public 2016 soundtrack recording; installed locally, not redistributed |
| 2016 splash/logo | `XboxClient/img/roblox.png` | 1920×1080 RGBA; provided tree |
| 2016 logo tiles | `XboxClient/img/logo208.png`, `logo100.png`, `robloxWide.png` | Icons/fallbacks |

Local frame sets used during recon:

- `/tmp/opencode/refs/k2016/t0.jpg` — title
- `/tmp/opencode/refs/k2016/t20.jpg` — sign-in choice
- `/tmp/opencode/refs/k2016/t65.jpg` — home
- `/tmp/opencode/refs/k2016/t85.jpg` — games
- `/tmp/opencode/refs/k2016/gd160.jpg` — game details
- `/tmp/opencode/refs/k2016/t100.jpg` — avatar/catalog
- `/tmp/opencode/refs/xbox2016_title.png` — title still

## Sober / runtime evidence

| Reference | Location | Notes |
|---|---|---|
| Sober desktop entry | `/var/lib/flatpak/exports/share/applications/org.vinegarhq.Sober.desktop` | Registers `roblox` and `roblox-player` schemes |
| Sober state | `~/.var/app/org.vinegarhq.Sober/data/sober/state` | `v1.app_version = 2.741.1061` |
| Sober cookie jar | `~/.var/app/org.vinegarhq.Sober/data/sober/cookies` | Names only in docs; never print values |
| Sober APK | `~/.var/app/org.vinegarhq.Sober/data/sober/packages/x86_64/com.roblox.client/base.apk` | Official Android client; deep-link strings |
| Sober logs | `~/.var/app/org.vinegarhq.Sober/data/sober/sober_logs/` | Real join/teleport evidence |
| Flatpak info | `flatpak info org.vinegarhq.Sober` | Version/branch/runtime |
| Flathub listing | <https://flathub.org/apps/org.vinegarhq.Sober> | Install source and version |
| Official Sober docs | <https://vinegarhq.org/Sober/> | Installation, configuration, FAQ |
| Community tips (launch URI) | <https://soberdocs.github.io/docs/Tips.html> | Community documentation, not official; launch URI forms |

## Cordial / Android runtime evidence

| Reference | URL | Notes |
|---|---|---|
| Cordial repository | <https://github.com/luohoa97/cordial> | Open-source Roblox Android runtime |
| Cordial deep-link analysis | <https://github.com/luohoa97/cordial/blob/main/docs/analysis/deep-links.md> | Measured `roblox://experiences/start?placeId=` reaching `Game.launch` |
| Cordial launch translation | <https://github.com/luohoa97/cordial/blob/main/crates/cordial-runtime/src/deeplink.rs> | `roblox-player://` to `roblox://` translation |
| Cordial ground-truth trace | <https://github.com/luohoa97/cordial/blob/main/docs/traces/README.md> | Android 13 / API 33, tablet hybrid UA |
| Cordial Android SDK | <https://github.com/luohoa97/cordial/blob/main/native/android_classes.cpp> | `SDK_INT = 33` |
| Cordial sign-in research | <https://github.com/luohoa97/cordial/blob/main/docs/design/sign-in.md> | Cookie sync and login boundaries |
| Phase 0 re-verification | `docs/phase-0-reverification-2026-11-02.md` | Local evidence and feasibility memo |

## Roblox APIs

| Reference | URL | Notes |
|---|---|---|
| Search | `https://apis.roblox.com/search-api/omni-search` | Public; send a `sessionId` (results are empty without one) |
| Explore sorts | `https://apis.roblox.com/explore-api/v1/get-sorts` | `device=console` |
| Sort content | `https://apis.roblox.com/explore-api/v1/get-sort-content` | Public |
| Game details | `https://games.roblox.com/v1/games?universeIds=...` | Public |
| Place → universe | `https://apis.roblox.com/universes/v1/places/<placeId>/universe` | Public |
| Votes | `https://games.roblox.com/v1/games/votes?universeIds=...` | Public |
| Favorites | `https://games.roblox.com/v1/games/<universeId>/favorites/count` | Public |
| Game icons (square tiles) | `https://thumbnails.roblox.com/v1/games/icons` | Public; `size=512x512` |
| Thumbnails | `https://thumbnails.roblox.com/v1/games/multiget/thumbnails` | Public; retained for media/other uses |
| Full-body avatar render | `https://thumbnails.roblox.com/v1/users/avatar` | Public; `size=420x420` |
| Headshot | `https://thumbnails.roblox.com/v1/users/avatar-headshot` | Public |
| Game media | `https://games.roblox.com/v2/games/<universeId>/media` | Public |
| Asset thumbnails | `https://thumbnails.roblox.com/v1/assets` | Public |
| Auth login | `https://auth.roblox.com/v2/login` | CSRF + session |
| Authenticated user | `https://users.roblox.com/v1/users/authenticated` | Session validation |
| Authentication ticket | `https://auth.roblox.com/v1/authentication-ticket` | Optional direct-join path |

## Libraries / platform docs

| Reference | URL |
|---|---|
| WebKitGTK CookieManager | <https://webkitgtk.org/reference/webkit2gtk/stable/class.CookieManager.html> |
| wry WebView cookies | <https://docs.rs/wry/latest/wry/struct.WebView.html> |
| wry Linux/GTK notes | <https://docs.rs/wry/latest/wry/> |
| egui | <https://docs.rs/egui/latest/egui/> |
| wgpu | <https://docs.rs/wgpu/latest/wgpu/> |
| gilrs | <https://docs.rs/gilrs/latest/gilrs/> |
| rodio | <https://docs.rs/rodio/latest/rodio/> |
| Roblox Xbox menu SFX (local user assets) | <https://github.com/Rawblocky/RobloxXbox-Deck-Soundpack> |

## Usage note

The 2016 captures, Roblox logo, game thumbnails, and soundtrack are property of their respective owners. They are used locally for reference and UI comparison only. Do not commit or redistribute them.
