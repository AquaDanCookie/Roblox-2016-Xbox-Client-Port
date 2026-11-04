# 2016 Xbox UI Reference and Rust Implementation Spec

**Target:** Roblox Xbox One “V1” shell, late 2015 – mid 2017.  
**Design canvas:** 1920×1080, 16:9, 10-foot UI.  
**Reference captures:** 1280×720 frames from the 2016 gameplay capture and Fandom stills.

> **Phase 2 note (2026-11-02):** Sign In opens a dedicated WebKitGTK window with an ephemeral profile. Use Sober session imports the existing cookie read-only. Home, Avatar, Friends, and Robux show real account data when signed in; the 3D avatar is still replaced by a 2D headshot. See [`phase-2-report.md`](phase-2-report.md).
>
> **Phase 4 note (2026-11-02):** the shell loops a user-supplied video in `~/.local/share/roblox-2016-xbox/background/` and a user-supplied track in `~/.local/share/roblox-2016-xbox/music/`. Without those files it uses a generated foggy scene and no music. The app does not bundle the 2016 video or soundtrack. See [`phase-4-report.md`](phase-4-report.md).
>
> **UI/audio pass (2026-11-03):** square 300×300 game icons, measured 2016 nav/typography, unboxed friend activity, 450×290 avatar card, and a gapless in-memory music loop. See [`ui-audio-pass-2026-11-03.md`](ui-audio-pass-2026-11-03.md).

---

## 1. Sources

| Source | Use |
|---|---|
| `/tmp/opencode/refs/xbox2016_title.png` | Title screen composition |
| `/tmp/opencode/refs/k2016/t0.jpg` | Title screen, logo/tagline |
| `/tmp/opencode/refs/k2016/t20.jpg` | Sign-in choice |
| `/tmp/opencode/refs/k2016/t65.jpg` | Home layout |
| `/tmp/opencode/refs/k2016/t85.jpg` | Games carousels |
| `/tmp/opencode/refs/k2016/gd160.jpg` | Game details |
| `/tmp/opencode/refs/k2016/t100.jpg` | Avatar/catalog |
| `/tmp/opencode/refs/xbox2016_pmObfZGsKIE.mkv` | Motion, focus, transitions |
| <https://roblox.fandom.com/wiki/Roblox_(Xbox)> | 2016 title/UI stills |
| <https://www.youtube.com/watch?v=pmObfZGsKIE> | 2016 gameplay capture |

The captures are reference-only. Do not redistribute extracted frames or the soundtrack.

## 2. Design tokens

Colors are sampled/approximated from the captures. The 2016 shell is darker and more desaturated than the modern Roblox UI.

| Token | Value | Use |
|---|---|---|
| `bg_deep` | `#10191e` | Deep background / vignette |
| `bg_mid` | `#202b30` | Mid background |
| `bg_light` | `#2c3b42` | Lifted background shapes |
| `panel` | `rgba(18, 30, 36, 0.72)` | Cards, panels, overlays |
| `panel_solid` | `#1b2a31` | Opaque panels |
| `text_primary` | `#f4f8fa` | Headings, selected nav, values |
| `text_secondary` | `#c3d2d9` | Body text |
| `text_muted` | `#8aa6b2` | Labels, metadata, inactive nav |
| `accent` | `#4aa2d2` | Links, underlines, active icons |
| `accent_bright` | `#8fd3ff` | Hover, secondary focus |
| `focus` | `#d9f7ff` | Focus border |
| `focus_glow` | `rgba(130, 205, 235, 0.55)` | Outer focus glow |
| `play` | `#2aa457` | PLAY button |
| `play_hover` | `#34c46a` | PLAY hover/focus |
| `secondary_button` | `#636b72` | FAVORITE / secondary |
| `price` | `#d9863a` | Robux price tags |
| `danger` | `#cd2d35` | Logo red / destructive |
| `success` | `#52a464` | Positive rating bar |
| `negative` | `#c96a72` | Negative rating bar |

### 2.1 Opacity and blur

- Background is always visible behind content; panels are translucent, not flat black.
- Use a subtle blur/vignette; avoid heavy glassmorphism. The 2016 UI is matte, not glossy.
- Focus glow is soft, not neon. No purple gradients, no rainbow borders.

## 3. Typography

