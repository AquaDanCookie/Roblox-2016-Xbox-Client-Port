# 2016 Fidelity Pass

> **2026-11-03 follow-up:** square game icons, measured 2016 typography, the unboxed Home layout, and the gapless audio loop are documented in [`ui-audio-pass-2026-11-03.md`](ui-audio-pass-2026-11-03.md). The measurements below describe the earlier 2026-11-02 pass.

**Status:** Complete on this machine, 2026-11-02.
**Goal:** Make the shell match the 2016 Xbox One capture as closely as the available assets allow.

---

## 1. Authentic assets found and used

### 1.1 Background animation

The 2016 shell loads its intro scene from a Roblox place. The source constant is in `XboxClient/XboxService.cpp`:

```cpp
const char* kIntroPlace = "rbxasset://ScaledWorldv4.7.rbxl";
// older/commented-out: "rbxasset://particles7.rbxl";
```

`ScaledWorldv4.7.rbxl` is not in the provided tree, and it is not in the upstream 2016 source content directory either. The provided tree contains only the older `XboxClient/Durango/Layout/Image/Loose/particles7.rbxl` (13 KB, a particle test place).

The closest faithful substitute is the public 2016 Xbox background animation recording:

- Title: “Roblox's Old Xbox One UI Background Animation (HQ) (4K)” by LuaWiz
- Source: <https://www.youtube.com/watch?v=ZRolvyk3nj8>
- Installed locally at: `~/.local/share/roblox-2016-xbox/background/menu.mp4`
- Local encoding: 960×540, 30 fps, H.264, 180 seconds, no audio, 4.9 MB.
- The recording contains the actual 2016 space-station, satellite, and fog-mountain shots.

The file is **not** in this repository and is not redistributed with the app. It is a local user asset.

### 1.2 Menu theme

The 2016 menu theme was recorded publicly. The local file is:

- Title: “Roblox Main Menu Theme Song | Xbox One”
- Source: <https://www.youtube.com/watch?v=m80z7ZMtpKU>
- Installed locally at: `~/.local/share/roblox-2016-xbox/music/menu.mp3`
- Local encoding: MP3, 48 kHz stereo, 187 seconds, 4.0 MB, normalized to −16 LUFS.
- The first OGG version decoded to zero samples under rodio 0.23 despite `ffprobe` confirming valid Vorbis audio. The MP3 conversion is the verified playable source.
- The app loops it, fades it in over 1.5 s, fades it out over 0.8 s on launch, and resumes on return.

The file is **not** in this repository and is not redistributed with the app. It is a local user asset.

### 1.3 Source Sans Pro

The 2016 source contains the engine font files in `content/fonts`:

- `SourceSansPro-Regular.ttf`
- `SourceSansPro-Light.ttf`
- `SourceSansPro-Bold.ttf`
- `SourceSansPro-It.ttf`

They are copied into `assets/fonts/` and bundled. Source Sans Pro is licensed under the SIL Open Font License 1.1; the license text is in `assets/fonts/OFL.txt`. The app now logs `font family: Source Sans Pro`.

### 1.4 Full-body avatar

The 2016 shell showed a full-body avatar, not a headshot. The app now uses:

```text
https://thumbnails.roblox.com/v1/users/avatar?userIds=<id>&size=420x420&format=Png&isCircular=false
```

The Home card uses an upper-body crop of the full-body render, matching the landscape 2016 avatar card. The Avatar tab still shows the full render. It is a still image, not an interactive 3D character. The previous headshot remains the fallback.

### 1.5 Layout measurements

The title and sign-in positions were measured against `t0.jpg` / `t20.jpg`:

| Element | 2016 capture (canvas 1920×1080) | Shell |
|---|---|---|
| Logo bbox | x 664–1254, y 418–556 | 610×163 wordmark centered at (960, 488) |
| Tagline line 1 | visible rows ~590–606 | 50 px, rect y 592 |
| Tagline line 2 | visible rows ~638–660 | 50 px, rect y 655 |
| Title prompt | bottom center ~838 | 34 px, rect y 820 |
| Sign-in create | ~755 | 352×52, center 756 |
| Sign-in button | ~845 | 220×52, center 862 |

Home proportions from `t65.jpg`:

| Element | 2016 capture (canvas 1920×1080) | Shell |
|---|---|---|
| Left margin | 100–104 | 100–104 |
| Avatar card | 450×290 at (104, 292) | 450×290 at (104, 292) |
| Friend activity | no boxed panel; 72 px rows | no boxed panel; 72 px rows |
| Featured grid | x 585, y 292 | x 585, y 292 |
| Card size | 300×300 | 300×300 |
| Card gap | 12 | 12 |
| Grid right edge | ~1821 | 1821 |

The navigation is 47 px Source Sans Pro Light at x=104, with 34 px item gaps, a 2 px selected underline, and the wordmark sized to the measured 222×49 visible bbox.

### 1.6 Luma-adaptive dimming

The real 2016 background is dark; the generated fallback is also dark. A fixed 160-alpha title scrim hid the actual footage. The video decoder now computes the average luminance of each frame:

- dark footage gets a light scrim and low dim;
- bright user footage gets a stronger scrim and higher dim;
- the generated fallback keeps its measured fixed values.

This keeps the 2016 background visible while preserving text contrast on arbitrary user videos.

---

## 2. Verification

| Check | Result |
|---|---|
| Font | App log: `font family: Source Sans Pro` |
| Background video | Release decoded `menu.mp4`; Settings shows `Background video playing` |
| Music | Release loaded `menu.mp3`; the analog output monitor measured −16.3 dB mean, so the theme is audible. The loop now uses a decoded in-memory source with the 76 ms lead silence trimmed. |
| UI sounds | Original 2016 source set installed in local user data; focus/select/back/toggle/launch/notice wired; WAV fallback decode test passes |
| Avatar | `full-body avatar loaded`; the Avatar tab shows the full render and the Home card uses an upper-body crop |
| Square game tiles | Home, Games, and Details now use the square `games/icons` endpoint |
| Friends / presence / Robux | 20 friends, 20 presence entries, real balance |
| Session cleanup | Sign-out activated `settings:account-signout`; app returned to Sign In; no session validated on restart |
| Title comparison | Updated `docs/screenshots/compare/title.png` |
| Sign-in comparison | Updated `docs/screenshots/compare/signin.png` |
| Home comparison | Updated `docs/screenshots/compare/home.png` |
| Final release launch | Joined `189707` and `119958473240233` on the release build. The later Games/Search layout changes do not touch `launch.rs`. |
| Tests | `cargo test` 15 passed; `cargo check` clean |
| Release binary | 35 MB, includes the OFL fonts |

The 2026-11-03 pass re-captured the title, sign-in, home, games, details, and settings-audio screens against the local 2016 background. The recordings are not in the app bundle. Signed-in screenshots remain local-only because they contain the account and friends.

---

## 3. Limitations

- `ScaledWorldv4.7.rbxl` is not in the provided tree or the upstream content we checked. The background is a recording of the real 2016 scene, not the engine-rendered place.
- The background and music are copyrighted Roblox recordings. They are installed only in this machine’s user data folder and are not in the repository. Another machine gets the generated fallback and no music until the user supplies files.
- The avatar is a full-body Roblox thumbnail render, not an interactive 3D character.
- The source’s `particles7.rbxl` is an older particle test place, not the final `ScaledWorldv4.7` intro scene.
- Source Sans Pro is from the 2016 source; the shell may have used a specific font variant, but the family matches the engine font set.
- Controller hardware is still unverified.
- The exposed Sober session still needs rotation.

---

## 4. Files added or changed in this pass

- `assets/fonts/SourceSansPro-*.ttf`, `assets/fonts/OFL.txt`
- `src/theme.rs`: Source Sans Pro primary font, Noto/emoji fallback.
- `src/video_background.rs`: video luma calculation.
- `src/canvas.rs`: `image_contain`, `video_luma`.
- `src/app.rs`: luma propagation, full-body avatar event, adaptive dims.
- `src/api.rs`: full-body avatar endpoint.
- `src/auth.rs`: `Session.avatar`.
- `src/screens/boot.rs`, `signin.rs`: measured title/sign-in layout.
- `src/screens/home.rs`: measured left column, grid, full-body avatar.
- `src/screens/placeholder.rs`: full-body Avatar tab.
- `src/screens/mod.rs`: Source Sans nav size/position, section labels, ghost backdrops.
- `docs/screenshots/*`, `docs/screenshots/compare/*`: regenerated with the fallback.
- `docs/references.md`, `docs/ui-reference.md`, `README.md`, `PLAN.md`, `docs/phase-4-report.md`.
