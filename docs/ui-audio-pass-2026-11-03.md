# 2016 Xbox UI and Audio Pass — 2026-11-03

This pass fixed the menu-music loop and rebuilt the visible shell against the real 2016 Xbox One client, using higher-resolution captures than the original 720p reference set.

> **Follow-up corrections:** The tab underline now sits below the glyphs (`y+31`, 3 px). The focus ring is a thick white rule with a soft cyan halo. The Home/Avatar account label is the signed-in display name, not the sample name `Lost Affect`, and text sits on a shadow without a plate behind it. The Xbox button prompts are hidden. The Xbox 2016–2023 menu SFX set is installed locally and mapped by name. The menu theme stays silent while a launched runtime is running. The `...` overflow on Home and Games opens the sliding category browser from the 2016 capture, not a grid. Games categories scroll horizontally. Friends scroll vertically and show Offline instead of the API's `Website` location. Details use thumb ratings, a scrollable description, and no overflow button. The audio loop returns `None` from `Source::current_span_len` and uses a short 0.6 s crossfade, so rodio does not rebuild the resampler and the tail no longer overlaps the head for long enough to sound like a second copy.
>
> **Second follow-up:** Ratings now use the 2016 outlined-in-a-ring thumb icons with bare vote counts, and the description is drawn once instead of a ghosted double pass. The instance guard uses an OS advisory file lock (`fs2`), so two launches cannot race and play the theme twice. Opening the category browser slides the page in and plays `SideMenuSlideIn.wav`; selecting a card grows it while the rail slides. Hover focus is disabled on the rail and excluded from the Games auto-scroll, which stops cursor-driven runaway scrolling. The Games category scroll uses a smoothed target instead of hard jumps. The theme ducks to 55% on Games/Browse/Details/Search so UI sounds sit above it.
>
> **Social pass:** The Friends list sorts online users first, offers Join for friends with a live place ID, and opens a 2016-styled profile with avatar, counts, Chat/Add Friend/Join, and currently wearing items. The Friends tab has an Add Friends form backed by the username lookup and friend-request APIs. A new Chat tab reads the Roblox platform-chat conversations/messages and sends messages. The Avatar tab has a Hats/Hair/Face/Shirts/Pants inventory browser and saves wearing-asset changes through the avatar API while keeping the 2D full-body preview. The Games tab has a Search control and reads all carousel sorts returned by the charts API. Browsing back plays `SideMenuSlideOut` (reverse of the slide-in) and slides the page out before returning.

## 1. Audio loop fix

The old player used `Decoder::repeat_infinite()` directly. That put the MP3 decoder on the loop boundary and, more importantly, the 2016 theme file starts with about 76 ms of near-digital-silence. Each loop therefore had a small silent gap and a decoder restart at exactly the point where the music repeats, which sounded like a stutter or a short rewind.

The player now:

- decodes the menu file once into memory;
- trims only pure silence at the start and end at frame boundaries;
- plays a custom `LoopSource` that wraps sample-by-sample with no decoder, no seek, and no buffer refill at the loop point;
- adds each audio queue to the mixer only after its source has been appended;
- prevents a second app instance from starting and playing a second copy of the theme.

The start of the decoded theme is trimmed by 0.076 s (3,630 frames). The loop is 8,985,008 frames at 48 kHz (187.19 s) before trimming and 8,981,378 frames after. The new loop source and edge-trimming logic have unit tests.

## 2. Reference analysis

The main references were:

| Source | Use |
|---|---|
| <https://www.youtube.com/watch?v=GhCUQpPJDww> | Real 2016 Xbox client running in 2026; 1080p home/title/avatar/details frames |
| <https://www.youtube.com/watch?v=Xmbbrp1TmHM> | 2016 Xbox One client title screen at 1080p |
| <https://www.youtube.com/watch?v=pmObfZGsKIE> | Original 2016 gameplay capture used for the first reference pass |
| `roblox.fandom.com/wiki/Roblox_(Xbox)` | `RobloxXboxTitleScreen2016.png` title still |
| <https://www.youtube.com/watch?v=u6Sc53ztspQ> | 2025 1:1 recreation, used only as a secondary cross-check |
| <https://www.youtube.com/watch?v=nD_OScrernE> | Xbox UI Launcher trailer, secondary cross-check |