| Role | Face | Size at 1080p | Weight | Case |
|---|---|---:|---|---|
| Logo | wordmark image | 610×163 | — | — |
| Top nav | Source Sans Pro | 47 px | Light | Uppercase |
| Section label | Source Sans Pro | 24 px | Regular | Uppercase, 1.5 px tracking |
| Screen title | Source Sans Pro | 50 px | Light | Sentence case |
| Tagline | Source Sans Pro | 50 px | Light | Sentence case |
| Game title | Source Sans Pro | 28 px | Regular | Sentence case |
| Body | Source Sans Pro | 24 px | Regular | Sentence case |
| Metadata | Source Sans Pro | 17 px | Regular | Sentence case |
| Button | Source Sans Pro | 32 px | Regular | Uppercase |
| Prompt | Source Sans Pro | 26 px | Regular | Sentence case |

- Source Sans Pro is bundled in `assets/fonts/` under the SIL OFL 1.1. It is the 2016 source font.
- Noto Sans and Noto Color Emoji are fallbacks for missing glyphs.
- Do not bundle Segoe UI. It is no longer used as the primary face.
- Avoid faux-condensed or all-bold text. The 2016 UI is thin and airy.

## 4. Layout grid

- Virtual canvas: 1920×1080.
- Safe margin: 96–104 px left/right.
- Top logo: visible bbox ~222×49 at (103, 43).
- Top nav: text centre y≈196, first item x=104, 47 px Source Sans Pro Light, 34 px gaps; selected item white with a 2 px light-cyan underline at y≈210.
- Content top: section labels at y≈252; cards at y≈282–292.
- Bottom prompt row: y≈1002.
- Game tiles: square 300×300 at 1080p; 12 px gaps. Home is 4×2; Games is three 2×2 carousels.
- Focus ring: 3 px light-cyan border with a 1 px faint outer edge; no large glow, no scale.
- Buttons: 232×80 (PLAY) and 220×80 (secondary); square corners; uppercase label.
- Avatar card: 450×290 at (104, 292). Friend activity is not boxed.

### 4.1 Window behavior

- Default 1920×1080 window; the window manager controls fullscreen.
- Scale the virtual canvas uniformly; letterbox if the aspect ratio is not 16:9.
- Minimum window: 1280×720.
- Respect compositor scaling; do not fight KWin.
- Do not ship an in-app fullscreen toggle that can black out a Wayland session.

## 5. Screen specifications

### 5.1 Boot / title

**Reference:** `t0.jpg`, `xbox2016_title.png`.

- Full-screen user-supplied video background; generated foggy scene when no video is installed.
- Centered 2016 wordmark, width ~32% of screen.
- Tagline: “Community Created Gaming.” / “Limitless Possibilities.” in two lines.
- Bottom center: “Press Any Button To Begin” in 24 px white.
- Any controller button, Enter, Space, or click advances.
- Transition: 400 ms fade to sign-in choice or home.

**Asset:** `assets/roblox_wordmark.png` is the transparent wordmark used on the title and sign-in. `XboxClient/img/roblox.png` remains a provided 2016 splash reference, but it is not used as the live title background; the title uses the video or generated fallback. Do not stretch the wordmark.

### 5.2 Sign-in choice

**Reference:** `t20.jpg`.

- Same background as title.
- Logo centered at y≈300.
- Tagline: “Community Created Gaming. Limitless Possibilities.”
- Two buttons stacked/centered:
  - `SIGN IN` — blue fill `accent`, white text, light border; primary focus.
  - `CREATE ACCOUNT` — grey fill, white text; opens the Roblox sign-up page in the WebView.
- The 2016 “SIGN UP USING LOST AFFECT” promotion is not reproduced; use a neutral `CREATE ACCOUNT`.
- If a valid stored session exists, skip this screen and go to Home; offer “Switch account” in Settings.

### 5.3 Login (WebView)

- Dedicated WebKitGTK window, 960×720, titled “Roblox Sign In”.
- Loads `https://www.roblox.com/login`.
- User completes captcha/2FA in the real page.
- On success, capture `.ROBLOSECURITY`, validate, close the window, return to the shell.
- Show a small shell overlay: “Signing in…” with a spinner; on error, show the error and a Retry button.
- Do not render a fake username/password form in the 2016 shell as the primary path.

### 5.4 Home

**Reference:** `t65.jpg`.

- Top-left wordmark; top nav `HOME AVATAR GAMES FRIENDS ROBUX`.
- `HOME` selected: white, underline.
- Left column (450 px):
  - Section label: the signed-in display name at x=100, y≈252 (the capture used `WATRABI`; the old sample text `LOST AFFECT` was not a fixed label).
  - Avatar card: 450×290 at (104, 292), blue gradient and an upper-body avatar crop. No name overlay.
  - Section label `FRIEND ACTIVITY` at y≈622.
  - Friend rows directly on the background: 72 px headshot, 132 px row step, thin separator; 2 rows.
