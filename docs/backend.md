# Backend, Auth, and Launch Contract

**Status:** Phase 0 design document. No backend code has been written.  
**Date:** 2026-10-28  
**Target:** Android-compatible Roblox web APIs + Sober Flatpak runtime on CachyOS.

---

## 1. Decision

The 2016 Xbox Live backend is dead. A literal port is impossible.

The reimplementation will use:

1. **Modern Roblox web APIs** for public catalog data and account identity.
2. **A real Roblox login page in WebKitGTK** for authentication, so captcha and 2FA are handled by Roblox rather than reimplemented.
3. **Sober’s official Android runtime** for game launch, via the verified `roblox://experiences/start?placeId=<id>` deep link.
4. **A read-only Sober session import** as a fallback for users who already have a working Sober login.

The app will never claim a launch succeeded unless Sober produces runtime evidence of a join.

## 2. Evidence summary

| Claim | Evidence |
|---|---|
| 2016 API host is gone | `api.roblox.com` returns DNS failure (`curl: (6) Could not resolve host`) |
| 2016 join URL is gone | `www.roblox.com/Game/PlaceLauncher.ashx?...` returns HTTP 404 HTML |
| Modern public APIs work | search/sorts/games/votes/favorites/thumbnails all returned HTTP 200 |
| Auth requires CSRF | `POST auth.roblox.com/v2/login` returned 403 + `x-csrf-token` |
| Sober is registered for `roblox://` | Flatpak desktop file + `xdg-mime query default` |
| Sober can join real games | Sober logs show `GameJoinLoadTime` and `onGameLoaded` for live places |
| Sober APK supports the URI | DEX strings include `roblox://placeId=%d&callId=%s` |
| Sober has a session | Cookie jar contains `GuestData`, `.ROBLOSECURITY`, `RBXEventTrackerV2` (values redacted) |

## 3. Session and authentication architecture

### 3.1 Primary: WebKitGTK login window

Roblox login can require captcha and 2FA. Reimplementing those flows is fragile and hostile to account security. The primary flow is therefore a real Roblox web page:

1. User selects **Sign In**.
2. The app opens a dedicated WebKitGTK window (`wry`) at `https://www.roblox.com/login`.
3. User completes username/password, captcha, and 2FA in the real page.
4. The app polls `WebView::cookies()` / `cookies_for_url("https://www.roblox.com")`.
5. On finding `.ROBLOSECURITY`, the app validates it with `GET https://users.roblox.com/v1/users/authenticated`.
6. The token is stored in the Secret Service via `keyring`; the WebView data directory is cleared on sign-out.

`wry` on Linux uses WebKitGTK and exposes cookie access. WebKitGTK 2.54.2 and GTK 3.24.52 are installed on this machine.

**Why a separate window:** the main app uses `winit` + `wgpu` + `egui`; GTK/WebKit needs a GTK main loop. A dedicated login window (or helper process) avoids mixing event loops and keeps the 2016 shell clean.

**Security requirements:**

- Only navigate to `https://www.roblox.com/*` and `https://*.roblox.com/*`.
- Do not enable remote debugging in release builds.
- Do not log URLs containing tokens, cookies, or query secrets.
- Use a dedicated WebKit data directory, not the user’s browser profile.
- Clear cookies/local storage on sign-out.
- Never store the password; the WebView handles it.
- Never expose `.ROBLOSECURITY` to JavaScript.

### 3.2 Fallback: read-only Sober session import

Sober already has a working authenticated Android session on this machine. The app may offer:

> **Use Sober session** — read the existing `.ROBLOSECURITY` from Sober’s cookie jar and use it for this app.

Requirements:

- Explicit user consent; explain that the token is read from Sober’s data directory.
- Read-only by default; do not write to Sober’s cookie jar.
- Validate the token with `users/authenticated` before use.
- Show the account name so the user can confirm it is the right account.
- Never log or display the token.
- If the token is invalid/expired, report that and fall back to WebView login.

Sober cookie path:

```text
~/.var/app/org.vinegarhq.Sober/data/sober/cookies
```

Format observed: a single-line semicolon-delimited string, e.g. `GuestData=...; .ROBLOSECURITY=...; RBXEventTrackerV2=...`. Values must be redacted in all logs and docs.

### 3.3 Native login API (reference only)

A native form can use the web login endpoint, but it is not the primary flow:

```http
POST https://auth.roblox.com/v2/login
Content-Type: application/json
Origin: https://www.roblox.com
Referer: https://www.roblox.com/
User-Agent: rbx2016/0.1 (Linux; Wayland)

{"ctype":"Username","cvalue":"<username>","password":"<password>"}
```

First request:

```http
HTTP/1.1 403 Forbidden
x-csrf-token: <token>
```

Retry with:

```http
x-csrf-token: <token>
```

On success, Roblox sets `.ROBLOSECURITY` (HttpOnly, Secure). If 2FA is enabled, the response contains a challenge; the exact schema is intentionally not hardcoded here because the WebView flow handles it. Do not build a password form that silently fails on captcha/2FA.

### 3.4 CSRF handling

- Any authenticated `POST` may return 403 with `x-csrf-token`.
- Store the token in memory for the session.
- Retry the request once with the header.
- Never persist the CSRF token.
- Do not log the token.

### 3.5 Session validation

```http
GET https://users.roblox.com/v1/users/authenticated
Cookie: .ROBLOSECURITY=<token>
```

Expected success (shape):

```json
{
  "id": 123456789,
  "name": "Username",
  "displayName": "DisplayName",
  "created": "2016-01-01T00:00:00.000Z"
}
```

- 401 → session invalid; clear stored token and show sign-in.
- Network error → keep token but show offline state; do not silently sign out.
- Validate on startup and after login.

### 3.6 Authentication ticket (optional, not used for launch)

For a future direct-join implementation, the game auth ticket is:

```http
POST https://auth.roblox.com/v1/authentication-ticket
Cookie: .ROBLOSECURITY=<token>
x-csrf-token: <token>
RBX-For-Gameauth: true
```

The ticket is expected in the `rbx-authentication-ticket` response header; this was not exercised with a real session in Phase 0. Unauthenticated probes return 401. This is **not** required for the Sober launch handoff and is documented only to keep the boundary clear.

### 3.7 Storage

| Item | Storage |
|---|---|
| `.ROBLOSECURITY` | Secret Service (`keyring`) service `roblox-2016-xbox`, account `session` |
| Fallback if Secret Service unavailable | File in `~/.local/share/roblox-2016-xbox/session` with mode `0600`; warn the user |
| Password | Never stored |
| CSRF token | Memory only |
| User profile | Memory + optional non-sensitive cache |
| Config | `~/.config/roblox-2016-xbox/config.toml` |
| Thumbnail cache | `~/.cache/roblox-2016-xbox/thumbnails/` |
| Logs | `~/.local/state/roblox-2016-xbox/logs/` |

Use `zeroize`/`secrecy` for token buffers where practical. Redact `Cookie`, `x-csrf-token`, `rbx-authentication-ticket`, and `.ROBLOSECURITY` in logs.

## 4. Public data APIs

### 4.1 Common parameters

- `sessionId`: UUID v4 generated per app run; some endpoints require it. Do not use a user identifier.
- `device=console`: closest to the 2016 Xbox curated catalog.
- `country=all`: avoid geo-specific omissions; the UI can later add a country setting.
- `User-Agent`: descriptive app UA; do not impersonate the Android client.
- Timeouts: 15 s connect/read; 2 retries with backoff on 429/5xx.
- Cache: sorts 5 min; game details 2 min; search 30 s; thumbnails 24 h.

### 4.2 Endpoint table

