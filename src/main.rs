mod api;
mod app;
mod audio;
mod auth;
mod background;
mod canvas;
mod config;
mod focus;
mod images;
mod input;
mod instance;
mod launch;
mod login_helper;
mod models;
mod screens;
mod theme;
mod util;
mod video_background;

use clap::Parser;
use eframe::egui;
use std::sync::Arc;

#[derive(Parser)]
#[command(
    name = "roblox-2016-xbox",
    version,
    about = "Rust reimplementation of the 2016 Roblox Xbox shell"
)]
struct Args {
    /// Start on a screen: boot, signin, home, games, favorites, search, details,
    /// settings, launch, browse, profile, chat, chatfriends, addfriend,
    /// avatar, avatar-all, outfits, avatar-body, avatar-clothing,
    /// avatar-accessories, avatar-animations, avatar-emotes, avatar-skin,
    /// avatar-build
    #[arg(long)]
    screen: Option<String>,

    /// Save one screenshot to this path, then exit
    #[arg(long)]
    screenshot: Option<std::path::PathBuf>,

    /// Seconds to wait before the automatic screenshot
    #[arg(long, default_value_t = 6.0)]
    wait: f32,

    /// Ignore keyboard, mouse, and controller input (for stable screenshots)
    #[arg(long)]
    lock_input: bool,

    /// Settings tab: account, audio, video, controls, diagnostics
    #[arg(long)]
    settings_tab: Option<String>,

    /// Run a fixed navigation sequence for acceptance testing
    #[arg(long)]
    demo: bool,

    /// Run the account sign-out sequence from Settings (acceptance testing)
    #[arg(long)]
    demo_auth: bool,

    /// Navigate Home -> Details -> Play for launch acceptance testing
    #[arg(long)]
    demo_launch: bool,

    /// Press Play on the preloaded Details screen (acceptance testing)
    #[arg(long)]
    demo_play: bool,

    /// Search -> Details -> Play for acceptance testing
    #[arg(long)]
    demo_search_play: bool,

    /// Drive the Settings Video tab controls for acceptance testing
    #[arg(long)]
    demo_settings_video: bool,

    /// Import the existing Sober session once at startup (read-only)
    #[arg(long)]
    import_sober: bool,

    /// Disable audio output (used by the offscreen capture harness)
    #[arg(long)]
    no_audio: bool,

    /// Allow more than one instance (development and capture runs)
    #[arg(long, hide = true)]
    allow_multiple: bool,
}

struct TeeWriter {
    file: Option<std::fs::File>,
    stderr: std::io::Stderr,
}

impl std::io::Write for TeeWriter {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        if let Some(file) = &mut self.file {
            let _ = file.write_all(buffer);
        }
        self.stderr.write_all(buffer)?;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(file) = &mut self.file {
            let _ = file.flush();
        }
        self.stderr.flush()
    }
}

fn load_icon(bytes: &[u8]) -> egui::IconData {
    let image = image::load_from_memory(bytes)
        .expect("embedded app icon")
        .to_rgba8();
    let (width, height) = image.dimensions();
    egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}

fn main() -> eframe::Result<()> {
    let log_paths = config::Paths::new();
    log_paths.ensure_directories();
    let app_log = log_paths.state_dir.join("app.log");
    if std::fs::metadata(&app_log).is_ok_and(|metadata| metadata.len() > 2_000_000) {
        let _ = std::fs::write(&app_log, b"");
    }
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&app_log)
        .ok();
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("roblox_2016_xbox=info,warn"),
    )
    .target(env_logger::Target::Pipe(Box::new(TeeWriter {
        file: log_file,
        stderr: std::io::stderr(),
    })))
    .init();

    if std::env::args().any(|arg| arg == "--login-helper") {
        std::process::exit(login_helper::run());
    }

    let args = Args::parse();
    let startup_config = config::Config::load(&log_paths);
    let start_maximized = args.screenshot.is_none() && startup_config.window_maximized;
    let _instance = if !args.allow_multiple && args.screenshot.is_none() {
        let paths = config::Paths::new();
        match instance::InstanceGuard::acquire(&paths.state_dir) {
            Some(guard) => Some(guard),
            None => {
                eprintln!("Roblox 2016 Xbox is already running.");
                return Ok(());
            }
        }
    } else {
        None
    };
    let app_id = if args.screenshot.is_some() {
        format!("roblox-2016-xbox-screenshot-{}", std::process::id())
    } else {
        "roblox-2016-xbox".to_owned()
    };
    let persistence_path = args.screenshot.as_ref().map(|_| {
        std::path::PathBuf::from(format!(
            "/tmp/opencode/roblox-2016-shot-{}.ron",
            std::process::id()
        ))
    });
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1920.0, 1080.0])
            .with_min_inner_size([1280.0, 720.0])
            .with_title("Roblox 2016 Xbox")
            .with_maximized(start_maximized)
            .with_app_id(app_id)
            .with_icon(Arc::new(load_icon(include_bytes!(
                "../XboxClient/img/logo208.png"
            )))),
        renderer: eframe::Renderer::Wgpu,
        persist_window: args.screenshot.is_none(),
        persistence_path,
        ..Default::default()
    };

    let start = app::StartOptions {
        screen: args.screen,
        screenshot: args.screenshot,
        wait: args.wait,
        lock_input: args.lock_input,
        settings_tab: args.settings_tab,
        demo: args.demo,
        demo_auth: args.demo_auth,
        demo_launch: args.demo_launch,
        demo_play: args.demo_play,
        demo_search_play: args.demo_search_play,
        demo_settings_video: args.demo_settings_video,
        import_sober: args.import_sober,
        no_audio: args.no_audio,
    };

    eframe::run_native(
        "Roblox 2016 Xbox",
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, start)))),
    )
}
