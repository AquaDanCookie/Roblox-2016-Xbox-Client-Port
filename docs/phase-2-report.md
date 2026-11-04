# Phase 2 Report: Authentication and Account Data

**Status:** Complete on 2026-11-02. Phase 3 launch scope was completed later that day; see [`phase-3-report.md`](phase-3-report.md).
**Host:** CachyOS, KDE Wayland, NVIDIA RTX 3060.

---

## Security notice, read first

During verification I ran `secret-tool search service roblox-2016-xbox`, expecting to see metadata. The command printed the stored `.ROBLOSECURITY` secret into the terminal and tool output. That was my mistake.

The app itself never logs the token, and its stored copy has been removed. The token still exists in the Sober cookie file and is still valid on Roblox. Treat that session as exposed. Sign out of Roblox on all devices or change the account password to invalidate it. The steps to re-authenticate afterward are at the end of this report.

Do not copy the token from the terminal transcript into a file, issue, or message.

---

## 1. What was built

### 1.1 Session storage

- `auth.rs` defines `Session`, `SessionStore`, `AuthState`, and `SessionStorage`.
- The token is held in `secrecy::SecretString`. The session type has no `Debug` output.
- Primary storage is the Secret Service through `keyring`. This was verified: the session validated after an app restart with no Sober import.
- If the Secret Service is unavailable, the app writes a `0600` fallback file at `~/.local/share/roblox-2016-xbox/session` and warns. This fallback path was implemented but not exercised because the Secret Service worked.
- Sign-out deletes the keyring entry and the fallback file.

### 1.2 Read-only Sober import

- `auth::read_sober_token` reads `~/.var/app/org.vinegarhq.Sober/data/sober/cookies`, extracts `.ROBLOSECURITY` from the single-line cookie string, and never writes to Sober's data.
- The token is validated with `GET https://users.roblox.com/v1/users/authenticated` before it becomes a session.
- The import is opt-in through the "Use Sober session" button on Sign In and Settings Account. No automatic Sober import happens at startup.

### 1.3 WebKitGTK login helper

- `login_helper.rs` runs as a separate process through the app binary with `--login-helper --out <path>`.
- It opens a GTK 3 window with a WebKitGTK web view at `https://www.roblox.com/login`.
- The WebView uses an **ephemeral** WebKit context. Cookies and site data are discarded when the window closes.
- A one-second poll reads the WebKit cookie manager for `.ROBLOSECURITY`. When found, the helper writes the token to a `0600` file and closes.
- The main app reads the file, deletes it, validates the token, and stores it in the Secret Service.
- The helper receives `WEBKIT_DISABLE_DMABUF_RENDERER=1` to avoid the NVIDIA/WebKitGTK DMA-BUF path. It rendered correctly under Xvfb.
- No developer tools are enabled. The page is not script-injected. Only `.roblox.com` cookies are read.

### 1.4 Authenticated API

Verified with the existing Sober session on 2026-11-02:

| Call | Result |
|---|---|
| `GET users.roblox.com/v1/users/authenticated` | 200, id/name/displayName |
| `GET friends.roblox.com/v1/users/<id>/friends` | 200, 20 friends |
| `POST users.roblox.com/v1/users` with friend IDs | 200, resolved names |
| `POST presence.roblox.com/v1/presence/users` | 200, 20 presence entries, some fields null |
| `GET economy.roblox.com/v1/users/<id>/currency` | 200, `robux` |
| `GET thumbnails.roblox.com/v1/users/avatar-headshot` | 200, Completed image URL |

The friends endpoint returned 20 entries with empty `name` and `displayName` values. The app now resolves those IDs through the public users batch endpoint and fills in the names. Presence entries can have null `lastLocation`, `placeId`, and `universeId`; the model handles nulls.

### 1.5 Account UI

- **Home:** signed-in avatar headshot, display name, username, and two online friends with headshots, presence, and real names. Signed out: the previous guest silhouette and honest empty state.
- **Avatar:** 2D Roblox headshot, display name, username, user ID, and an explicit note that the 2016 3D avatar and catalog are not reproduced.
- **Friends:** up to 20 real friends in two columns with headshots, names, and presence.
- **Robux:** real balance from the economy endpoint, with loading, error, and Retry states. Purchases and trades are not implemented.
- **Settings Account:** signed-in account, storage location, sign-out, and signed-out Sign In / Use Sober session buttons.
- **Sign In:** real WebView login, read-only Sober import, guest browsing, and Create Account.
- **Auto-restore:** a stored Secret Service session validates at startup and goes to Home. Verified on this machine.
- **Sign-out:** verified with the demo sequence. The screen returned to Sign In and the keyring entry was deleted.

No fake account data was added. Signed-out screens keep honest empty states.

---

## 2. Verification

### 2.1 Build and tests