- Right area:
  - Section label `FEATURED` at x=585, y≈252.
  - 4×2 grid of square 300×300 game icons at (585, 292), 12 px gaps; focused card shows a title overlay.
  - `...` overflow button at the bottom-right of the grid.
  - Avatar card focused by default.
- Bottom-right prompt row: hidden in the current pass.
- Loading: skeleton cards in panel color; no fake game titles.
- Empty/error: “Could not load featured games” + Retry.

### 5.5 Games

**Reference:** `t85.jpg`.

- Top nav with `GAMES` selected.
- Three horizontal carousels: `FEATURED`, `POPULAR`, `TOP RATED` (map to modern sorts; labels may be `TOP TRENDING`, `TOP PLAYING NOW`, `UP AND COMING` if the modern sort is more accurate).
- Each carousel: section label, a 2×2 block of 300×300 square tiles with the third column peeking, and a `...` overflow button. Carousel origins are x≈96, 750, and 1404.
- Focus moves left/right within a carousel; up/down between carousels.
- LB/RB switch top-level tabs; LT/RT page the focused carousel.
- `Y` opens Search.
- Card focus: 3 px light-blue border, slight scale, title/player count overlay.

### 5.6 Search

**Adaptation:** the 2016 capture does not show a dedicated search screen. This screen uses the same visual language and is reachable via `Y` or the magnifier in the Games header.

- Top nav remains visible; `GAMES` stays selected.
- Search field: 640×56, dark panel, 2 px accent border when focused, placeholder “Search experiences”.
- On-screen keyboard hint: “Press A to type” / “Type to search”; use the system keyboard on desktop.
- Debounce 350 ms; show a spinner in the field.
- Results: 5-column grid of 250×250 square game tiles; same focus behavior.
- Empty state: “No experiences found for ‘…’.”
- Error state: “Search is unavailable” + Retry.
- `B` returns to Games; `Y` clears/refocuses the field.

### 5.7 Game details

**Reference:** `gd160.jpg`.

- Top-left: `← Home` back button; `B` also returns.
- Title: `<Game Name> - <version/creator>` at x=156, centre y≈180; 50 px light.
- Left: square 540×540 game icon at (156, 278), with a thin border; live image.
- Center column (~430 px):
  - Label `RATING AND DESCRIPTION` at x=753, y≈252.
  - Thumbs up/down icons; segmented green/red rating bar; exact up/down counts.
  - Description text, scrollable; show “Read more” if clipped.
  - Metadata: creator, visits, max players, updated date.
- Right column (432 px):
  - Label `GAME IMAGES` at x=1233, y≈252.
  - One 432×243 image and two 210×118 images from `v2/games/<universeId>/media`.
- Bottom-left:
  - `PLAY` — green `#2aa457`, 232×80 at y≈952; focused by default.
  - `FAVORITE` — grey, 220×80; honest unavailable toast, never a fake success.
- Bottom-right: 96×48 `...` overflow at (1776, 864) — only implement real actions.
- Loading: thumbnail skeleton, metadata skeleton; no fake rating.
- Launch: PLAY opens the loading overlay and hands off to Sober.

### 5.8 Avatar

**Reference:** `t100.jpg`.

- Top nav `AVATAR` selected.
- Left: the signed-in display name and a large full-body avatar render from `thumbnails.roblox.com/v1/users/avatar` with a soft cyan glow; no panel and no interactive 3D character.
- Right: `MY COLLECTION` label and one 180×180 avatar tile at (765, 285); `CATALOG` label and four 180×180 blue catalog tiles.
- If catalog/3D customization is not implemented, show honest silhouette placeholders and a “Catalog and interactive avatar customization are not available in this build.” note; do not show fake prices or items.
- The 2016 interactive 3D avatar cannot be reproduced without the engine/assets; the full-body render is a still-image approximation.

### 5.9 Friends

- Top nav `FRIENDS` selected.
- List of friends with headshot, name, presence (`In game`, `Online`, `Offline`), and current place if available.
- Use `friends.roblox.com` + `presence.roblox.com`.
- Privacy-limited/empty: “No friends to show” or “Friends list is private.”
- No fake activity.

### 5.10 Robux

- Top nav `ROBUX` selected.
- Balance card from `economy.roblox.com/v1/users/<id>/currency` when authenticated.
- If unauthenticated: “Sign in to view your Robux balance.”
- Do not implement purchases or trades in v1.