| Purpose | Method | URL | Auth | Notes |
|---|---|---|---|---|
| Search | GET | `https://apis.roblox.com/search-api/omni-search?searchQuery=<q>&sessionId=<uuid>&pageToken=&pageType=all` | No | `searchResults[].contents[]` with `universeId`, `rootPlaceId`, `name`, `playerCount`, votes, creator |
| Home sorts | GET | `https://apis.roblox.com/explore-api/v1/get-sorts?sessionId=<uuid>&device=console&country=all` | No | Returns `sorts` incl. `filters_v5`, `top-trending`, `up-and-coming`, `top-playing-now`, `fun-with-friends`, `top-revisited` |
| Sort content | GET | `https://apis.roblox.com/explore-api/v1/get-sort-content?sessionId=<uuid>&sortId=<id>&device=console` | No | Returns `games[]` with `universeId`, `rootPlaceId`, `name`, `playerCount`, votes |
| Game details | GET | `https://games.roblox.com/v1/games?universeIds=<id[,id...]>` | No | Batch endpoint; keep batches small and verify the limit at implementation time. Returns name, description, creator, playing, visits, maxPlayers, genre, `rootPlaceId`, `favoritedCount` |
| Place → universe | GET | `https://apis.roblox.com/universes/v1/places/<placeId>/universe` | No | `{"universeId":65241}`; needed when only a place ID is known |
| Votes | GET | `https://games.roblox.com/v1/games/votes?universeIds=<id>` | No | `upVotes`, `downVotes` |
| Favorites | GET | `https://games.roblox.com/v1/games/<universeId>/favorites/count` | No | `favoritesCount`; also present in game details |
| Game thumbnails | GET | `https://thumbnails.roblox.com/v1/games/multiget/thumbnails?universeIds=<ids>&size=768x432&format=Png&isCircular=false` | No | `thumbnails[].imageUrl` on `tr.rbxcdn.com` |
| Game icons | GET | `https://thumbnails.roblox.com/v1/games/icons?universeIds=<ids>&size=256x256&format=Png&isCircular=false` | No | Square icons |
| Game media | GET | `https://games.roblox.com/v2/games/<universeId>/media` | No | Returns image asset IDs for the details screen |
| Asset thumbnails | GET | `https://thumbnails.roblox.com/v1/assets?assetIds=<ids>&size=768x432&format=Png` | No | Render media asset IDs as images |
| Avatar headshot | GET | `https://thumbnails.roblox.com/v1/users/avatar-headshot?userIds=<ids>&size=150x150&format=Png&isCircular=false` | No | Home/nav avatar |
| Avatar data | GET | `https://avatar.roblox.com/v1/users/<userId>/avatar` | No | Avatar scales/colors; 2D only in this client |
| Friends | GET | `https://friends.roblox.com/v1/users/<userId>/friends` | No | May be privacy-limited; empty is valid |
| Presence | POST | `https://presence.roblox.com/v1/presence/users` body `{"userIds":[...]}` | No | Friend activity |
| Robux balance | GET | `https://economy.roblox.com/v1/users/<userId>/currency` | Yes | 401 without auth |
| Authenticated user | GET | `https://users.roblox.com/v1/users/authenticated` | Yes | Session validation |
| Public servers | GET | `https://games.roblox.com/v1/games/<placeId>/servers/Public?limit=10` | No | Optional server browser; not required for launch |

### 4.3 Place vs universe IDs

- Search and sort content return both `universeId` and `rootPlaceId`.
- Details are keyed by `universeId`.
- Launch is keyed by `rootPlaceId` (the place ID Sober expects).
- If only a place ID is available, resolve it with the place→universe endpoint, then fetch details.
- Do not pass a universe ID to Sober as a place ID.

### 4.4 Home layout mapping

The 2016 home has a featured grid and friend activity. Modern data can fill it honestly:

- `FEATURED` grid: first 8 games from `top-trending` (or a blended set from `top-playing-now`).
- `Games` carousels: `top-trending`, `top-playing-now`, `up-and-coming`, `fun-with-friends`, `top-revisited`.
- `Friend activity`: authenticated friends + presence; if unavailable, show an honest empty state.
- `Avatar`: headshot + display name; the 3D avatar is not reproduced.
- `Robux`: authenticated currency balance; if unavailable, show an honest error.

### 4.5 Error handling

- 401: clear session only after a validation request; show sign-in.
- 403 with `x-csrf-token`: refresh token and retry once.
- 429: backoff and show “Roblox is rate-limiting; retry shortly.”
- 5xx: retry twice, then show a retry button.
- Network/DNS: show offline state; keep cached data if available; do not fabricate.
- Invalid/empty results: show “No results” not a fake list.
- Thumbnail failure: show a neutral placeholder with the game name; no broken-image icon.

## 5. Launch handoff

### 5.1 Why Sober

Sober runs the official Roblox Android client in a Flatpak sandbox on Linux. It already has a working authenticated session and has joined live experiences on this machine. Direct in-process game rendering is out of scope.

### 5.2 URI

Use the place ID, not the universe ID. The form verified on this machine on 2026-11-02 is:

```text
roblox://experiences/start?placeId=<rootPlaceId>
```

APK evidence also lists:

```text
roblox://placeId=%d&callId=%s
roblox://placeId=%d&gameInstanceId=%s&callId=%s
roblox://placeId=%d&reservedServerAccessCode=%s&callId=%s
roblox://experiences/start?placeId=<id>
```

The plain `roblox://placeId=<id>` form was not needed after the verified form worked. Keep it as a documented fallback, not the primary.

### 5.3 Command

Verified command:

```bash
flatpak run org.vinegarhq.Sober 'roblox://experiences/start?placeId=<rootPlaceId>'
```

Cordial fallback (registered handler, not exercised because Cordial is not installed):

```bash
xdg-open 'roblox://experiences/start?placeId=<rootPlaceId>'
```

Implementation requirements:

- Pass the URI as a separate argument; never use a shell string.
- Validate that the place ID is numeric.
- Do not pass user-controlled text into the URI.
- Capture stdout/stderr for diagnostics; redact tokens.
- If `flatpak` is missing, show an actionable error.
- If Sober is not installed, show the install command; do not fake a launch.
- If neither Sober nor Cordial is installed, refuse to launch and say so.

### 5.4 Session alignment

Sober joins with **its own** cookie jar, not our app’s session. Before launch:

1. Read Sober’s cookie jar (names only, then validate `.ROBLOSECURITY`).
2. Resolve the Sober account with `users/authenticated`.
3. Compare with our app’s account.
4. If same → launch.
5. If different → warn: “Sober is signed in as X; launching will join as X.” Offer to cancel or continue.
6. If Sober has no valid session → warn: “Sober is not signed in. Open Sober and sign in, then retry.” Offer an “Open Sober” button.
7. If the cookie jar is unreadable → warn; do not guess.

This prevents the confusing case where the shell shows one account but the game joins another.

### 5.5 Launch verification

A launch is successful only when runtime evidence confirms it:

1. Record the current size/offset of `~/.var/app/org.vinegarhq.Sober/data/sober/sober_logs/latest.log` (or the newest log).
2. Start Sober with the URI.
3. Poll for up to 90 s for new lines containing:
   - `GameJoinLoadTime` with `placeid:<rootPlaceId>`, or
   - `onGameLoaded()` with `placeId:<rootPlaceId>`.
4. Treat `Roblox has crashed`, `out of memory`, `Invalid device`, and `RBXCRASH` as launch failures.
5. On match: show “Joined” with a return-to-shell button.
6. On timeout or crash: show “Launch failed”, include the log path, and offer Retry / Open Sober / Back.
7. On process error: show the error; do not claim success.

The log format is proprietary and may change. The detector must be isolated in `backend/launch.rs` and covered by tests with synthetic log lines.

### 5.6 Loading and return

- Show a 2016-style loading overlay while the handoff is in progress.
- If Sober takes focus, the shell can minimize.
- When Sober exits, restore the shell (best effort on Wayland).
- If the user cancels, kill only the process we started if it is still ours; do not kill an existing Sober session.

## 6. 2016 vs modern comparison

| Concern | 2016 Xbox client | 2026 reality | Reimplementation |
|---|---|---|---|
| Identity | Xbox Live user/gamertag | Roblox web account | WebView login or Sober session |
| Auth | `xbl-authz-actor-10` → `api.roblox.com/xboxlive/connect` | `auth.roblox.com/v2/login` + `.ROBLOSECURITY` | WebView + cookie validation |
| User info | `/xboxlive/get-roblox-userInfo` | `users.roblox.com/v1/users/authenticated` | Modern endpoint |
| Catalog | `api.roblox.com` Xbox endpoints | `apis.roblox.com`, `games.roblox.com` | Modern endpoints |
| Search | 2016 catalog/search | `search-api/omni-search` | Modern endpoint |
| Join | `PlaceLauncher.ashx` + `gamerTag` | Android join via Sober; `assetgame.roblox.com/Game/Join.ashx` observed | `roblox://placeId=` → Sober |
| Protocol activation | `ActivationKind::Protocol` | `x-scheme-handler/roblox` | `flatpak run ...` / `xdg-open` |
| Controller | `Windows::Xbox::Input` | Linux evdev/gilrs | `gilrs` |
| Rendering | In-process engine | Official Android runtime | Sober |
| UI assets | 2016 scenes/XAML | Not in tree | Reconstructed with egui |
| Music | 2016 soundtrack | Copyrighted, not in tree | User-supplied file |
| Background | Blurred 3D scene | Not in tree | Procedural shader; optional user image |

## 7. Security and privacy

### 7.1 Token handling

- `.ROBLOSECURITY` is equivalent to a password. Treat it as a secret.
- Never print it, log it, put it in a URL, or include it in error messages.
- Redact `Cookie`, `Set-Cookie`, `x-csrf-token`, `rbx-authentication-ticket`, and any `_|WARNING` token prefix.
- Store in Secret Service; fallback file must be `0600` and the UI must warn.
- Clear on sign-out; clear WebView cookies/local storage.
- Do not sync tokens to cloud storage or telemetry.

