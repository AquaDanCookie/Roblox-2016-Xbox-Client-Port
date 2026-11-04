# Screenshots

These images were captured from the Rust shell. The games, search, and details images were refreshed on 2026-11-04 at 1600x900; the remaining images were captured on 2026-11-03 at 1504x846. The title, sign-in, home, games, search, details, and audio captures use the local 2016 background recording so the measured 2016 layout can be compared; that recording is not in the app bundle. See [`../fidelity-pass.md`](../fidelity-pass.md) and [`../ui-audio-pass-2026-11-03.md`](../ui-audio-pass-2026-11-03.md).

- `title.png`: boot/title with the 2016 wordmark, measured 50 px tagline, video background, and the press-any-button prompt.
- `signin.png`: sign-in choice with the video background, WebView sign-in, read-only Sober import, guest browsing, and Create Account.
- `webview-login.png`: the real Roblox login page in the isolated WebKitGTK helper.
- `home.png`: live 4×2 square-tile featured grid, focused avatar card, guest avatar, and honest friend empty state. Signed-in sessions use an upper-body avatar crop.
- `games.png`: live square-tile sort carousels with the 2016 overflow buttons.
- `search.png`: live search results for "Adopt Me".
- `details.png`: live details for Bug Evolution by Corgi Crew, with a square game icon and the measured media column.
- `launch.png`: real Joined state for Natural Disaster Survival (`189707`) after Search → Details → Play on the final release binary.
- `launch-dontsplash.png`: real Joined state for the first featured game, DONT SPLASH, from Home → Details → Play on the final release binary.
- `launch-failed.png`: honest Launch failed state, captured while another game had exhausted VRAM. It shows crash detection and the Retry / Open Sober / Back actions.
- `launch-unverified.png`: the signed-out Sober branch with Launch anyway, Open Sober, and Back.
- `settings-*.png`: account, audio, video, controls, and diagnostics, including the user-supplied music/video paths and the app log path.

Signed-in account screenshots are kept outside the repository because they contain the account name, user ID, avatar, and friends. Phase 2 verification copies are in `/tmp/opencode/phase2-account/` on this machine only.

`compare/` contains side-by-side images with 2016 reference frames. Those frames are local analysis material only and must not be redistributed. The app does not bundle them.

The shell does not bundle 2016 music or video. On this machine the real 2016 background and theme are installed in `~/.local/share/roblox-2016-xbox/` for local use only. Other machines fall back to the generated scene and no music until files are supplied. Settings has buttons to open both folders and reload the files.

The live game thumbnails come from the Roblox CDN and remain the property of their creators. They are shown here as part of the local build.