### 5.11 Settings

- Opened by `X` on Home, `Menu`, or `Esc`.
- Left category list: Account, Audio, Video, Controls, Diagnostics.
- Right panel:
  - Account: current user, Switch account, Sign out.
  - Audio: master volume, music volume, mute on focus loss, UI sounds.
  - Video: fullscreen, UI scale (100/125/150%), reduced motion, high contrast.
  - Controls: controller/keyboard mapping reference; vibration toggle if supported.
  - Diagnostics: app version, API status, Sober status, log folder, copy diagnostics (redacted).
- Sign out: clear keyring, clear WebView data, return to sign-in.

### 5.12 Launch / loading

- Full-screen dim overlay over the details screen.
- 2016-style spinner (thin arc) + “Joining <game>…”.
- Subtext: “Handing off to Sober…”.
- Cancel button (B/Esc) until the handoff starts; after Sober starts, “Cancel” only stops our wait, not Sober.
- On success: “Joined” briefly, then minimize the shell / let Sober take focus.
- On failure: error panel with reason, log path, Retry, Open Sober.
- Never show a fake progress percentage.

## 6. Components

### 6.1 Top nav

- Items: `HOME`, `AVATAR`, `GAMES`, `FRIENDS`, `ROBUX`.
- Idle: `text_muted`; selected: white + 2 px accent underline; hover/focus: `accent_bright`.
- Left/right moves focus; A activates; LB/RB jumps tabs.
- Search is not a top-nav item; use `Y`/magnifier to preserve the 2016 nav.

### 6.2 Game card

- Square game icon, 300×300 in Home/Games, 2 px transparent border.
- Bottom gradient overlay for title/player count.
- Focus: 3 px `focus` border, 8 px glow, scale 1.03.
- Missing thumbnail: panel background + game name initials; no broken image.
- Sponsored cards: small `Sponsored` label; do not hide it.

### 6.3 Buttons

- Primary: `play` green, white uppercase text.
- Secondary: `secondary_button` grey.
- Destructive: `danger` red, used sparingly.
- Focus: 2 px `focus` border + glow; no color inversion.
- Disabled: 40% opacity; explain why on focus.

### 6.4 Focus ring

- 3 px `#d9f7ff` border, 8 px outer glow `rgba(130,205,235,0.55)`.
- Always visible for controller/keyboard; hide for mouse until hover.
- Never rely on color alone; pair with scale/underline.

### 6.5 Controller prompt

- Circled letter (A/B/X/Y) or glyph + label, bottom-right.
- Show only connected input prompts; if no controller, show keyboard equivalents.
- Examples: `A Select`, `B Back`, `X Settings`, `Y Search`.

### 6.6 Dialogs/toasts

- Dark panel, 2 px accent border, centered.
- Title + body + actions.
- No emoji, no exclamation-mark clutter.
- Errors include a retry action and a redacted diagnostic code.

## 7. Input mapping

### 7.1 Controller (gilrs)

| Input | Action |
|---|---|
| Left stick / D-pad | Move focus |
| A (South) | Select / confirm |
| B (East) | Back / cancel |
| X (West) | Context action (Settings on Home, Favorite on Details) |
| Y (North) | Search |
| LB / RB | Previous / next top-level tab |
| LT / RT | Page focused carousel |
| Menu / Start | Settings overlay |
| View / Select | Diagnostics (dev builds only) |
| Guide | Not captured; do not rely on it |

- Deadzone: 0.25; repeat delay 350 ms; repeat rate 90 ms.
- Hotplug: rescan with gilrs; update prompts.
- Vibration: optional; only if the gamepad supports it and the user enables it.

### 7.2 Keyboard

| Input | Action |
|---|---|
| Arrows / WASD | Move focus |
| Enter / Space | Select |
| Esc / Backspace | Back |
| Tab / Shift+Tab | Cycle focus (secondary) |
| Y | Search |
| X | Settings |
| M | Mute |
| F12 | Screenshot |

### 7.3 Mouse

- Hover focuses; click selects.
- Wheel scrolls lists/carousels.
- Right-click / Back button goes back.

## 8. Motion

- Screen transition: 220 ms ease-out fade + 12 px slide.
- Focus: 120 ms scale/border.
- Carousel: 180 ms ease-out; no bounce.
- Background: slow parallax/drift, 0.02–0.05 Hz; no distracting motion.
- Loading spinner: 900 ms rotation, thin arc.
- Reduced motion: disable slides/parallax; keep 100 ms fades.