- `cargo check`: clean, no warnings.
- `cargo test`: 8 passed after adding the number-format test.
- `cargo build`: success.
- `cargo build --release`: success, 34 MB binary. The release binary renders the sign-in screen with the same layout.

### 2.2 Live account flow

The app imported the Sober session and logged:

```text
session validated for the account
user headshots loaded: 1
friends loaded: 20
robux balance loaded
user headshots loaded: 19
presence entries loaded: 20
```

The signed-in Home, Avatar, Friends, Robux, and Settings Account screens rendered with real data. Those screenshots are in `/tmp/opencode/phase2-account/` for local review only. They contain the account and friends and are not part of the repository.

### 2.3 Auto-restore

After importing once and storing the session in the Secret Service, starting the app on the Sign In screen without `--import-sober` validated the stored session and moved to signed-in Home. The keyring entry was then removed with the sign-out demo.

### 2.4 Sign-out

```text
screen -> Settings
demo action: Navigate(Right)
demo action: Confirm
focus activate settings:account-signout (click=false, confirm=true)
screen -> SignIn
```

The keyring lookup after sign-out returned no entry.

### 2.5 WebView login page

The helper rendered the real Roblox login page in an isolated Xvfb display. The screenshot is `docs/screenshots/webview-login.png`. The page is the official login form, not a replica.

The actual credential login was not completed by me because I do not have the account credentials. The cookie capture code is implemented and compiles. The Sober import path is the verified real-account login for this phase.

---

## 3. Screenshots

Repository screenshots updated:

- `docs/screenshots/signin.png`: guest sign-in with the Sober import button.
- `docs/screenshots/settings-account.png`: signed-out Account tab with Sign In and Use Sober session.
- `docs/screenshots/webview-login.png`: the real Roblox login page in the helper.
- `docs/screenshots/compare/signin.png`: updated 2016 comparison.

Local-only account screenshots:

- `/tmp/opencode/phase2-account/home.png`
- `/tmp/opencode/phase2-account/avatar.png`
- `/tmp/opencode/phase2-account/friends.png`
- `/tmp/opencode/phase2-account/robux.png`
- `/tmp/opencode/phase2-account/settings-account.png`

These contain the account name, user ID, avatar, and friends. Do not redistribute them.

---

## 4. Security and privacy

- The app never logs the token. It does not appear in the UI, errors, or diagnostics.
- The token is stored in the Secret Service. The fallback file is `0600`.
- The WebView uses an ephemeral context and no developer tools.
- The helper token file is `0600`, read once, and deleted. Stale `login-*.token` files are cleaned before a new login.
- Sign-out clears the keyring and the fallback file.
- The token exposure described at the top came from a diagnostic command, not from the app. The app's stored copy is gone; the Sober cookie still holds the exposed token. Rotate it.

---

## 5. Known gaps

- **WebView login end-to-end:** the login page renders, but I did not complete a credential login. The user must complete it or rotate and re-import.
- **WebView navigation policy:** the helper does not enforce a domain allowlist. It uses an ephemeral context, no developer tools, and only reads `.roblox.com` cookies. A strict navigation policy is a Phase 4 hardening item.
- **3D avatar:** the app shows the 2D headshot. The 2016 3D avatar and catalog remain unavailable.
- **Favorite action:** the details Favorite button still shows a Phase 2 message. The toggle endpoint was not verified, so it was not implemented. This is outside the phase's core acceptance.
- **Friends pagination:** the app shows up to 20 friends from the first page. Larger lists are not paginated.
- **Presence labels:** the API returns "Website" for some online friends. That is the real value and is shown as-is.
- **Secret Service fallback:** the `0600` file path was not exercised because the Secret Service worked.
- **Controller, window size, fullscreen:** unchanged from Phase 1. No controller hardware; KWin keeps the window at 1280x720 logical; the in-app fullscreen toggle was removed.
- **Token exposure:** see the security notice. Rotate the session.

---

## 6. Re-authenticate after rotating

1. Rotate the Roblox session (sign out of all sessions or change the password).
2. Sign back in to Sober, or use the app's Sign In button and complete the real Roblox login page.
3. If using Sober, click Use Sober session in the app. The import is read-only.
4. The app validates the token and stores its own copy in the Secret Service.

---

## 7. Next phase

Phase 3, pending approval:

- Test the Sober launch URI forms in order:
  1. `roblox://experiences/start?placeId=<id>`
  2. `roblox://placeId=<id>`
  3. `roblox://experience?placeId=<id>`
  4. `xdg-open 'roblox://...'`
- Hand the place ID from the 2016 details screen to Sober.
- Verify the join from new `GameJoinLoadTime` and `onGameLoaded` log lines, not a timer.
- Keep Cordial as a secondary runtime and launch fallback.
- Implement loading, cancel, failure, and return-to-shell states.
- Then the fidelity, audio, and polish pass.