### 7.2 WebView

- Restrict navigation to Roblox domains.
- Do not enable developer tools in release.
- Use a dedicated data directory.
- Do not inject scripts that read HttpOnly cookies.
- Use the WebKit cookie manager to read `.ROBLOSECURITY`.
- Close and destroy the WebView after login.

### 7.3 Subprocess

- Use `Command::new("flatpak").arg("run").arg("org.vinegarhq.Sober").arg(uri)`.
- Never use `sh -c`.
- Validate place IDs as `u64`.
- Do not pass account names, tokens, or free-form text to Sober.
- Do not kill processes we did not start.

### 7.4 Network

- TLS verification on; no custom CA by default.
- No proxy by default; respect `HTTPS_PROXY` only if the user configures it.
- No telemetry.
- Cache only public thumbnails/metadata; never cache tokens in the thumbnail cache.
- Rate-limit and backoff.

### 7.5 Legal

- This is an unofficial fan reimplementation; it is not affiliated with Roblox.
- Roblox’s Terms of Use may restrict modified clients; the user assumes the account risk.
- Do not bypass anti-cheat, DRM, or security controls.
- Do not redistribute Roblox assets, the 2016 soundtrack, or extracted captures.
- Code can be licensed separately; bundled third-party assets keep their licenses.

## 8. Open questions

1. **Login source:** WebView (recommended), read-only Sober import (fallback), or both?
2. **Session sync:** read-only only, or explicit write-back to Sober? Write-back needs a separate security review.
3. **Launch URI:** test `roblox://placeId=<id>` first; fallback forms only if needed.
4. **Music:** user-supplied file vs licensed replacement.
5. **Sober detection:** log tailing is the current plan; is there a cleaner IPC/DBus signal? Sober is proprietary; do not depend on undocumented internals beyond logs.

## 9. Phase 2 acceptance tests

| # | Test | Pass condition |
|---|---|---|
| 1 | WebView login | Real account signs in on this machine; `users/authenticated` returns it |
| 2 | Token storage | Token is in Secret Service; not in logs; not in project files |
| 3 | Restart | App restores session and shows the same account |
| 4 | Search | “Natural Disaster Survival” returns live results |
| 5 | Details | Live metadata, votes, favorites, thumbnail, media load |
| 6 | Launch | PLAY hands `roblox://placeId=189707` to Sober; Sober joins; log confirms `placeid:189707` |
| 7 | Sober signed out | App reports it; no false success |
| 8 | Different Sober account | App warns before launch |
| 9 | Sign out | Keyring entry removed; WebView data cleared; sign-in screen shown |
| 10 | No network | Honest offline/error states; no fake data |
| 11 | Token leakage scan | `grep` logs for token prefix returns nothing |
| 12 | Controller | If hardware present, full navigation; otherwise marked unverified |

## 10. References

- 2016 source: `XboxClient/XboxService.cpp`, `XboxClient/main.cpp`, `XboxClient/Package.appxmanifest`.
- Sober desktop file: `/var/lib/flatpak/exports/share/applications/org.vinegarhq.Sober.desktop`.
- Sober APK: `~/.var/app/org.vinegarhq.Sober/data/sober/packages/x86_64/com.roblox.client/base.apk`.
- Sober logs: `~/.var/app/org.vinegarhq.Sober/data/sober/sober_logs/`.
- Roblox API probes: see `docs/phase-0-recon.md`.
- WebKitGTK cookie manager: <https://webkitgtk.org/reference/webkit2gtk/stable/class.CookieManager.html>
- wry WebView cookies: <https://docs.rs/wry/latest/wry/struct.WebView.html>
- 2016 UI reference: <https://roblox.fandom.com/wiki/Roblox_(Xbox)>
- 2016 gameplay capture: <https://www.youtube.com/watch?v=pmObfZGsKIE>

## 11. Addendum, 2026-11-02

This section corrects the launch URI plan and records the Android identity evidence found during re-verification. The rest of this document still describes the intended architecture.

### 11.1 Android-compatible identity

Sober runs the official Roblox Android APK. The installed build reports:

- package `com.roblox.client`
- app version `2.741.1061`
- ABI `x86_64`
- base APK at `~/.var/app/org.vinegarhq.Sober/data/sober/packages/x86_64/com.roblox.client/base.apk`

Cordial's ground-truth trace of the same APK records:

- User-Agent: `... ROBLOX Android App 2.732.1043 Tablet Hybrid GooglePlayStore RobloxApp/2.732.1043`
- `OS Ver. = 13, Lvl = 33`
- `Build = googleProdRelease`
- `Arch = X86_64`
- `BaseUrl = www.roblox.com/`

The Android client therefore presents **Android 13, API level 33**, as a tablet hybrid on Google Play. Sober's exact User-Agent is not logged, so the version token is expected to reflect the installed `2.741.1061` build, but that is an inference. The Roblox web endpoints do not select behavior through an API-version header; they are path-versioned (`/v1`, `/v2`). The shell should use a truthful user agent of its own.

Sources:
- <https://github.com/luohoa97/cordial/blob/main/docs/traces/README.md>
- <https://github.com/luohoa97/cordial/blob/main/native/android_classes.cpp>

### 11.2 Launch URI correction

Do not treat `roblox://placeId=<id>` as the only form. The APK's own strings include:

```text
roblox://placeId=%d&callId=%s
roblox://placeId=%d&gameInstanceId=%s&callId=%s
roblox://placeId=%d&reservedServerAccessCode=%s&callId=%s
roblox://experiences/start
roblox://navigation/game_details
```

Cordial measured this form reaching the engine's `Game.launch` message:

```text
roblox://experiences/start?placeId=1818
```

Community Sober documentation lists `roblox://experience?placeId=<place-id>` and a `placelauncherurl:` form. The exact CLI handoff has not been tested on this machine. Test order for Phase 3:

1. `flatpak run org.vinegarhq.Sober 'roblox://experiences/start?placeId=<id>'`
2. `flatpak run org.vinegarhq.Sober 'roblox://placeId=<id>'`
3. `flatpak run org.vinegarhq.Sober 'roblox://experience?placeId=<id>'`
4. `xdg-open 'roblox://...'` as the registered-handler fallback

A launch is confirmed only by new Sober log lines containing `FLog::GameJoinLoadTime` and `onGameLoaded` for the requested place ID.

### 11.3 Join ticket boundary

A live Sober log shows the runtime using:

```text
https://assetgame.roblox.com/Game/Join.ashx?ticketVersion=2&ticket=<redacted>
```

Sober mints and redeems that ticket with its own session. The shell does not mint it and cannot pass its own session to Sober through the simple place URI. The shell must validate Sober's account separately and warn if it differs from the shell's account.

### 11.4 Cordial as a second runtime

Cordial is an open-source Roblox Android runtime for Linux:

- <https://github.com/luohoa97/cordial>
- Flatpak ID `io.github.luohoa97.Cordial`
- same official Android APK, loaded natively
- registered for `roblox://` and `roblox-player://`
- CLI join path `cordial-run --join-url 'roblox://experiences/start?placeId=<id>'`
- its engine network stack is untouched; it talks to Roblox directly

Cordial is not installed on this machine. It is a credible fallback or secondary runtime and the best public evidence for the Android launch and auth contract. It does not change the primary plan, which is a Sober handoff.

### 11.5 Sandbox handoff limits

Sober's Flatpak permissions are `shared=ipc;network;`, sockets `fallback-x11;pulseaudio;wayland;`, device `dri`, and no host home filesystem. The shell can pass one URI as an argv argument. It cannot pass environment variables, file descriptors, host files, or an auth cookie into the sandbox. Focus restoration after Sober exits is best-effort under Wayland.

## 12. Phase 2 implementation notes

The authenticated path was exercised with the existing Sober session on 2026-11-02.

- `GET https://users.roblox.com/v1/users/authenticated` returned the real account.
- `GET https://friends.roblox.com/v1/users/<id>/friends` returned 20 entries, but every `name` and `displayName` was empty. The app resolves those IDs through `POST https://users.roblox.com/v1/users` with `userIds` and `excludeBannedUsers`. That batch returned real names.
- `POST https://presence.roblox.com/v1/presence/users` returned 20 entries. `lastLocation`, `placeId`, and `universeId` can be null, so the response model uses optional fields.
- `GET https://economy.roblox.com/v1/users/<id>/currency` returned the Robux balance.
- `GET https://thumbnails.roblox.com/v1/users/avatar-headshot` returned a Completed 2D headshot URL.

The login helper uses an ephemeral WebKitGTK context, so no WebView cookies or site data persist after the window closes. It writes the token to a `0600` file, and the main app reads and deletes that file before storing the token in the Secret Service. The fallback session file is also `0600`.