## 9. Audio

- Menu music: looping user-supplied file.
- Path: `~/.local/share/roblox-2016-xbox/music/menu.mp3` (also `.wav`, `.flac`). OGG Vorbis is not accepted because rodio 0.23 decoded the 2016 theme to zero samples.
- Fade in 1.5 s; fade out 0.8 s on launch; resume on return. Implemented.
- Volume: master + music; default 80%; mute on focus loss is on by default so a foreground game is not interrupted.
- UI sounds: the 2016 source set is installed locally in `~/.local/share/roblox-2016-xbox/sounds/`; generated WAV fallbacks are embedded. The Audio settings tab has a UI sounds toggle, on by default.
- If the music file is missing: no music; Settings shows “Menu music not installed” with “Open music folder.” Do not substitute a copyrighted track.

## 10. Background

- Primary: user-supplied video in `~/.local/share/roblox-2016-xbox/background/`. Accepts `menu.mp4`, `menu.webm`, `menu.mkv`, `menu.mov`, `background.mp4`, or the first video file in that folder.
- Decoded with `ffmpeg` at 960×540, 30 fps, looped, no audio. The app does not download or bundle the 2016 video.
- Fallback: generated 960×540 foggy scene with layered ridges, a building silhouette, bokeh, vignette, and grain. No frames from the YouTube capture are shipped.
- Reduced motion disables video and uses the generated scene.
- The 2016 background was a real-time 3D scene; a video or the generated scene is an approximation, not a reconstruction of the engine scene.

## 11. Assets and licensing

| Asset | Use | License/action |
|---|---|---|
| `assets/roblox_wordmark.png` | Title/sign-in wordmark | Provided tree; Roblox trademark; local use only |
| `assets/fonts/SourceSansPro-*.ttf` | UI typography | SIL OFL 1.1; bundled with `OFL.txt` |
| Full-body avatar render | Avatar card and Avatar screen | Public Roblox thumbnail API; not redistributed |
| `XboxClient/img/roblox.png` | 2016 splash reference | Provided tree; not used as the live background |
| `XboxClient/img/robloxWide.png` | Logo fallback | Same |
| `XboxClient/img/logo208.png` | App icon | Same |
| `XboxClient/img/logo100.png` | Small icon | Same |
| Live game thumbnails | Cards/details | Roblox CDN; display via API; do not redistribute |
| Segoe UI | Local font | Do not redistribute; use system font |
| Noto Sans | Bundled fallback | OFL; include license |
| Selawik | Optional Segoe UI metric-compatible | Confirm license before bundling |
| Menu music | User-supplied | Do not download/bundle the 2016 track |
| Menu background video | User-supplied | Do not download/bundle the 2016 video |
| Generated fallback scene | Code-generated | No copyrighted frames |
| 2016 captures | Reference only | Do not ship |

## 12. Fidelity tiers

### Tier A — must match

- Screen hierarchy and top nav labels.
- 16:9 10-foot layout, safe margins, square game tiles, and the measured 2016 typography.
- Dark blurred background mood.
- Thin white/light-blue typography and uppercase section labels.
- Light-blue focus border/glow.
- Green PLAY, grey secondary, orange price.
- Controller focus behavior and prompts.

### Tier B — approximate

- Exact background scene and bokeh. User-supplied video can be exact; generated fallback is approximate.
- Exact logo scaling/animation.
- Transition timings and easing.
- UI sound effects.
- Avatar presentation (2D instead of 3D).

### Tier C — not reproduced

- In-engine 3D avatar/catalog.
- Exact 2016 catalog content.
- Xbox Live identity/gamertag flow.
- 2016 engine UI inside games.
- Exact 2016 soundtrack.

## 13. Screen acceptance checklist

- [ ] Boot/title matches composition and typography.
- [ ] Sign-in choice has the correct hierarchy and button colors.
- [ ] Home: nav, avatar card, featured grid, friend activity, settings prompt.
- [ ] Games: three carousels, focus, overflow buttons.
- [ ] Search: field, debounce, results grid, empty/error states.
- [ ] Details: back, title, thumbnail, rating, description, media, PLAY/FAVORITE.
- [ ] Settings: account/audio/video/controls/diagnostics.
- [ ] Launch: loading overlay, honest success/failure.
- [ ] Focus visible on every interactive element.
- [ ] Keyboard navigation works end-to-end.
- [ ] Controller navigation verified on hardware (or marked unverified).
- [ ] No fake data, no fake login, no fake launch.
- [ ] No copyrighted music/frames bundled.