The recreation videos were not treated as source of truth. The final measurements come from the real-client frames and the original 2016 capture.

### 2.1 Measured layout at 1920×1080

| Element | 2016 reference | Rust shell |
|---|---|---|
| Wordmark | visible ~222×49 at (103, 43) | visible ~221×50 at (103, 45) |
| Top nav | 47 px Source Sans Pro Light, first item x=104, baseline centre ~196, 34 px gaps | same |
| Selected tab | white label, 2 px light-cyan underline | same |
| Home section labels | x=100 and x=585, y≈252 | same |
| Avatar card | 450×290 at (104, 292), 3 px cyan focus border | same |
| Home grid | 4×2 square tiles, 300×300, 12 px gap, origin (585, 292) | same |
| Friend activity | no boxed panel; 72 px headshots, 132 px row step | same |
| Games | three carousels, 2×2 square tiles per carousel, origin x=96/750/1404 | same |
| Details title | ~50 px light, x=156, centre y≈180 | same |
| Details thumbnail | ~540×540 square game icon at (156, 278) | same |
| Details media | 432×243 primary, two 210×118 thumbs | same |
| PLAY / FAVORITE | 232×80 and 220×80 at y≈952 | same |
| Controller prompts | circled A/B/X/Y, 48 px circle, 26 px label | same |

### 2.2 What changed visually

- Game images are now the square `games/icons` endpoint instead of 16:9 thumbnails. The 2016 shell used square game tiles on Home, Games, and Details.
- Home is a 4×2 grid of 300 px square tiles with the 2016 spacing and an overflow `...` button.
- Home avatar card is landscape 450×290, uses an upper-body crop of the avatar render, and no longer overlays the display name.
- Friend activity is drawn directly on the background with separators, not inside a modern panel.
- The 2016 focus treatment is a 3 px light-cyan border with a very small outer edge, not a large neon glow.
- Navigation uses Source Sans Pro Light at 47 px with the measured gaps and a short selected underline.
- Buttons, details copy, tagline, and prompts were resized to the 2016 proportions. The previous text was too small on Home, Details, and the title/sign-in screens.
- Details uses a 540 px square icon, a narrower media column, exact vote counts, and the overflow button at the bottom-right of the media area.
- The Avatar tab now follows the `Lost Affect / My Collection / Catalog` composition. Catalog tiles are honest placeholders because the 2016 catalog and 3D editor are not reproduced.
- The video background is brightened slightly (`eq=brightness=0.05:contrast=1.05:saturation=1.12`) and the full-screen dim is much lighter, so the real 2016 footage stays visible like the reference.
- The title/sign-in tagline is now 50 px, matching the reference, instead of the previous small 34 px text.

## 3. Verification

All UI checks were run offscreen under Xvfb with the Vulkan lavapipe driver. No window was opened on the user's desktop.

- `cargo test`: 15 passed.
- `cargo build --release`: success.
- Offscreen captures: title, sign-in, home, games, details, settings audio.
- Audio path: the loop source is tested for sample-exact wrapping. The real theme decodes to 8,981,378 frames (187.112 s) after trimming 76 ms of leading silence.
- Real-theme playback: the EasyEffects monitor measured −15.8 dB mean / −2.4 dB max while the theme played.
- Loop boundary test: a 3-second test track was played for 9 seconds (three loop points) through the same `LoopSource`; `silencedetect` found no gap at −50 dB / 20 ms. The physical sink was muted during the test and restored afterwards.
- Duplicate-instance guard: a second normal launch exits with `Roblox 2016 Xbox is already running.`

Updated screenshots:

- `docs/screenshots/title.png`
- `docs/screenshots/signin.png`
- `docs/screenshots/home.png`
- `docs/screenshots/games.png`
- `docs/screenshots/details.png`
- `docs/screenshots/settings-audio.png`
- `docs/screenshots/compare/*.png`

## 4. Limitations

- The signed-in Home and Avatar screens were not re-captured because the app session is still cleared. The layout code follows the reference, but the avatar bust crop and friend rows need a final visual check after signing in.
- The installed background recording cycles through bright and dark scenes. Some captures catch a dark space-station frame or its bright runway line; that is the recording, not a UI element.
- The 2016 catalog, 3D avatar editor, and exact 2016 game tiles are not reproduced.
- Controller hardware acceptance is still unverified.
- The recordings remain local user assets and are not in the repository.
