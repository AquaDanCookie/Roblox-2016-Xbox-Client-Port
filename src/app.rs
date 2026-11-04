use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::process::Child;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use eframe::egui::{self, Color32, Pos2, Rect, TextureHandle, Vec2, ViewportCommand};
use secrecy::{ExposeSecret, SecretString};

use crate::api::{Api, ApiError, ApiEvent};
use crate::audio::{Music, Sfx};
use crate::auth::{self, AuthState, LoginProcess, Session, SessionStore};
use crate::background;
use crate::canvas::Canvas;
use crate::config::{Config, Paths};
use crate::focus::{Dir, FocusItem, FocusState};
use crate::images::ImageCache;
use crate::input::{self, Action, Gamepad};
use crate::launch::{self, LaunchLog, LaunchRuntime};
use crate::models::{
    AuthenticatedUser, AvatarAsset, AvatarAssetType, AvatarBundlesBundle, AvatarScales, BodyColors,
    ChatConversation, ChatMessage, DetailsBundle, EquippedEmote, GameSummary, InventoryItem, Load,
    OutfitFile, PlatformOutfitsBundle, ProfileBundle, SavedOutfit, SortInfo,
};
use crate::screens::{NavTab, Screen, ScrollDrag};
use crate::theme;
use crate::video_background::{self, VideoBackground};

pub const AVATAR_INVENTORY_TYPES: &[u64] = &[
    2, 8, 11, 12, 17, 18, 19, 27, 28, 29, 30, 31, 41, 42, 43, 44, 45, 46, 47, 48, 50, 51, 52, 53,
    54, 55, 61, 79,
];

pub struct StartOptions {
    pub screen: Option<String>,
    pub screenshot: Option<PathBuf>,
    pub wait: f32,
    pub lock_input: bool,
    pub settings_tab: Option<String>,
    pub demo: bool,
    pub demo_auth: bool,
    pub demo_launch: bool,
    pub demo_play: bool,
    pub demo_search_play: bool,
    pub demo_settings_video: bool,
    pub import_sober: bool,
    pub no_audio: bool,
}

pub struct App {
    pub paths: Paths,
    pub config: Config,
    pub api: Api,
    pub api_tx: Sender<ApiEvent>,
    pub api_rx: Receiver<ApiEvent>,
    pub screenshot_tx: Sender<Arc<egui::ColorImage>>,
    pub screenshot_rx: Receiver<Arc<egui::ColorImage>>,
    pub images: ImageCache,
    pub music: Music,
    pub background: TextureHandle,
    pub procedural_background: TextureHandle,
    pub video: Option<VideoBackground>,
    pub video_error: Option<String>,
    pub wordmark: Option<TextureHandle>,
    pub focus: FocusState,
    pub registry: Vec<FocusItem>,
    pub screen: Screen,
    pub nav: NavTab,
    pub placeholder_tab: NavTab,
    pub catalog: CatalogState,
    pub games: GamesState,
    pub browse: BrowseState,
    pub favorites: FavoritesState,
    pub favorite_states: HashMap<u64, bool>,
    pub favorite_status_requested: HashSet<u64>,
    pub favorite_pending: HashSet<u64>,
    pub friends_scroll: f32,
    pub home_friends_scroll: f32,
    pub scroll_drag: Option<ScrollDrag>,
    pub search: SearchState,
    pub details: DetailsState,
    pub settings: SettingsState,
    pub profile: ProfileState,
    pub add_friend: AddFriendState,
    pub chat: ChatState,
    pub avatar_edit: AvatarEditState,
    pub launch: LaunchState,
    pub gamepad: Gamepad,
    pub confirm: bool,
    pub toast: Option<Toast>,
    pub transition: Transition,
    pub screen_changed: bool,
    pub api_status: String,
    pub last_api_error: Option<String>,
    pub sober: SoberStatus,
    pub auth_state: AuthState,
    pub auth_error: Option<String>,
    pub session: Option<Session>,
    pub session_store: SessionStore,
    pub login_process: Option<LoginProcess>,
    pub auto_screenshot: Option<AutoScreenshot>,
    pub close_after_screenshot: bool,
    pub window_focused: bool,
    pub focus_visible: bool,
    pub avatar_refresh_at: Option<Instant>,
    pub lock_input: bool,
    pub demo: Option<DemoRun>,
    pub started: Instant,
}

pub struct DemoRun {
    pub start: Instant,
    pub steps: Vec<(Duration, Action)>,
    pub index: usize,
}

impl DemoRun {
    fn auth() -> Self {
        Self {
            start: Instant::now(),
            steps: vec![
                (Duration::from_secs(3), Action::Navigate(Dir::Right)),
                (Duration::from_millis(3500), Action::Confirm),
            ],
            index: 0,
        }
    }

    fn launch() -> Self {
        Self {
            start: Instant::now(),
            steps: vec![
                (Duration::from_secs(3), Action::Navigate(Dir::Down)),
                (Duration::from_millis(3500), Action::Navigate(Dir::Right)),
                (Duration::from_secs(4), Action::Confirm),
                (Duration::from_secs(5), Action::Confirm),
            ],
            index: 0,
        }
    }

    fn play() -> Self {
        Self {
            start: Instant::now(),
            steps: vec![(Duration::from_secs(3), Action::Confirm)],
            index: 0,
        }
    }

    fn search_play() -> Self {
        Self {
            start: Instant::now(),
            steps: vec![
                (Duration::from_secs(3), Action::Navigate(Dir::Down)),
                (Duration::from_millis(3500), Action::Navigate(Dir::Left)),
                (Duration::from_secs(4), Action::Confirm),
                (Duration::from_secs(10), Action::Confirm),
            ],
            index: 0,
        }
    }

    fn settings_video() -> Self {
        Self {
            start: Instant::now(),
            steps: vec![
                (Duration::from_secs(3), Action::Navigate(Dir::Right)),
                (Duration::from_millis(3500), Action::Navigate(Dir::Down)),
                (Duration::from_secs(4), Action::Navigate(Dir::Right)),
                (Duration::from_millis(4500), Action::Confirm),
            ],
            index: 0,
        }
    }

    fn new() -> Self {
        Self {
            start: Instant::now(),
            steps: vec![
                (Duration::from_secs(3), Action::Navigate(Dir::Down)),
                (Duration::from_millis(3500), Action::Navigate(Dir::Right)),
                (Duration::from_secs(4), Action::Confirm),
                (Duration::from_secs(5), Action::Confirm),
                (Duration::from_secs(6), Action::Back),
                (Duration::from_millis(6500), Action::Back),
                (Duration::from_secs(7), Action::Search),
                (Duration::from_secs(8), Action::Back),
                (Duration::from_millis(8300), Action::Back),
                (Duration::from_millis(8600), Action::Settings),
                (Duration::from_millis(9600), Action::Back),
                (Duration::from_secs(10), Action::ToggleMute),
                (Duration::from_millis(10500), Action::ToggleMute),
            ],
            index: 0,
        }
    }
}

pub struct CatalogState {
    pub sorts: Load<Vec<SortInfo>>,
    pub sort_games: HashMap<String, Load<Vec<GameSummary>>>,
    pub requested_sorts: bool,
    pub requested_content: HashSet<String>,
    pub thumbnail_urls: HashMap<u64, String>,
    pub requested_thumbnails: HashSet<u64>,
}

#[derive(Default)]
pub struct GamesState {
    pub scroll: f32,
    pub scroll_target: f32,
    pub home_offset: usize,
    pub last_focus: Option<String>,
    pub wheel_cooldown: f32,
}

pub struct FavoritesState {
    pub games: Load<Vec<GameSummary>>,
    pub requested: bool,
    pub from: Screen,
}

pub struct LightboxState {
    pub urls: Vec<String>,
    pub index: usize,
    pub previous_index: Option<usize>,
    pub slide_started: Option<Instant>,
    pub slide_forward: bool,
}

pub struct BrowseState {
    pub sort_id: String,
    pub title: String,
    pub from: Screen,
    pub selected: usize,
    pub scroll_pos: f32,
    pub opened_at: Instant,
    pub closing_at: Option<Instant>,
    pub card_rects: Vec<Rect>,
    pub wheel_accum: f32,
    pub wheel_cooldown: f32,
}

pub struct SearchState {
    pub query: String,
    pub last_query: String,
    pub editing: bool,
    pub focus_requested: bool,
    pub results: Load<Vec<GameSummary>>,
    pub last_edit: Instant,
    pub last_submitted: String,
    pub return_screen: Screen,
}

pub struct DetailsState {
    pub universe_id: u64,
    pub root_place_id: u64,
    pub details: Load<DetailsBundle>,
    pub requested: bool,
    pub from: Screen,
    pub focus_play: bool,
    pub scroll: f32,
    pub lightbox: Option<LightboxState>,
}

pub struct SettingsState {
    pub tab: SettingsTab,
    pub from: Screen,
    pub drag_slider: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AvatarCategory {
    All,
    Avatars,
    Body,
    Clothing,
    Accessories,
    Animations,
    Emotes,
}

impl AvatarCategory {
    pub const ALL: [AvatarCategory; 7] = [
        AvatarCategory::All,
        AvatarCategory::Avatars,
        AvatarCategory::Body,
        AvatarCategory::Clothing,
        AvatarCategory::Accessories,
        AvatarCategory::Animations,
        AvatarCategory::Emotes,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Avatars => "Avatars",
            Self::Body => "Body",
            Self::Clothing => "Clothing",
            Self::Accessories => "Accessories",
            Self::Animations => "Animations",
            Self::Emotes => "Emotes",
        }
    }

    pub fn subcategories(self) -> &'static [(u64, &'static str)] {
        match self {
            Self::All | Self::Avatars => &[],
            Self::Body => &[
                (2000, "Full Body"),
                (41, "Hair"),
                (17, "Head"),
                (1001, "Build"),
                (1000, "Skin"),
                (79, "Dynamic Head"),
                (18, "Face"),
            ],
            // Classic clothing only. The layered Top/Outerwear/Bottom/Shoes
            // categories are intentionally excluded.
            Self::Clothing => &[
                (11, "Shirt Classic"),
                (12, "Pants Classic"),
                (2, "T-Shirt Classic"),
            ],
            Self::Accessories => &[
                (8, "Head"),
                (42, "Face"),
                (43, "Neck"),
                (44, "Shoulder"),
                (45, "Front"),
                (46, "Back"),
                (47, "Waist"),
                (19, "Gear"),
            ],
            Self::Animations => &[
                (2001, "Full Set"),
                (51, "Idle"),
                (55, "Walk"),
                (53, "Run"),
                (52, "Jump"),
                (50, "Fall"),
                (48, "Climb"),
                (54, "Swim"),
            ],
            Self::Emotes => &[(61, "Emotes")],
        }
    }

    pub fn first_type(self) -> u64 {
        self.subcategories()
            .first()
            .map(|(kind, _)| *kind)
            .unwrap_or(0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkinPart {
    All,
    Head,
    Torso,
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
}

impl SkinPart {
    pub const ALL: [SkinPart; 7] = [
        Self::All,
        Self::Head,
        Self::Torso,
        Self::LeftArm,
        Self::RightArm,
        Self::LeftLeg,
        Self::RightLeg,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Head => "Head",
            Self::Torso => "Torso",
            Self::LeftArm => "Left Arm",
            Self::RightArm => "Right Arm",
            Self::LeftLeg => "Left Leg",
            Self::RightLeg => "Right Leg",
        }
    }
}

pub struct AvatarEditState {
    pub category: AvatarCategory,
    pub profile: Load<ProfileBundle>,
    pub requested: bool,
    pub worn: Vec<crate::models::AvatarAsset>,
    pub emotes: Vec<EquippedEmote>,
    pub body_colors: Option<BodyColors>,
    pub scales: Option<AvatarScales>,
    pub player_avatar_type: Option<String>,
    pub worn_initialized: bool,
    pub inventory: HashMap<u64, Load<crate::models::InventoryBundle>>,
    pub selected_type: u64,
    pub saving: bool,
    pub appearance_saving: bool,
    pub pending_save: bool,
    pub status: Option<(String, bool)>,
    pub outfits: Vec<SavedOutfit>,
    pub outfits_for: Option<u64>,
    pub outfit_name: String,
    pub bundles: Load<AvatarBundlesBundle>,
    pub bundles_requested: bool,
    pub platform_outfits: Load<PlatformOutfitsBundle>,
    pub outfits_requested: bool,
    pub all_requested: bool,
    pub scroll: f32,
    pub body_part: u64,
    pub body_part_choice: u64,
    pub body_part_menu: bool,
    pub build_drag: Option<u8>,
    pub skin_part: SkinPart,
    pub picker_hue: f32,
    pub picker_sat: f32,
    pub picker_val: f32,
    pub picker_hex: String,
    pub picker_drag: Option<u8>,
}

pub struct ChatState {
    pub conversations: Load<Vec<ChatConversation>>,
    pub requested: bool,
    pub selected: Option<String>,
    pub messages: Load<Vec<ChatMessage>>,
    pub messages_for: Option<String>,
    pub start_with: Option<(u64, String)>,
    pub start_requested: bool,
    pub selected_name: Option<String>,
    pub compose: String,
    pub editing: bool,
    pub focus_requested: bool,
    pub filter: String,
    pub status: Option<(String, bool)>,
    pub show_friends: bool,
    pub list_scroll: f32,
    pub closed_chats: HashSet<String>,
    pub from: Screen,
}

pub struct AddFriendState {
    pub username: String,
    pub target: Option<(u64, String)>,
    pub status: Option<(String, bool)>,
    pub from: Screen,
    pub editing: bool,
    pub focus_requested: bool,
}

pub struct ProfileState {
    pub user_id: u64,
    pub display_name: String,
    pub username: String,
    pub headshot_url: Option<String>,
    pub presence: String,
    pub place_id: u64,
    pub from: Screen,
    pub bundle: Load<ProfileBundle>,
    pub requested: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsTab {
    Account,
    Audio,
    Video,
    Controls,
    Diagnostics,
}

impl SettingsTab {
    pub const ALL: [SettingsTab; 5] = [
        SettingsTab::Account,
        SettingsTab::Audio,
        SettingsTab::Video,
        SettingsTab::Controls,
        SettingsTab::Diagnostics,
    ];

    pub fn label(self) -> &'static str {
        match self {
            SettingsTab::Account => "Account",
            SettingsTab::Audio => "Audio",
            SettingsTab::Video => "Video",
            SettingsTab::Controls => "Controls",
            SettingsTab::Diagnostics => "Diagnostics",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            SettingsTab::Account => "settings:tab:account",
            SettingsTab::Audio => "settings:tab:audio",
            SettingsTab::Video => "settings:tab:video",
            SettingsTab::Controls => "settings:tab:controls",
            SettingsTab::Diagnostics => "settings:tab:diagnostics",
        }
    }

    pub fn index(self) -> usize {
        match self {
            SettingsTab::Account => 0,
            SettingsTab::Audio => 1,
            SettingsTab::Video => 2,
            SettingsTab::Controls => 3,
            SettingsTab::Diagnostics => 4,
        }
    }

    pub fn from_index(index: usize) -> Self {
        Self::ALL
            .get(index)
            .copied()
            .unwrap_or(SettingsTab::Account)
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "account" => Some(SettingsTab::Account),
            "audio" => Some(SettingsTab::Audio),
            "video" => Some(SettingsTab::Video),
            "controls" => Some(SettingsTab::Controls),
            "diagnostics" => Some(SettingsTab::Diagnostics),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaunchPhase {
    Checking,
    Launching,
    Waiting,
    Joined,
    SessionUnverified,
    Mismatch,
    Failed,
}

pub struct LaunchState {
    pub name: String,
    pub root_place_id: u64,
    pub started: Instant,
    pub from: Screen,
    pub phase: LaunchPhase,
    pub check_requested: bool,
    pub sober_account: Option<AuthenticatedUser>,
    pub session_error: Option<String>,
    pub child: Option<Child>,
    pub log: Option<LaunchLog>,
    pub log_path: Option<PathBuf>,
    pub message: Option<String>,
    pub last_poll: Instant,
}

pub struct Toast {
    pub message: String,
    pub created: Instant,
    pub duration: Duration,
}

pub struct Transition {
    pub start: Instant,
    pub duration: Duration,
}

impl Transition {
    pub fn idle() -> Self {
        Self {
            start: Instant::now() - Duration::from_secs(10),
            duration: Duration::ZERO,
        }
    }

    pub fn active(&self) -> bool {
        self.start.elapsed() < self.duration
    }

    pub fn alpha(&self) -> u8 {
        if !self.active() || self.duration.is_zero() {
            return 0;
        }
        let t = (self.start.elapsed().as_secs_f32() / self.duration.as_secs_f32()).clamp(0.0, 1.0);
        (180.0 * (1.0 - t)) as u8
    }
}

pub struct AutoScreenshot {
    pub path: PathBuf,
    pub deadline: Instant,
    pub requested: bool,
}

pub struct SoberStatus {
    pub installed: bool,
    pub version: String,
    pub detail: String,
    pub cordial_installed: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, options: StartOptions) -> Self {
        let paths = Paths::new();
        paths.ensure_directories();
        let config = Config::load(&paths);

        let fonts = theme::install_fonts(&cc.egui_ctx);
        log::info!("font family: {}", fonts.primary);
        theme::configure_egui(&cc.egui_ctx);
        cc.egui_ctx.set_zoom_factor(1.0);

        let procedural_background = background::texture(&cc.egui_ctx);
        let background = procedural_background.clone();
        let wordmark = load_texture(
            &cc.egui_ctx,
            "roblox-wordmark",
            include_bytes!("../assets/roblox_wordmark_hi.png"),
        );
        let (video, video_error) = start_background_video(&paths, &config);

        let api = match Api::new() {
            Ok(api) => api,
            Err(error) => panic!("could not start the network runtime: {error}"),
        };
        let (api_tx, api_rx) = mpsc::channel();
        let (screenshot_tx, screenshot_rx) = mpsc::channel();
        let images = ImageCache::new(api.runtime_handle(), paths.thumbnail_dir());
        let music = Music::new(&paths, &config, options.no_audio);
        log::info!(
            "music: {} path={:?} master={:.2} music={:.2} muted={} ui_sounds={}",
            music.status_text(),
            music.path(),
            config.master_volume,
            config.music_volume,
            config.muted,
            config.ui_sounds
        );
        let gamepad = Gamepad::new();
        let sober = detect_sober();
        let initial_settings_tab = SettingsTab::from_index(config.settings_tab);
        let lock_input = options.lock_input;
        let demo = options.demo;
        let demo_auth = options.demo_auth;
        let demo_launch = options.demo_launch;
        let demo_play = options.demo_play;
        let demo_search_play = options.demo_search_play;
        let demo_settings_video = options.demo_settings_video;
        let import_sober = options.import_sober;
        let session_store = SessionStore::new(paths.clone());

        let mut app = Self {
            paths,
            config,
            api,
            api_tx,
            api_rx,
            screenshot_tx,
            screenshot_rx,
            images,
            music,
            background,
            procedural_background,
            video,
            video_error,
            wordmark,
            focus: FocusState::default(),
            registry: Vec::new(),
            screen: Screen::Boot,
            nav: NavTab::Home,
            placeholder_tab: NavTab::Avatar,
            catalog: CatalogState {
                sorts: Load::Idle,
                sort_games: HashMap::new(),
                requested_sorts: false,
                requested_content: HashSet::new(),
                thumbnail_urls: HashMap::new(),
                requested_thumbnails: HashSet::new(),
            },
            games: GamesState::default(),
            browse: BrowseState {
                sort_id: String::new(),
                title: String::new(),
                from: Screen::Games,
                selected: 0,
                scroll_pos: 0.0,
                opened_at: Instant::now(),
                closing_at: None,
                card_rects: Vec::new(),
                wheel_accum: 0.0,
                wheel_cooldown: 0.0,
            },
            favorites: FavoritesState {
                games: Load::Idle,
                requested: false,
                from: Screen::Games,
            },
            favorite_states: HashMap::new(),
            favorite_status_requested: HashSet::new(),
            favorite_pending: HashSet::new(),
            friends_scroll: 0.0,
            home_friends_scroll: 0.0,
            scroll_drag: None,
            search: SearchState {
                query: String::new(),
                last_query: String::new(),
                editing: false,
                focus_requested: false,
                results: Load::Idle,
                last_edit: Instant::now(),
                last_submitted: String::new(),
                return_screen: Screen::Games,
            },
            details: DetailsState {
                universe_id: 0,
                root_place_id: 0,
                details: Load::Idle,
                requested: false,
                from: Screen::Home,
                focus_play: true,
                scroll: 0.0,
                lightbox: None,
            },
            settings: SettingsState {
                tab: initial_settings_tab,
                from: Screen::Home,
                drag_slider: None,
            },
            profile: ProfileState {
                user_id: 0,
                display_name: String::new(),
                username: String::new(),
                headshot_url: None,
                presence: "Offline".to_owned(),
                place_id: 0,
                from: Screen::Placeholder,
                bundle: Load::Idle,
                requested: false,
            },
            add_friend: AddFriendState {
                username: String::new(),
                target: None,
                status: None,
                from: Screen::Placeholder,
                editing: false,
                focus_requested: false,
            },
            chat: ChatState {
                conversations: Load::Idle,
                requested: false,
                selected: None,
                messages: Load::Idle,
                messages_for: None,
                start_with: None,
                start_requested: false,
                selected_name: None,
                compose: String::new(),
                editing: false,
                focus_requested: false,
                filter: String::new(),
                status: None,
                show_friends: false,
                list_scroll: 0.0,
                closed_chats: HashSet::new(),
                from: Screen::Home,
            },
            avatar_edit: AvatarEditState {
                category: AvatarCategory::All,
                profile: Load::Idle,
                requested: false,
                worn: Vec::new(),
                emotes: Vec::new(),
                body_colors: None,
                scales: None,
                player_avatar_type: None,
                worn_initialized: false,
                inventory: HashMap::new(),
                selected_type: 0,
                saving: false,
                appearance_saving: false,
                pending_save: false,
                status: None,
                outfits: Vec::new(),
                outfits_for: None,
                outfit_name: String::new(),
                bundles: Load::Idle,
                bundles_requested: false,
                platform_outfits: Load::Idle,
                outfits_requested: false,
                all_requested: false,
                scroll: 0.0,
                body_part: 0,
                body_part_choice: 0,
                body_part_menu: false,
                build_drag: None,
                skin_part: SkinPart::All,
                picker_hue: 0.06,
                picker_sat: 0.55,
                picker_val: 0.90,
                picker_hex: "#DA867A".to_owned(),
                picker_drag: None,
            },
            launch: LaunchState {
                name: String::new(),
                root_place_id: 0,
                started: Instant::now(),
                from: Screen::Details,
                phase: LaunchPhase::Checking,
                check_requested: false,
                sober_account: None,
                session_error: None,
                child: None,
                log: None,
                log_path: None,
                message: None,
                last_poll: Instant::now(),
            },
            gamepad,
            confirm: false,
            toast: None,
            transition: Transition::idle(),
            screen_changed: false,
            api_status: "Idle".to_owned(),
            last_api_error: None,
            sober,
            auth_state: AuthState::SignedOut,
            auth_error: None,
            session: None,
            session_store,
            login_process: None,
            auto_screenshot: None,
            close_after_screenshot: false,
            window_focused: true,
            focus_visible: false,
            avatar_refresh_at: None,
            lock_input,
            demo: if demo {
                Some(DemoRun::new())
            } else if demo_auth {
                Some(DemoRun::auth())
            } else if demo_launch {
                Some(DemoRun::launch())
            } else if demo_play {
                Some(DemoRun::play())
            } else if demo_search_play {
                Some(DemoRun::search_play())
            } else if demo_settings_video {
                Some(DemoRun::settings_video())
            } else {
                None
            },
            started: Instant::now(),
        };
        app.apply_start_options(options);
        if import_sober {
            app.start_sober_import();
        } else {
            app.try_restore_session();
        }
        app
    }

    fn apply_start_options(&mut self, options: StartOptions) {
        if let Some(path) = options.screenshot {
            self.auto_screenshot = Some(AutoScreenshot {
                path,
                deadline: Instant::now() + Duration::from_secs_f32(options.wait.max(0.5)),
                requested: false,
            });
            self.close_after_screenshot = true;
        }

        let Some(screen) = options.screen.as_deref() else {
            if self.close_after_screenshot {
                self.goto(Screen::Home);
            }
            return;
        };

        match screen {
            "boot" => self.goto(Screen::Boot),
            "signin" => self.goto(Screen::SignIn),
            "home" => self.goto(Screen::Home),
            "games" => self.goto(Screen::Games),
            "favorites" => {
                self.favorites.from = Screen::Games;
                self.goto(Screen::Favorites);
            }
            "search" => {
                self.search.query = "natural disaster".to_owned();
                self.search.last_query = self.search.query.clone();
                self.search.last_edit = Instant::now() - Duration::from_secs(2);
                self.nav = NavTab::Games;
                self.goto(Screen::Search);
            }
            "details" => {
                self.details.universe_id = 65241;
                self.details.root_place_id = 189707;
                self.details.from = Screen::Home;
                self.details.focus_play = true;
                self.goto(Screen::Details);
            }
            "browse" => {
                self.browse = BrowseState {
                    sort_id: "top-trending".to_owned(),
                    title: "Top Trending".to_owned(),
                    from: Screen::Games,
                    selected: 0,
                    scroll_pos: 0.0,
                    opened_at: Instant::now(),
                    closing_at: None,
                    card_rects: Vec::new(),
                    wheel_accum: 0.0,
                    wheel_cooldown: 0.0,
                };
                self.goto(Screen::Browse);
            }
            "profile" => {
                self.open_profile(
                    1,
                    "Roblox".to_owned(),
                    "Roblox".to_owned(),
                    None,
                    "Offline".to_owned(),
                    0,
                    Screen::Home,
                );
            }
            "chat" => self.go_tab(crate::screens::NavTab::Chat),
            "chatfriends" => {
                self.chat.show_friends = true;
                self.go_tab(crate::screens::NavTab::Chat);
            }
            "profilelily" => {
                self.open_profile(
                    10149702296,
                    "OwnerxLKRSUP_Lily".to_owned(),
                    "Kim_cutiepotato".to_owned(),
                    None,
                    "Offline".to_owned(),
                    0,
                    Screen::Placeholder,
                );
            }
            "chatjuno" => {
                self.chat.start_with = Some((6191391921, "Juno".to_owned()));
                self.chat.selected_name = Some("Juno".to_owned());
                self.chat.start_requested = false;
                self.chat.selected = None;
                self.go_tab(crate::screens::NavTab::Chat);
            }
            "outfits" => {
                self.avatar_edit.category = AvatarCategory::Avatars;
                self.avatar_edit.selected_type = 0;
                self.placeholder_tab = NavTab::Avatar;
                self.goto(Screen::Placeholder);
            }
            "avatar-all" | "avatar-body" | "avatar-bodyparts" | "avatar-torso"
            | "avatar-clothing" | "avatar-accessories" | "avatar-animations" | "avatar-emotes"
            | "avatar-skin" | "avatar-build" => {
                let category = match screen {
                    "avatar-body" | "avatar-bodyparts" | "avatar-torso" => AvatarCategory::Body,
                    "avatar-clothing" => AvatarCategory::Clothing,
                    "avatar-accessories" => AvatarCategory::Accessories,
                    "avatar-animations" => AvatarCategory::Animations,
                    "avatar-emotes" => AvatarCategory::Emotes,
                    "avatar-skin" | "avatar-build" => AvatarCategory::Body,
                    _ => AvatarCategory::All,
                };
                self.avatar_edit.category = category;
                self.avatar_edit.selected_type = match screen {
                    "avatar-skin" => 1000,
                    "avatar-build" => 1001,
                    "avatar-bodyparts" | "avatar-torso" => 2000,
                    "avatar-all" => 0,
                    _ => category.first_type(),
                };
                if screen == "avatar-bodyparts" {
                    self.avatar_edit.body_part_menu = true;
                } else if screen == "avatar-torso" {
                    self.avatar_edit.body_part = 27;
                }
                self.placeholder_tab = NavTab::Avatar;
                self.goto(Screen::Placeholder);
            }
            "addfriend" => self.open_add_friend(None),
            "settings" => {
                if let Some(name) = options.settings_tab.as_deref() {
                    self.settings.tab =
                        SettingsTab::from_name(name).unwrap_or(SettingsTab::Account);
                    self.config.settings_tab = self.settings.tab.index();
                }
                self.settings.from = Screen::Home;
                self.goto(Screen::Settings);
            }
            "avatar" => {
                self.placeholder_tab = NavTab::Avatar;
                self.goto(Screen::Placeholder);
            }
            "friends" => {
                self.placeholder_tab = NavTab::Friends;
                self.goto(Screen::Placeholder);
            }
            "robux" => {
                self.placeholder_tab = NavTab::Robux;
                self.goto(Screen::Placeholder);
            }
            "launch" => {
                self.open_launch(
                    "Natural Disaster Survival".to_owned(),
                    189707,
                    Screen::Details,
                );
            }
            other => {
                log::warn!("Unknown start screen: {other}");
                self.goto(Screen::Home);
            }
        }
    }

    fn leave_boot(&mut self) {
        if self.auth_state == AuthState::SignedIn && self.session.is_some() {
            self.goto(Screen::Home);
        } else {
            self.goto(Screen::SignIn);
        }
    }

    pub fn goto(&mut self, screen: Screen) {
        if self.screen == screen {
            return;
        }
        log::debug!("screen -> {screen:?}");
        self.transition = Transition {
            start: Instant::now(),
            duration: if self.config.reduced_motion {
                Duration::ZERO
            } else {
                Duration::from_millis(220)
            },
        };
        self.screen = screen;
        self.focus_visible = false;
        self.scroll_drag = None;
        if screen != Screen::Details {
            self.details.lightbox = None;
        }
        self.focus.clear();
        self.registry.clear();
        self.screen_changed = true;
        self.confirm = false;

        match screen {
            Screen::Home => {
                self.nav = NavTab::Home;
                self.focus.set("home:avatar");
            }
            Screen::Games => {
                self.nav = NavTab::Games;
                self.focus.set("nav:GAMES");
            }
            Screen::Favorites => self.focus.set("favorites:back"),
            Screen::SignIn => self.focus.set("signin:signin"),
            Screen::Search => self.focus.set("search:field"),
            Screen::Details => self.focus.set("details:play"),
            Screen::Settings => self.focus.set(self.settings.tab.id()),
            Screen::Launch => self.focus.set("launch:back"),
            Screen::Browse => self.focus.set("browse:back"),
            Screen::Profile => self.focus.set("profile:back"),
            Screen::AddFriend => self.focus.set("addfriend:field"),
            Screen::Chat => {
                self.focus.set("chat:filter");
                self.chat.list_scroll = 0.0;
            }
            Screen::Placeholder => self.focus.set("placeholder:home"),
            Screen::Boot => {}
        }
    }

    pub fn go_tab(&mut self, tab: NavTab) {
        if tab == NavTab::Settings {
            self.open_settings();
            return;
        }
        self.nav = tab;
        self.placeholder_tab = tab;
        if tab == NavTab::Chat {
            self.chat.from = Screen::Home;
        }
        self.goto(tab.screen());
    }

    pub fn open_settings(&mut self) {
        if self.screen == Screen::Settings {
            return;
        }
        self.settings.from = self.screen;
        self.settings.tab = SettingsTab::from_index(self.config.settings_tab);
        self.nav = NavTab::Settings;
        self.goto(Screen::Settings);
        self.focus.set(self.settings.tab.id());
    }

    pub fn open_search(&mut self) {
        if self.screen == Screen::Search {
            self.search.editing = true;
            self.search.focus_requested = true;
            self.focus.set("search:field");
            return;
        }
        if matches!(
            self.screen,
            Screen::Home | Screen::Games | Screen::Placeholder
        ) {
            self.search.return_screen = self.screen;
            self.nav = NavTab::Games;
            self.goto(Screen::Search);
            self.search.editing = true;
            self.search.focus_requested = true;
            self.focus.set("search:field");
        }
    }

    pub fn open_browse(&mut self, sort_id: String, title: String, from: Screen) {
        self.browse = BrowseState {
            sort_id,
            title,
            from,
            selected: 0,
            scroll_pos: 0.0,
            opened_at: Instant::now(),
            closing_at: None,
            card_rects: Vec::new(),
            wheel_accum: 0.0,
            wheel_cooldown: 0.0,
        };
        self.music.play_sfx(Sfx::Slide, &self.paths, &self.config);
        self.goto(Screen::Browse);
    }

    pub fn start_browse_close(&mut self) {
        if self.browse.closing_at.is_some() {
            return;
        }
        self.browse.closing_at = Some(Instant::now());
        self.music
            .play_sfx(Sfx::SlideOut, &self.paths, &self.config);
    }

    pub fn browse_move(&mut self, forward: bool) {
        let length = self
            .catalog
            .sort_games
            .get(&self.browse.sort_id)
            .and_then(Load::ready)
            .map(Vec::len)
            .unwrap_or(0);
        if length == 0 {
            return;
        }
        let selected = self.browse.selected.min(length - 1);
        self.browse.selected = if forward {
            (selected + 1) % length
        } else {
            (selected + length - 1) % length
        };
        self.focus
            .set(format!("browse:card:{}", self.browse.selected));
    }

    pub fn open_profile(
        &mut self,
        user_id: u64,
        display_name: String,
        username: String,
        headshot_url: Option<String>,
        presence: String,
        place_id: u64,
        from: Screen,
    ) {
        self.profile = ProfileState {
            user_id,
            display_name,
            username,
            headshot_url,
            presence,
            place_id,
            from,
            bundle: Load::Idle,
            requested: false,
        };
        self.goto(Screen::Profile);
    }

    pub fn open_chat_with(
        &mut self,
        user_id: u64,
        display_name: String,
        _username: String,
        _headshot_url: Option<String>,
        from: Screen,
    ) {
        self.chat.start_with = Some((user_id, display_name.clone()));
        self.chat.selected_name = Some(display_name);
        self.chat.start_requested = false;
        self.chat.selected = None;
        self.chat.messages = Load::Idle;
        self.chat.messages_for = None;
        self.chat.status = None;
        self.chat.show_friends = false;
        self.chat.list_scroll = 0.0;
        self.chat.from = from;
        self.goto(Screen::Chat);
    }

    pub fn open_add_friend(&mut self, target: Option<(u64, String)>) {
        let username = target
            .as_ref()
            .map(|(_, name)| name.clone())
            .unwrap_or_default();
        let editing = target.is_none();
        self.add_friend = AddFriendState {
            username,
            target,
            status: None,
            from: self.screen,
            editing,
            focus_requested: editing,
        };
        self.goto(Screen::AddFriend);
    }

    pub fn open_details(&mut self, universe_id: u64, root_place_id: u64, from: Screen) {
        self.details = DetailsState {
            universe_id,
            root_place_id,
            details: Load::Idle,
            requested: false,
            from,
            focus_play: true,
            scroll: 0.0,
            lightbox: None,
        };
        self.goto(Screen::Details);
    }

    pub fn open_favorites(&mut self) {
        self.favorites.games = Load::Idle;
        self.favorites.requested = false;
        self.favorites.from = Screen::Games;
        self.goto(Screen::Favorites);
    }

    pub fn ensure_favorite_status(&mut self, universe_id: u64) {
        if universe_id == 0
            || self.favorite_states.contains_key(&universe_id)
            || self.favorite_status_requested.contains(&universe_id)
        {
            return;
        }
        let Some(token) = self
            .session
            .as_ref()
            .map(|session| session.token.expose_secret().to_owned())
        else {
            return;
        };
        self.favorite_status_requested.insert(universe_id);
        self.api
            .load_favorite_status(token, universe_id, self.api_tx.clone());
    }

    pub fn toggle_favorite(&mut self, universe_id: u64) {
        if universe_id == 0 || self.favorite_pending.contains(&universe_id) {
            return;
        }
        let Some(token) = self
            .session
            .as_ref()
            .map(|session| session.token.expose_secret().to_owned())
        else {
            self.toast("Sign in to favorite experiences.");
            return;
        };
        let target = !self
            .favorite_states
            .get(&universe_id)
            .copied()
            .unwrap_or(false);
        self.favorite_pending.insert(universe_id);
        self.api
            .set_favorite(token, universe_id, target, self.api_tx.clone());
    }

    fn ensure_favorites(&mut self) {
        if self.favorites.requested {
            return;
        }
        let Some(user_id) = self.session.as_ref().map(|session| session.user.id) else {
            self.favorites.games = Load::Ready(Vec::new());
            return;
        };
        self.favorites.requested = true;
        self.favorites.games = Load::Loading;
        self.api.load_favorites(user_id, self.api_tx.clone());
    }

    pub fn open_launch(&mut self, name: String, root_place_id: u64, from: Screen) {
        self.launch = LaunchState {
            name,
            root_place_id,
            started: Instant::now(),
            from,
            phase: LaunchPhase::Checking,
            check_requested: false,
            sober_account: None,
            session_error: None,
            child: None,
            log: None,
            log_path: None,
            message: None,
            last_poll: Instant::now(),
        };
        self.goto(Screen::Launch);
    }

    pub fn try_restore_session(&mut self) {
        match self.session_store.load() {
            Ok(Some(loaded)) => {
                self.auth_state = AuthState::Checking;
                self.auth_error = None;
                self.api.load_account(loaded.token, self.api_tx.clone());
            }
            Ok(None) => {
                self.auth_state = AuthState::SignedOut;
            }
            Err(error) => {
                log::warn!("Could not load the saved session: {error}");
                self.auth_error = Some(error);
                self.auth_state = AuthState::SignedOut;
            }
        }
    }

    pub fn start_sober_import(&mut self) {
        if self.auth_state == AuthState::SigningIn {
            return;
        }
        match auth::read_sober_token(&self.paths) {
            Ok(token) => {
                self.auth_state = AuthState::SigningIn;
                self.auth_error = None;
                self.toast_silent("Validating the Sober session...");
                self.api.load_account(token, self.api_tx.clone());
            }
            Err(error) => {
                self.auth_state = AuthState::SignedOut;
                self.auth_error = Some(error.clone());
                self.toast(error);
            }
        }
    }

    pub fn start_webview_login(&mut self) {
        if self.login_process.is_some() || self.auth_state == AuthState::SigningIn {
            return;
        }
        match auth::spawn_login_helper(&self.paths) {
            Ok(process) => {
                self.login_process = Some(process);
                self.auth_state = AuthState::SigningIn;
                self.auth_error = None;
                self.toast_silent("Complete sign-in in the Roblox window.");
            }
            Err(error) => {
                self.auth_state = AuthState::SignedOut;
                self.auth_error = Some(error.clone());
                self.toast(error);
            }
        }
    }

    fn poll_login_process(&mut self) {
        let Some(process) = &mut self.login_process else {
            return;
        };
        let status = match process.child.try_wait() {
            Ok(Some(status)) => status,
            Ok(None) => return,
            Err(error) => {
                self.login_process = None;
                self.auth_state = AuthState::SignedOut;
                self.toast(format!("Sign-in window error: {error}"));
                return;
            }
        };
        let process = self.login_process.take().expect("login process");
        let token = std::fs::read_to_string(&process.out)
            .ok()
            .map(|text| text.trim().to_owned())
            .filter(|token| !token.is_empty());
        if status.success()
            && let Some(token) = token
        {
            self.api.load_account(token, self.api_tx.clone());
        } else {
            self.auth_state = AuthState::SignedOut;
            self.toast_silent("Sign-in cancelled.");
        }
    }

    pub fn sign_out(&mut self) {
        self.session = None;
        self.auth_state = AuthState::SignedOut;
        self.auth_error = None;
        self.login_process = None;
        self.session_store.clear();
        self.favorite_states.clear();
        self.favorite_status_requested.clear();
        self.favorite_pending.clear();
        self.favorites.requested = false;
        self.favorites.games = Load::Idle;
        self.toast_silent("Signed out.");
        self.goto(Screen::SignIn);
    }

    pub fn load_account_data(&mut self) {
        let Some(session) = &self.session else {
            return;
        };
        let token = session.token.expose_secret().to_owned();
        let user_id = session.user.id;
        self.api
            .load_user_headshots(vec![user_id], self.api_tx.clone());
        self.api.load_user_avatar(user_id, self.api_tx.clone());
        self.api
            .load_friends(token.clone(), user_id, self.api_tx.clone());
        self.api.load_robux(token, user_id, self.api_tx.clone());
    }

    fn ensure_launch(&mut self) {
        match self.launch.phase {
            LaunchPhase::Checking => {
                if self.auth_state == AuthState::Checking {
                    return;
                }
                if !self.launch.check_requested {
                    self.launch.check_requested = true;
                    match auth::read_sober_token(&self.paths) {
                        Ok(token) => self.api.check_account(token, self.api_tx.clone()),
                        Err(error) => {
                            self.launch.session_error = Some(error);
                            self.launch.phase = LaunchPhase::SessionUnverified;
                        }
                    }
                }
            }
            LaunchPhase::Launching => {
                if self.launch.child.is_none() {
                    self.start_launch_process();
                }
            }
            LaunchPhase::Waiting => {
                if self.launch.last_poll.elapsed() >= Duration::from_millis(500) {
                    self.launch.last_poll = Instant::now();
                    self.poll_launch();
                }
            }
            _ => {}
        }
    }

    fn start_launch_process(&mut self) {
        let runtime = if self.sober.installed {
            LaunchRuntime::Sober
        } else if self.sober.cordial_installed {
            LaunchRuntime::Cordial
        } else {
            LaunchRuntime::None
        };
        if runtime == LaunchRuntime::None {
            self.launch.phase = LaunchPhase::Failed;
            self.launch.message =
                Some("No Roblox runtime is installed. Install Sober or Cordial.".to_owned());
            return;
        }

        self.music.play_sfx(Sfx::Launch, &self.paths, &self.config);
        log::info!(
            "launching place {} with {:?}",
            self.launch.root_place_id,
            runtime
        );
        let log = LaunchLog::snapshot(&self.paths.sober_log_dir());
        self.launch.log_path = Some(log.path().to_path_buf());
        self.launch.log = Some(log);

        match launch::spawn_launch(self.launch.root_place_id, runtime) {
            Ok(child) => {
                self.launch.child = Some(child);
                self.launch.phase = LaunchPhase::Waiting;
                self.launch.started = Instant::now();
                self.launch.last_poll = Instant::now();
                self.launch.message = None;
            }
            Err(error) => {
                self.launch.phase = LaunchPhase::Failed;
                self.launch.message = Some(error);
            }
        }
    }

    /// Watch the runtime process even when the user has left the Launch
    /// screen. Without this, the menu would stay muted forever after a game
    /// exits while the shell is showing Home or Games.
    fn poll_game_process(&mut self) {
        let mut result = None;
        if let Some(child) = &mut self.launch.child {
            match child.try_wait() {
                Ok(Some(status)) => result = Some(status.success()),
                Ok(None) => {}
                Err(error) => log::warn!("Could not check the Sober process: {error}"),
            }
        }
        let Some(success) = result else {
            return;
        };
        self.launch.child = None;
        if !success
            && matches!(
                self.launch.phase,
                LaunchPhase::Launching | LaunchPhase::Waiting
            )
        {
            self.launch.phase = LaunchPhase::Failed;
            self.launch.message = Some(
                "Sober exited before the join was confirmed. Check that Sober is installed and signed in."
                    .to_owned(),
            );
        }
    }

    fn poll_launch(&mut self) {
        let mut exited = false;
        let mut failed = false;
        if let Some(child) = &mut self.launch.child {
            match child.try_wait() {
                Ok(Some(status)) if !status.success() => failed = true,
                Ok(Some(_)) => exited = true,
                Ok(None) => {}
                Err(error) => log::warn!("Could not check the Sober process: {error}"),
            }
        }
        if failed {
            self.launch.child = None;
            self.launch.phase = LaunchPhase::Failed;
            self.launch.message = Some(
                "Sober exited before the join was confirmed. Check that Sober is installed and signed in."
                    .to_owned(),
            );
            return;
        }
        if exited {
            self.launch.child = None;
        }

        let joined = self
            .launch
            .log
            .as_mut()
            .is_some_and(|log| log.poll(self.launch.root_place_id));
        if let Some(log) = &self.launch.log {
            self.launch.log_path = Some(log.path().to_path_buf());
        }
        let crashed = self.launch.log.as_ref().is_some_and(|log| log.crashed());
        if crashed {
            log::warn!(
                "Sober crashed while loading place {}",
                self.launch.root_place_id
            );
            self.launch.phase = LaunchPhase::Failed;
            self.launch.message = Some(
                "Sober crashed while loading the experience. This is usually GPU memory pressure or a Vulkan device error. Close other GPU-heavy apps and retry."
                    .to_owned(),
            );
            return;
        }
        if joined {
            log::info!("launch joined place {}", self.launch.root_place_id);
            self.launch.phase = LaunchPhase::Joined;
            self.launch.message = Some("Sober confirmed the join.".to_owned());
            return;
        }

        if self.launch.started.elapsed() >= Duration::from_secs(90) {
            log::warn!("launch timed out for place {}", self.launch.root_place_id);
            self.launch.phase = LaunchPhase::Failed;
            self.launch.message = Some(
                "Sober launched, but no join was confirmed within 90 seconds. Check the Sober log."
                    .to_owned(),
            );
        }
    }

    pub fn retry_launch(&mut self) {
        self.launch.phase = LaunchPhase::Checking;
        self.launch.check_requested = false;
        self.launch.sober_account = None;
        self.launch.session_error = None;
        self.launch.child = None;
        self.launch.log = None;
        self.launch.log_path = None;
        self.launch.message = None;
        self.launch.last_poll = Instant::now();
    }

    pub fn launch_anyway(&mut self) {
        self.launch.phase = LaunchPhase::Launching;
        self.launch.session_error = None;
        self.launch.message = None;
    }

    pub fn open_sober(&mut self) {
        if let Err(error) = launch::open_sober() {
            self.toast(error);
        }
    }

    pub fn leave_launch(&mut self) {
        self.goto(self.launch.from);
    }

    /// True while the launched runtime is still open. The menu soundtrack is
    /// suppressed for the whole session so it cannot play over the game when
    /// the shell is behind the Sober window.
    fn game_running(&self) -> bool {
        self.launch.child.is_some()
            && matches!(
                self.launch.phase,
                LaunchPhase::Launching | LaunchPhase::Waiting | LaunchPhase::Joined
            )
    }

    pub fn toast(&mut self, message: impl Into<String>) {
        self.toast_with_sound(message, true);
    }

    /// Status messages such as "signed in" or "settings saved" should not
    /// interrupt the music with a notification ping.
    pub fn toast_silent(&mut self, message: impl Into<String>) {
        self.toast_with_sound(message, false);
    }

    fn toast_with_sound(&mut self, message: impl Into<String>, sound: bool) {
        if sound {
            self.music.play_sfx(Sfx::Notice, &self.paths, &self.config);
        }
        self.toast = Some(Toast {
            message: message.into(),
            created: Instant::now(),
            duration: Duration::from_secs(4),
        });
    }

    pub fn ensure_sorts(&mut self) {
        if !self.catalog.requested_sorts && self.catalog.sorts.is_idle() {
            self.catalog.requested_sorts = true;
            self.catalog.sorts = Load::Loading;
            self.api_status = "Loading catalog".to_owned();
            self.api.load_sorts(self.api_tx.clone());
        }
    }

    pub fn ensure_sort_content(&mut self, sort_id: &str) {
        let sort_id = sort_id.to_owned();
        if !self.catalog.requested_content.contains(&sort_id) {
            self.catalog.requested_content.insert(sort_id.clone());
            self.catalog
                .sort_games
                .insert(sort_id.clone(), Load::Loading);
            self.api.load_sort_content(sort_id, self.api_tx.clone());
        }
    }

    pub fn retry_sorts(&mut self) {
        self.catalog.requested_sorts = false;
        self.catalog.sorts = Load::Idle;
        self.ensure_sorts();
    }

    pub fn retry_sort_content(&mut self, sort_id: &str) {
        self.catalog.requested_content.remove(sort_id);
        self.catalog.sort_games.remove(sort_id);
        self.ensure_sort_content(sort_id);
    }

    pub fn ensure_thumbnails_for(&mut self, games: &[GameSummary]) {
        let ids: Vec<u64> = games
            .iter()
            .map(|game| game.universe_id)
            .filter(|id| {
                !self.catalog.requested_thumbnails.contains(id)
                    && !self.catalog.thumbnail_urls.contains_key(id)
            })
            .take(50)
            .collect();
        if ids.is_empty() {
            return;
        }
        for id in &ids {
            self.catalog.requested_thumbnails.insert(*id);
        }
        self.api.load_thumbnails(ids, self.api_tx.clone());
    }

    pub fn thumbnail(&mut self, universe_id: u64) -> Option<&TextureHandle> {
        let url = self.catalog.thumbnail_urls.get(&universe_id)?.clone();
        self.images.ensure(&url);
        self.images.get(&url)
    }

    pub fn ensure_details(&mut self) {
        if !self.details.requested && self.details.universe_id > 0 {
            self.details.requested = true;
            self.details.details = Load::Loading;
            self.api_status = "Loading experience".to_owned();
            self.api
                .load_details(self.details.universe_id, self.api_tx.clone());
        }
        self.ensure_favorite_status(self.details.universe_id);
    }

    pub fn retry_details(&mut self) {
        self.details.requested = false;
        self.details.details = Load::Idle;
        self.ensure_details();
    }

    pub fn submit_search(&mut self) {
        let query = self.search.query.trim().to_owned();
        if query.is_empty() {
            self.search.results = Load::Idle;
            self.search.last_submitted.clear();
            return;
        }
        self.search.last_submitted = self.search.query.clone();
        self.search.results = Load::Loading;
        self.api_status = "Searching".to_owned();
        self.api.search(query, self.api_tx.clone());
    }

    fn maybe_search(&mut self) {
        if self.search.query.trim().is_empty() {
            if !self.search.results.is_idle() {
                self.search.results = Load::Idle;
            }
            return;
        }
        if self.search.query != self.search.last_submitted
            && self.search.last_edit.elapsed() >= Duration::from_millis(350)
        {
            self.submit_search();
        }
    }

    fn ensure_screen_data(&mut self) {
        match self.screen {
            Screen::Home | Screen::Games => {
                self.ensure_sorts();
                let ids: Vec<String> = if let Load::Ready(sorts) = &self.catalog.sorts {
                    if self.screen == Screen::Games {
                        // The rail now includes all genre sorts. Load the
                        // original carousel rows eagerly and fetch genre rows
                        // only as they come near the viewport, so a single
                        // startup does not fire nineteen content requests.
                        let step = crate::screens::games::COLUMN_STEP;
                        let first = (self.games.scroll / step).floor() as usize;
                        let last = ((self.games.scroll + crate::screens::games::VIEWPORT_WIDTH)
                            / step)
                            .ceil() as usize;
                        sorts
                            .iter()
                            .enumerate()
                            .filter(|(index, _)| {
                                *index < 5
                                    || (*index >= first.saturating_sub(1) && *index <= last + 1)
                            })
                            .map(|(_, sort)| sort.sort_id.clone())
                            .collect()
                    } else {
                        sorts
                            .iter()
                            .take(1)
                            .map(|sort| sort.sort_id.clone())
                            .collect()
                    }
                } else {
                    Vec::new()
                };
                for id in ids {
                    self.ensure_sort_content(&id);
                }
            }
            Screen::Search => self.maybe_search(),
            Screen::Favorites => self.ensure_favorites(),
            Screen::Details => self.ensure_details(),
            Screen::Launch => self.ensure_launch(),
            Screen::Browse => {
                let sort_id = self.browse.sort_id.clone();
                if !sort_id.is_empty() {
                    self.ensure_sort_content(&sort_id);
                }
            }
            Screen::Profile => {
                if !self.profile.requested && self.profile.user_id > 0 {
                    self.profile.requested = true;
                    self.profile.bundle = Load::Loading;
                    self.api
                        .load_profile(self.profile.user_id, self.api_tx.clone());
                }
            }
            Screen::Chat => self.ensure_chat(),
            Screen::Placeholder if self.placeholder_tab == NavTab::Avatar => {
                self.ensure_avatar_edit();
            }
            _ => {}
        }
    }

    fn ensure_avatar_edit(&mut self) {
        let Some(session) = &self.session else {
            return;
        };
        let user_id = session.user.id;
        let token = session.token.expose_secret().to_owned();
        if !self.avatar_edit.requested {
            self.avatar_edit.requested = true;
            if self.avatar_edit.profile.is_idle() {
                self.avatar_edit.profile = Load::Loading;
                self.api.load_profile(user_id, self.api_tx.clone());
            }
        }
        let category = self.avatar_edit.category;
        let selected_type = self.avatar_edit.selected_type;
        let body_part = self.avatar_edit.body_part;
        let mut load_type = |asset_type: u64| {
            if asset_type > 0
                && asset_type < 1000
                && !self.avatar_edit.inventory.contains_key(&asset_type)
            {
                self.avatar_edit.inventory.insert(asset_type, Load::Loading);
                self.api
                    .load_inventory(token.clone(), user_id, asset_type, self.api_tx.clone());
            }
        };
        if category == AvatarCategory::All {
            for asset_type in AVATAR_INVENTORY_TYPES {
                load_type(*asset_type);
            }
            self.avatar_edit.all_requested = true;
        } else if category == AvatarCategory::Avatars {
            if !self.avatar_edit.outfits_requested {
                self.avatar_edit.outfits_requested = true;
                self.avatar_edit.platform_outfits = Load::Loading;
                self.api
                    .load_platform_outfits(token.clone(), user_id, self.api_tx.clone());
            }
        } else if category == AvatarCategory::Body && selected_type == 2000 {
            if !self.avatar_edit.bundles_requested {
                self.avatar_edit.bundles_requested = true;
                self.avatar_edit.bundles = Load::Loading;
                self.api.load_avatar_bundles(user_id, self.api_tx.clone());
            }
            if body_part > 0 {
                load_type(body_part);
            }
        } else if category == AvatarCategory::Animations && selected_type == 2001 {
            if !self.avatar_edit.bundles_requested {
                self.avatar_edit.bundles_requested = true;
                self.avatar_edit.bundles = Load::Loading;
                self.api.load_avatar_bundles(user_id, self.api_tx.clone());
            }
        } else if selected_type < 1000 && selected_type != 2002 {
            load_type(selected_type);
        }
        if self.avatar_edit.outfits_for != Some(user_id) {
            self.avatar_edit.outfits = load_outfits(&self.paths, user_id);
            self.avatar_edit.outfits_for = Some(user_id);
        }
    }

    pub fn toggle_avatar_asset(&mut self, item: &InventoryItem) {
        if item.asset_type == 61 {
            self.toggle_emote(item.asset_id);
            return;
        }
        if let Some(index) =
            self.avatar_edit.worn.iter().position(|asset| {
                asset.id == item.asset_id && asset.asset_type.id == item.asset_type
            })
        {
            self.avatar_edit.worn.remove(index);
        } else {
            self.avatar_edit
                .worn
                .retain(|asset| asset.asset_type.id != item.asset_type);
            self.avatar_edit.worn.push(AvatarAsset {
                id: item.asset_id,
                name: item.name.clone(),
                asset_type: AvatarAssetType {
                    id: item.asset_type,
                    name: String::new(),
                },
            });
        }
        self.avatar_edit.status = None;
        // The 2016 editor applied each change immediately. Keep the local
        // selection responsive and serialize the writes so a quick second
        // click is queued behind the first request instead of being dropped.
        self.queue_avatar_save();
    }

    fn toggle_emote(&mut self, asset_id: u64) {
        if self.avatar_edit.appearance_saving {
            return;
        }
        let Some(token) = self
            .session
            .as_ref()
            .map(|session| session.token.expose_secret().to_owned())
        else {
            self.avatar_edit.status = Some(("Sign in to change your emotes.".to_owned(), true));
            return;
        };
        self.avatar_edit.appearance_saving = true;
        if let Some(equipped) = self
            .avatar_edit
            .emotes
            .iter()
            .find(|emote| emote.asset_id == asset_id)
        {
            self.avatar_edit.status = Some(("Removing emote...".to_owned(), false));
            self.api
                .remove_emote(token, equipped.position, self.api_tx.clone());
            return;
        }
        let used: Vec<u64> = self
            .avatar_edit
            .emotes
            .iter()
            .map(|emote| emote.position)
            .collect();
        let Some(position) = (1..=8).find(|position| !used.contains(position)) else {
            self.avatar_edit.appearance_saving = false;
            self.avatar_edit.status = Some(("All emote slots are full.".to_owned(), true));
            return;
        };
        self.avatar_edit.status = Some(("Equipping emote...".to_owned(), false));
        self.api
            .set_emote(token, asset_id, position, self.api_tx.clone());
    }

    pub(crate) fn queue_avatar_save(&mut self) {
        self.avatar_edit.pending_save = true;
        if !self.avatar_edit.saving {
            self.save_avatar();
        }
    }

    pub fn save_avatar(&mut self) {
        if self.avatar_edit.saving {
            self.avatar_edit.pending_save = true;
            return;
        }
        let Some(token) = self
            .session
            .as_ref()
            .map(|session| session.token.expose_secret().to_owned())
        else {
            self.avatar_edit.status = Some(("Sign in to edit your avatar.".to_owned(), true));
            return;
        };
        self.avatar_edit.saving = true;
        self.avatar_edit.pending_save = false;
        self.avatar_edit.status = Some(("Saving avatar...".to_owned(), false));
        self.api
            .set_avatar(token, self.avatar_edit.worn.clone(), self.api_tx.clone());
    }

    pub fn apply_skin_color(&mut self, brick_color_id: u64) {
        if self.avatar_edit.appearance_saving {
            return;
        }
        let Some(token) = self
            .session
            .as_ref()
            .map(|session| session.token.expose_secret().to_owned())
        else {
            self.avatar_edit.status = Some(("Sign in to change your skin tone.".to_owned(), true));
            return;
        };
        let mut colors = self.avatar_edit.body_colors.clone().unwrap_or(BodyColors {
            head: 194,
            torso: 194,
            right_arm: 194,
            left_arm: 194,
            right_leg: 194,
            left_leg: 194,
        });
        match self.avatar_edit.skin_part {
            SkinPart::All => {
                colors.head = brick_color_id;
                colors.torso = brick_color_id;
                colors.right_arm = brick_color_id;
                colors.left_arm = brick_color_id;
                colors.right_leg = brick_color_id;
                colors.left_leg = brick_color_id;
            }
            SkinPart::Head => colors.head = brick_color_id,
            SkinPart::Torso => colors.torso = brick_color_id,
            SkinPart::LeftArm => colors.left_arm = brick_color_id,
            SkinPart::RightArm => colors.right_arm = brick_color_id,
            SkinPart::LeftLeg => colors.left_leg = brick_color_id,
            SkinPart::RightLeg => colors.right_leg = brick_color_id,
        }
        self.avatar_edit.body_colors = Some(colors.clone());
        self.avatar_edit.appearance_saving = true;
        self.avatar_edit.status = Some(("Applying skin tone...".to_owned(), false));
        self.api.set_body_colors(token, colors, self.api_tx.clone());
    }

    pub fn set_avatar_type(&mut self, avatar_type: &str) {
        if self.avatar_edit.appearance_saving {
            return;
        }
        let Some(token) = self
            .session
            .as_ref()
            .map(|session| session.token.expose_secret().to_owned())
        else {
            self.avatar_edit.status = Some(("Sign in to change avatar type.".to_owned(), true));
            return;
        };
        self.avatar_edit.appearance_saving = true;
        self.avatar_edit.status = Some((format!("Switching to {avatar_type}..."), false));
        self.api
            .set_avatar_type(token, avatar_type.to_owned(), self.api_tx.clone());
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_build(
        &mut self,
        height: f64,
        width: f64,
        head: f64,
        depth: f64,
        proportion: f64,
        body_type: f64,
    ) {
        if self.avatar_edit.appearance_saving {
            return;
        }
        let Some(token) = self
            .session
            .as_ref()
            .map(|session| session.token.expose_secret().to_owned())
        else {
            self.avatar_edit.status = Some(("Sign in to change your build.".to_owned(), true));
            return;
        };
        self.avatar_edit.appearance_saving = true;
        self.avatar_edit.status = Some(("Applying build...".to_owned(), false));
        self.api.set_scales(
            token,
            height,
            width,
            head,
            depth,
            proportion,
            body_type,
            self.api_tx.clone(),
        );
    }

    pub fn save_current_outfit(&mut self) {
        let Some(user_id) = self.session.as_ref().map(|session| session.user.id) else {
            self.avatar_edit.status = Some(("Sign in to save outfits.".to_owned(), true));
            return;
        };
        if self.avatar_edit.worn.is_empty() {
            self.avatar_edit.status = Some((
                "Choose at least one item before saving an outfit.".to_owned(),
                true,
            ));
            return;
        }
        let name = self.avatar_edit.outfit_name.trim().to_owned();
        let name = if name.is_empty() {
            format!("Outfit {}", self.avatar_edit.outfits.len() + 1)
        } else {
            name
        };
        self.avatar_edit.outfits.push(SavedOutfit {
            name: name.clone(),
            assets: self.avatar_edit.worn.clone(),
        });
        self.avatar_edit.outfit_name.clear();
        match write_outfits(&self.paths, user_id, &self.avatar_edit.outfits) {
            Ok(()) => {
                self.avatar_edit.status = Some((format!("Saved outfit \"{name}\"."), false));
            }
            Err(error) => {
                self.avatar_edit.status = Some((format!("Could not save outfit: {error}"), true));
            }
        }
    }

    pub fn equip_outfit(&mut self, index: usize) {
        let Some(outfit) = self.avatar_edit.outfits.get(index).cloned() else {
            return;
        };
        self.avatar_edit.worn = outfit.assets;
        self.avatar_edit.status = Some((format!("Equipping \"{}\"...", outfit.name), false));
        self.queue_avatar_save();
    }

    fn ensure_chat(&mut self) {
        let Some((token, owner_id)) = self
            .session
            .as_ref()
            .map(|session| (session.token.expose_secret().to_owned(), session.user.id))
        else {
            return;
        };
        if !self.chat.requested {
            self.chat.requested = true;
            self.chat.conversations = Load::Loading;
            self.api
                .load_chat_conversations(token.clone(), self.api_tx.clone());
        }
        if let Some((user_id, _)) = self.chat.start_with.clone()
            && let Load::Ready(conversations) = &self.chat.conversations
        {
            if let Some(conversation) = conversations.iter().find(|conversation| {
                !conversation.id.is_empty() && conversation.participant_user_ids.contains(&user_id)
            }) {
                self.chat.selected = Some(conversation.id.clone());
                self.chat.start_with = None;
            } else if !self.chat.start_requested {
                self.chat.start_requested = true;
                self.api
                    .start_chat(token.clone(), owner_id, user_id, self.api_tx.clone());
            }
        }
        if let Some(conversation_id) = self.chat.selected.clone()
            && self.chat.messages_for.as_deref() != Some(conversation_id.as_str())
        {
            self.chat.messages_for = Some(conversation_id.clone());
            self.chat.messages = Load::Loading;
            self.api
                .load_chat_messages(token, conversation_id, self.api_tx.clone());
        }
    }

    fn poll_api(&mut self) {
        while let Ok(event) = self.api_rx.try_recv() {
            match event {
                ApiEvent::Sorts(result) => {
                    self.update_status(&result);
                    self.catalog.sorts = crate::api::load_state_from_result(result);
                }
                ApiEvent::SortContent { sort_id, result } => {
                    self.update_status(&result);
                    self.catalog
                        .sort_games
                        .insert(sort_id, crate::api::load_state_from_result(result));
                }
                ApiEvent::Thumbnails { ids, result } => match result {
                    Ok(urls) => {
                        self.api_status = "OK".to_owned();
                        self.catalog.thumbnail_urls.extend(urls);
                    }
                    Err(error) => {
                        self.api_status = format!("Thumbnail error: {}", error.user_message());
                        self.last_api_error = Some(error.detail());
                        for id in ids {
                            self.catalog.requested_thumbnails.remove(&id);
                        }
                    }
                },
                ApiEvent::Search { query, result } => {
                    if query == self.search.query.trim() {
                        self.update_status(&result);
                        self.search.results = crate::api::load_state_from_result(result);
                    }
                }
                ApiEvent::Details {
                    universe_id,
                    result,
                } => {
                    if universe_id == self.details.universe_id {
                        self.update_status(&result);
                        self.details.details = crate::api::load_state_from_result(result);
                    }
                }
                ApiEvent::PlaceResolved { result, .. } => {
                    self.update_status(&result);
                }
                ApiEvent::Authenticated { token, result } => match result {
                    Ok(user) => {
                        log::info!("session validated for user {}", user.id);
                        let storage = self.session_store.save(&token);
                        let session = Session {
                            token: SecretString::from(token),
                            user,
                            storage,
                            headshots: HashMap::new(),
                            avatar: None,
                            friends: Load::Idle,
                            presence: HashMap::new(),
                            robux: Load::Idle,
                        };
                        let display = session.user.display().to_owned();
                        let was_on_signin = self.screen == Screen::SignIn;
                        self.api_status = "OK".to_owned();
                        self.session = Some(session);
                        self.auth_state = AuthState::SignedIn;
                        self.auth_error = None;
                        self.load_account_data();
                        // Never pull the user out of the title screen. The title
                        // waits for an explicit button press even when a saved
                        // session was restored in the background.
                        if was_on_signin {
                            self.goto(Screen::Home);
                            self.toast_silent(format!("Signed in as {display}"));
                        } else {
                            log::info!("session restored while on {:?}; staying put", self.screen);
                        }
                    }
                    Err(error) => {
                        log::warn!("session validation failed: {}", error.user_message());
                        let unauthorized = matches!(error, ApiError::Http { status: 401, .. });
                        if unauthorized {
                            self.session_store.clear();
                            self.session = None;
                            self.auth_state = AuthState::SignedOut;
                            self.toast("The saved session expired. Sign in again.");
                        } else {
                            let message = error.user_message();
                            self.auth_state = AuthState::SignedOut;
                            self.auth_error = Some(message.clone());
                            self.toast(message);
                        }
                    }
                },
                ApiEvent::Friends { user_id, result } => {
                    if self.session.as_ref().map(|session| session.user.id) != Some(user_id) {
                        continue;
                    }
                    match result {
                        Ok(friends) => {
                            log::info!("friends loaded: {}", friends.len());
                            let ids: Vec<u64> =
                                friends.iter().take(100).map(|friend| friend.id).collect();
                            if let Some(session) = &mut self.session {
                                session.friends = Load::Ready(friends);
                            }
                            let token = self
                                .session
                                .as_ref()
                                .map(|session| session.token.expose_secret().to_owned())
                                .unwrap_or_default();
                            if !ids.is_empty() {
                                self.api
                                    .load_user_headshots(ids.clone(), self.api_tx.clone());
                                self.api
                                    .load_presence(token, user_id, ids, self.api_tx.clone());
                            }
                        }
                        Err(error) => {
                            if let Some(session) = &mut self.session {
                                session.friends = Load::Failed(error.user_message());
                            }
                        }
                    }
                }
                ApiEvent::Presence { user_id, result } => {
                    if self.session.as_ref().map(|session| session.user.id) != Some(user_id) {
                        continue;
                    }
                    match result {
                        Ok(presence) => {
                            log::info!("presence entries loaded: {}", presence.len());
                            if let Some(session) = &mut self.session {
                                session.presence = presence;
                            }
                        }
                        Err(error) => {
                            log::warn!("presence request failed: {}", error.user_message());
                        }
                    }
                }
                ApiEvent::Robux { user_id, result } => {
                    if self.session.as_ref().map(|session| session.user.id) != Some(user_id) {
                        continue;
                    }
                    if let Some(session) = &mut self.session {
                        session.robux = match result {
                            Ok(robux) => {
                                log::info!("robux balance loaded");
                                Load::Ready(robux)
                            }
                            Err(error) => Load::Failed(error.user_message()),
                        };
                    }
                }
                ApiEvent::UserHeadshots { result } => {
                    if let Ok(headshots) = result {
                        log::info!("user headshots loaded: {}", headshots.len());
                        if let Some(session) = &mut self.session {
                            session.headshots.extend(headshots);
                        }
                    }
                }
                ApiEvent::UserAvatar { user_id, result } => match result {
                    Ok(url) => {
                        log::info!("full-body avatar loaded");
                        if let Some(session) = &mut self.session
                            && session.user.id == user_id
                        {
                            // The CDN can hand back the same thumbnail URL
                            // even after an avatar edit. Add a version marker
                            // so the local texture cache cannot show the
                            // previous render.
                            let separator = if url.contains('?') { '&' } else { '?' };
                            let version = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis();
                            let fresh = format!("{url}{separator}v={version}");
                            session.avatar = Some(fresh.clone());
                            self.images.ensure(&fresh);
                        }
                    }
                    Err(error) => log::warn!("full-body avatar unavailable: {error}"),
                },
                ApiEvent::Profile { user_id, result } => {
                    let state = crate::api::load_state_from_result(result);
                    if user_id == self.profile.user_id {
                        self.profile.bundle = state.clone();
                    }
                    if self.session.as_ref().map(|session| session.user.id) == Some(user_id) {
                        self.avatar_edit.profile = state.clone();
                        if let Load::Ready(bundle) = &state {
                            if !self.avatar_edit.worn_initialized {
                                self.avatar_edit.worn = bundle.assets.clone();
                                self.avatar_edit.worn_initialized = true;
                            }
                            self.avatar_edit.emotes = bundle.emotes.clone();
                            self.avatar_edit.body_colors = bundle.body_colors.clone();
                            self.avatar_edit.scales = bundle.scales.clone();
                            self.avatar_edit.player_avatar_type = bundle.player_avatar_type.clone();
                        }
                    }
                }
                ApiEvent::UsernameResolved { query, result } => {
                    if self.screen != Screen::AddFriend || query != self.add_friend.username.trim()
                    {
                        continue;
                    }
                    match result {
                        Ok(user) => {
                            self.add_friend.target = Some((user.id, user.display().to_owned()));
                            if let Some(token) = self
                                .session
                                .as_ref()
                                .map(|session| session.token.expose_secret().to_owned())
                            {
                                self.add_friend.status =
                                    Some(("Sending friend request...".to_owned(), false));
                                self.api
                                    .send_friend_request(token, user.id, self.api_tx.clone());
                            } else {
                                self.add_friend.status =
                                    Some(("Sign in to send friend requests.".to_owned(), true));
                            }
                        }
                        Err(error) => {
                            self.add_friend.status = Some((error.user_message(), true));
                        }
                    }
                }
                ApiEvent::FriendRequest { user_id, result } => {
                    if self.screen == Screen::AddFriend {
                        match result {
                            Ok(()) => {
                                self.add_friend.status = Some((
                                    format!("Friend request sent to user {user_id}."),
                                    false,
                                ));
                            }
                            Err(error) => {
                                self.add_friend.status = Some((error.user_message(), true));
                            }
                        }
                    }
                }
                ApiEvent::ChatConversations(result) => match result {
                    Ok(conversations) => {
                        self.chat.conversations = Load::Ready(conversations);
                    }
                    Err(error) => {
                        self.chat.status = Some((error.user_message(), true));
                        self.chat.conversations = Load::Failed(error.user_message());
                    }
                },
                ApiEvent::ChatMessages {
                    conversation_id,
                    result,
                } => {
                    if self.chat.selected.as_deref() == Some(conversation_id.as_str()) {
                        self.chat.messages = crate::api::load_state_from_result(result);
                        if let (Load::Ready(messages), Load::Ready(conversations)) =
                            (&self.chat.messages, &mut self.chat.conversations)
                            && let Some(conversation) = conversations
                                .iter_mut()
                                .find(|conversation| conversation.id == conversation_id)
                        {
                            conversation.messages = messages.clone();
                        }
                    }
                }
                ApiEvent::ChatSent {
                    conversation_id,
                    result,
                } => {
                    if self.chat.selected.as_deref() == Some(conversation_id.as_str()) {
                        match result {
                            Ok(()) => {
                                self.chat.compose.clear();
                                self.chat.status = None;
                                self.chat.messages_for = None;
                            }
                            Err(error) => {
                                self.chat.status = Some((error.user_message(), true));
                            }
                        }
                    }
                }
                ApiEvent::ChatStarted { user_id, result } => {
                    let started = self.chat.start_with.take();
                    self.chat.start_requested = false;
                    match result {
                        Ok(conversation_id) => {
                            log::info!("started chat {} with user {}", conversation_id, user_id);
                            let name = started
                                .as_ref()
                                .map(|(_, name)| name.clone())
                                .or_else(|| self.chat.selected_name.clone())
                                .unwrap_or_else(|| format!("User {user_id}"));
                            self.chat.selected_name = Some(name.clone());
                            self.chat.selected = Some(conversation_id.clone());
                            self.chat.messages_for = None;
                            self.chat.messages = Load::Idle;
                            self.chat.status = None;
                            // The newly created conversation may not be in the
                            // server list yet (or it may be an empty DM). Keep
                            // a local row so the thread opens immediately.
                            if let Load::Ready(conversations) = &mut self.chat.conversations
                                && !conversations
                                    .iter()
                                    .any(|conversation| conversation.id == conversation_id)
                            {
                                let owner_id = self
                                    .session
                                    .as_ref()
                                    .map(|session| session.user.id)
                                    .unwrap_or_default();
                                let mut user_data = HashMap::new();
                                user_data.insert(
                                    user_id.to_string(),
                                    crate::models::ChatUserData {
                                        id: user_id,
                                        name: name.clone(),
                                        display_name: name,
                                    },
                                );
                                conversations.insert(
                                    0,
                                    ChatConversation {
                                        id: conversation_id,
                                        name: None,
                                        participant_user_ids: vec![owner_id, user_id],
                                        user_data,
                                        messages: Vec::new(),
                                        updated_at: String::new(),
                                    },
                                );
                            }
                        }
                        Err(error) => {
                            log::warn!(
                                "start chat with {user_id} failed: {} ({})",
                                error.user_message(),
                                error.detail()
                            );
                            let message = if matches!(&error, ApiError::Http { status: 400, .. }) {
                                "Could not open this chat. The user may have chat disabled."
                                    .to_owned()
                            } else {
                                format!(
                                    "Could not start a chat with user {user_id}: {}",
                                    error.user_message()
                                )
                            };
                            self.chat.status = Some((message, true));
                        }
                    }
                }
                ApiEvent::Inventory {
                    user_id,
                    asset_type,
                    result,
                } => {
                    if self.session.as_ref().map(|session| session.user.id) == Some(user_id) {
                        self.avatar_edit
                            .inventory
                            .insert(asset_type, crate::api::load_state_from_result(result));
                    }
                }
                ApiEvent::AvatarBundles { user_id, result } => {
                    if self.session.as_ref().map(|session| session.user.id) == Some(user_id) {
                        match result {
                            Ok(bundle) => {
                                for url in bundle.urls.values() {
                                    self.images.ensure(url);
                                }
                                self.avatar_edit.bundles = Load::Ready(bundle);
                            }
                            Err(error) => {
                                self.avatar_edit.bundles = Load::Failed(error.user_message());
                            }
                        }
                    }
                }
                ApiEvent::PlatformOutfits { user_id, result } => {
                    if self.session.as_ref().map(|session| session.user.id) == Some(user_id) {
                        match result {
                            Ok(outfits) => {
                                for url in outfits.urls.values() {
                                    self.images.ensure(url);
                                }
                                self.avatar_edit.platform_outfits = Load::Ready(outfits);
                            }
                            Err(error) => {
                                self.avatar_edit.platform_outfits =
                                    Load::Failed(error.user_message());
                            }
                        }
                    }
                }
                ApiEvent::OutfitEquipped(result) | ApiEvent::BundleEquipped(result) => {
                    self.avatar_edit.appearance_saving = false;
                    match result {
                        Ok(()) => {
                            self.avatar_edit.status = Some(("Avatar updated.".to_owned(), false));
                            self.avatar_edit.worn_initialized = false;
                            self.avatar_edit.profile = Load::Idle;
                            self.avatar_edit.requested = false;
                            if let Some(session) = &self.session {
                                self.api
                                    .load_user_avatar(session.user.id, self.api_tx.clone());
                            }
                        }
                        Err(error) => {
                            self.avatar_edit.status = Some((error.user_message(), true));
                        }
                    }
                }
                ApiEvent::BodyColorsUpdated(result) => {
                    self.avatar_edit.appearance_saving = false;
                    match result {
                        Ok(()) => {
                            self.avatar_edit.status =
                                Some(("Skin tone updated.".to_owned(), false));
                            self.avatar_edit.profile = Load::Idle;
                            self.avatar_edit.requested = false;
                            if let Some(session) = &self.session {
                                self.api
                                    .load_user_avatar(session.user.id, self.api_tx.clone());
                            }
                            self.avatar_refresh_at =
                                Some(Instant::now() + Duration::from_millis(1200));
                        }
                        Err(error) => {
                            self.avatar_edit.status = Some((error.user_message(), true));
                        }
                    }
                }
                ApiEvent::AvatarTypeUpdated(result) => {
                    self.avatar_edit.appearance_saving = false;
                    match result {
                        Ok(()) => {
                            self.avatar_edit.status =
                                Some(("Avatar type updated.".to_owned(), false));
                            self.avatar_edit.profile = Load::Idle;
                            self.avatar_edit.requested = false;
                            if let Some(session) = &self.session {
                                self.api
                                    .load_user_avatar(session.user.id, self.api_tx.clone());
                            }
                        }
                        Err(error) => {
                            self.avatar_edit.status = Some((error.user_message(), true));
                        }
                    }
                }
                ApiEvent::ScalesUpdated(result) => {
                    self.avatar_edit.appearance_saving = false;
                    match result {
                        Ok(()) => {
                            self.avatar_edit.status = Some(("Build updated.".to_owned(), false));
                            self.avatar_edit.profile = Load::Idle;
                            self.avatar_edit.requested = false;
                            if let Some(session) = &self.session {
                                self.api
                                    .load_user_avatar(session.user.id, self.api_tx.clone());
                            }
                        }
                        Err(error) => {
                            self.avatar_edit.status = Some((error.user_message(), true));
                        }
                    }
                }
                ApiEvent::EmotesUpdated(result) => {
                    self.avatar_edit.appearance_saving = false;
                    match result {
                        Ok(()) => {
                            self.avatar_edit.status = Some(("Emotes updated.".to_owned(), false));
                            self.avatar_edit.profile = Load::Idle;
                            self.avatar_edit.requested = false;
                            if let Some(session) = &self.session {
                                self.api
                                    .load_user_avatar(session.user.id, self.api_tx.clone());
                            }
                        }
                        Err(error) => {
                            self.avatar_edit.status = Some((error.user_message(), true));
                        }
                    }
                }
                ApiEvent::Favorites { user_id, result } => {
                    if self.session.as_ref().map(|session| session.user.id) == Some(user_id) {
                        self.favorites.games = crate::api::load_state_from_result(result);
                    }
                }
                ApiEvent::FavoriteStatus {
                    universe_id,
                    result,
                } => {
                    self.favorite_status_requested.remove(&universe_id);
                    match result {
                        Ok(is_favorited) => {
                            self.favorite_states.insert(universe_id, is_favorited);
                        }
                        Err(error) => {
                            // Fall back to the neutral state instead of
                            // retrying the status request every frame.
                            log::debug!(
                                "favorite status for {universe_id} failed: {}",
                                error.user_message()
                            );
                            self.favorite_states.insert(universe_id, false);
                        }
                    }
                }
                ApiEvent::FavoriteUpdated {
                    universe_id,
                    is_favorited,
                    result,
                } => {
                    self.favorite_pending.remove(&universe_id);
                    match result {
                        Ok(()) => {
                            self.favorite_states.insert(universe_id, is_favorited);
                            self.favorites.requested = false;
                            self.favorites.games = Load::Idle;
                            self.toast_silent(if is_favorited {
                                "Added to favorites."
                            } else {
                                "Removed from favorites."
                            });
                        }
                        Err(error) => {
                            self.toast(format!(
                                "Could not update favorite: {}",
                                error.user_message()
                            ));
                        }
                    }
                }
                ApiEvent::AvatarUpdated(result) => {
                    self.avatar_edit.saving = false;
                    match result {
                        Ok(()) => {
                            if self.avatar_edit.pending_save {
                                // More items were clicked while the first
                                // request was in flight.
                                self.save_avatar();
                            } else {
                                self.avatar_edit.status =
                                    Some(("Avatar updated.".to_owned(), false));
                                self.avatar_edit.worn_initialized = false;
                                self.avatar_edit.profile = Load::Idle;
                                self.avatar_edit.requested = false;
                                if let Some(session) = &self.session {
                                    let user_id = session.user.id;
                                    self.api.load_user_avatar(user_id, self.api_tx.clone());
                                }
                            }
                        }
                        Err(error) => {
                            self.avatar_edit.pending_save = false;
                            self.avatar_edit.status = Some((error.user_message(), true));
                        }
                    }
                }
                ApiEvent::SoberAccount { result } => match result {
                    Ok(account) => {
                        log::info!("Sober account check ok for launch");
                        let app_id = self.session.as_ref().map(|session| session.user.id);
                        let mismatch = app_id.is_some_and(|id| id != account.id);
                        self.launch.sober_account = Some(account);
                        if self.screen == Screen::Launch {
                            self.launch.phase = if mismatch {
                                LaunchPhase::Mismatch
                            } else {
                                LaunchPhase::Launching
                            };
                        }
                    }
                    Err(error) => {
                        self.launch.session_error = Some(error.user_message());
                        if self.screen == Screen::Launch {
                            self.launch.phase = LaunchPhase::SessionUnverified;
                        }
                    }
                },
            }
        }
    }

    fn update_status<T>(&mut self, result: &Result<T, ApiError>) {
        match result {
            Ok(_) => {
                self.api_status = "OK".to_owned();
                self.last_api_error = None;
            }
            Err(error) => {
                self.api_status = error.user_message();
                self.last_api_error = Some(error.detail());
            }
        }
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
        self.screen_changed = false;
        self.confirm = false;
        if ctx.input(|input| input.pointer.delta() != Vec2::ZERO || input.pointer.any_click()) {
            self.focus_visible = false;
        }

        let mut demo_actions = Vec::new();
        if let Some(demo) = &mut self.demo {
            let elapsed = demo.start.elapsed();
            while demo.index < demo.steps.len() && elapsed >= demo.steps[demo.index].0 {
                demo_actions.push(demo.steps[demo.index].1);
                demo.index += 1;
            }
        }
        if !demo_actions.is_empty() {
            self.focus_visible = true;
        }
        for action in demo_actions {
            log::info!("demo action: {action:?}");
            if action == Action::Confirm {
                self.confirm = true;
            }
            self.apply_action(action, ctx);
        }
        if self.demo.is_some() {
            return;
        }

        if self.lock_input || self.started.elapsed() < Duration::from_millis(600) {
            return;
        }
        let mouse_clicked = ctx.input(|input| input.pointer.any_click());

        if self.screen == Screen::Boot {
            if mouse_clicked {
                self.music.play_sfx(Sfx::Select, &self.paths, &self.config);
                self.leave_boot();
                return;
            }
            let actions = self.gamepad.poll();
            let keyboard = input::keyboard_actions(ctx);
            // Ignore bare stick/D-pad navigation. A drifting analog stick
            // otherwise dismissed the title screen without the player
            // pressing anything.
            if actions
                .iter()
                .chain(keyboard.iter())
                .any(|action| !matches!(action, Action::Navigate(_)))
            {
                self.music.play_sfx(Sfx::Select, &self.paths, &self.config);
                self.leave_boot();
            }
            return;
        }

        if self.screen == Screen::Details && self.details.lightbox.is_some() {
            let mut actions = self.gamepad.poll();
            actions.extend(input::keyboard_actions(ctx));
            if !actions.is_empty() {
                self.focus_visible = true;
            }
            for action in actions {
                match action {
                    Action::Back => self.close_lightbox(),
                    Action::Navigate(Dir::Left) => {
                        self.music
                            .play_sfx(Sfx::Navigate, &self.paths, &self.config);
                        self.lightbox_step(false);
                    }
                    Action::Navigate(Dir::Right) => {
                        self.music
                            .play_sfx(Sfx::Navigate, &self.paths, &self.config);
                        self.lightbox_step(true);
                    }
                    _ => {}
                }
            }
            return;
        }

        let mut actions = self.gamepad.poll();
        if self.search.editing {
            let mut submit = false;
            let mut cancel = false;
            ctx.input_mut(|input| {
                submit = input.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
                cancel = input.consume_key(egui::Modifiers::NONE, egui::Key::Escape);
            });
            if submit {
                self.search.editing = false;
                self.submit_search();
            }
            if cancel {
                self.search.editing = false;
            }
        } else if self.screen == Screen::Chat && self.chat.editing {
            let mut send = false;
            let mut cancel = false;
            ctx.input_mut(|input| {
                send = input.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
                cancel = input.consume_key(egui::Modifiers::NONE, egui::Key::Escape);
            });
            if send {
                self.send_chat_message();
            }
            if cancel {
                self.chat.editing = false;
            }
        } else {
            actions.extend(input::keyboard_actions(ctx));
        }

        if !actions.is_empty() {
            self.focus_visible = true;
        }
        let mut confirm = false;
        for action in actions {
            if action == Action::Confirm {
                confirm = true;
            }
            self.apply_action(action, ctx);
        }
        self.confirm = confirm && !self.screen_changed;
    }

    fn apply_action(&mut self, action: Action, ctx: &egui::Context) {
        log::debug!("input action: {action:?}");
        let effect = match action {
            Action::Navigate(_)
            | Action::TabPrev
            | Action::TabNext
            | Action::PagePrev
            | Action::PageNext
            | Action::FocusNext
            | Action::FocusPrev
            | Action::Settings
            | Action::Search => Some(Sfx::Navigate),
            Action::ToggleMute => Some(Sfx::Toggle),
            Action::Back | Action::Confirm | Action::Screenshot => None,
        };
        if let Some(effect) = effect {
            self.music.play_sfx(effect, &self.paths, &self.config);
        }
        match action {
            Action::Navigate(direction) => {
                if self.screen == Screen::Settings {
                    self.adjust_setting(direction);
                } else if self.screen == Screen::Browse
                    && matches!(direction, Dir::Left | Dir::Right)
                    && self
                        .focus
                        .current
                        .as_deref()
                        .is_some_and(|id| id.starts_with("browse:card:"))
                {
                    // Use the same one-card step as the wheel so keyboard
                    // movement has the same continuous slide animation.
                    self.browse_move(direction == Dir::Right);
                } else {
                    self.focus.navigate(direction, &self.registry);
                    if self.screen == Screen::Games {
                        self.scroll_games_to_focus();
                    }
                }
            }
            Action::Confirm => {
                if self.screen == Screen::Search
                    && self.focus.current.as_deref() == Some("search:field")
                {
                    self.search.editing = true;
                    self.search.focus_requested = true;
                }
            }
            Action::Back => {
                if self.screen == Screen::Browse {
                    self.start_browse_close();
                } else {
                    let effect = if self.screen == Screen::Details {
                        Sfx::SlideOut
                    } else {
                        Sfx::Back
                    };
                    self.music.play_sfx(effect, &self.paths, &self.config);
                    self.back(ctx);
                }
            }
            Action::Settings => {
                if self.screen != Screen::Settings && !self.search.editing {
                    self.open_settings();
                }
            }
            Action::Search => {
                if !self.search.editing {
                    self.open_search();
                }
            }
            Action::TabPrev => self.cycle_tab(false),
            Action::TabNext => self.cycle_tab(true),
            Action::PagePrev => self.page_games(false),
            Action::PageNext => self.page_games(true),
            Action::FocusNext => {
                self.focus.cycle(&self.registry, true);
            }
            Action::FocusPrev => {
                self.focus.cycle(&self.registry, false);
            }
            Action::ToggleMute => {
                self.config.muted = !self.config.muted;
                self.config.save(&self.paths);
                self.music.set_levels(&self.config);
                let state = if self.config.muted {
                    "Muted"
                } else {
                    "Sound on"
                };
                self.toast_silent(state);
            }
            Action::Screenshot => self.request_screenshot(None),
        }
    }

    pub fn open_lightbox(&mut self, urls: Vec<String>, index: usize) {
        if urls.is_empty() {
            return;
        }
        self.details.lightbox = Some(LightboxState {
            index: index.min(urls.len() - 1),
            urls,
            previous_index: None,
            slide_started: None,
            slide_forward: false,
        });
    }

    pub fn close_lightbox(&mut self) {
        self.details.lightbox = None;
    }

    pub fn lightbox_step(&mut self, forward: bool) {
        let Some(lightbox) = &mut self.details.lightbox else {
            return;
        };
        let len = lightbox.urls.len();
        if len == 0 {
            return;
        }
        let old = lightbox.index;
        lightbox.previous_index = Some(old);
        lightbox.index = if forward {
            (old + 1) % len
        } else {
            (old + len - 1) % len
        };
        lightbox.slide_started = Some(Instant::now());
        lightbox.slide_forward = forward;
    }

    fn back(&mut self, _ctx: &egui::Context) {
        if self.search.editing {
            self.search.editing = false;
            return;
        }
        match self.screen {
            Screen::Boot | Screen::SignIn => self.goto(Screen::Boot),
            Screen::Home | Screen::Games | Screen::Placeholder => self.open_settings(),
            Screen::Search => self.goto(self.search.return_screen),
            Screen::Favorites => self.goto(Screen::Games),
            Screen::Details => self.goto(self.details.from),
            Screen::Settings => self.goto(self.settings.from),
            Screen::Launch => self.goto(self.launch.from),
            Screen::Browse => self.start_browse_close(),
            Screen::Profile => self.goto(self.profile.from),
            Screen::AddFriend => self.goto(self.add_friend.from),
            Screen::Chat => self.goto(self.chat.from),
        }
    }

    fn cycle_tab(&mut self, forward: bool) {
        if matches!(
            self.screen,
            Screen::Details | Screen::Launch | Screen::Settings | Screen::Favorites
        ) {
            return;
        }
        let current = self.nav;
        let index = NavTab::ALL
            .iter()
            .position(|tab| *tab == current)
            .unwrap_or(0);
        let next = if forward {
            (index + 1) % NavTab::ALL.len()
        } else {
            (index + NavTab::ALL.len() - 1) % NavTab::ALL.len()
        };
        self.go_tab(NavTab::ALL[next]);
    }

    fn scroll_games_to_focus(&mut self) {
        let Some(id) = self.focus.current.as_deref() else {
            return;
        };
        let Some(column) = id
            .strip_prefix("games:")
            .and_then(|rest| rest.split(':').next())
            .and_then(|value| value.parse::<usize>().ok())
        else {
            return;
        };
        let column_x = 96.0 + column as f32 * crate::screens::games::COLUMN_STEP;
        let screen_x = column_x - self.games.scroll;
        let viewport = crate::screens::games::VIEWPORT_WIDTH;
        let mut target = self.games.scroll_target;
        if screen_x < 96.0 {
            target = column_x - 96.0;
        } else if screen_x + crate::screens::games::COLUMN_WIDTH > 96.0 + viewport {
            target = column_x + crate::screens::games::COLUMN_WIDTH - 96.0 - viewport;
        }
        self.games.scroll_target = target.max(0.0);
    }

    fn page_games(&mut self, forward: bool) {
        match self.screen {
            Screen::Browse => self.browse_move(forward),
            Screen::Details => {
                let step = if forward { 300.0 } else { -300.0 };
                self.details.scroll = (self.details.scroll + step).max(0.0);
            }
            Screen::Placeholder if self.placeholder_tab == NavTab::Friends => {
                let step = if forward { 260.0 } else { -260.0 };
                self.friends_scroll = (self.friends_scroll + step).max(0.0);
            }
            Screen::Games => {
                let step = crate::screens::games::COLUMN_STEP;
                let delta = if forward { step } else { -step };
                self.games.scroll_target = (self.games.scroll_target + delta).max(0.0);
            }
            _ => {}
        }
    }

    fn adjust_setting(&mut self, direction: Dir) {
        match self.focus.current.as_deref() {
            Some("settings:master") => {
                if direction == Dir::Left || direction == Dir::Right {
                    let step = if direction == Dir::Right { 0.05 } else { -0.05 };
                    self.config.master_volume = (self.config.master_volume + step).clamp(0.0, 1.0);
                    self.config.save(&self.paths);
                    self.music.set_levels(&self.config);
                }
            }
            Some("settings:music") => {
                if direction == Dir::Left || direction == Dir::Right {
                    let step = if direction == Dir::Right { 0.05 } else { -0.05 };
                    self.config.music_volume = (self.config.music_volume + step).clamp(0.0, 1.0);
                    self.config.save(&self.paths);
                    self.music.set_levels(&self.config);
                }
            }
            _ => {
                self.focus.navigate(direction, &self.registry);
            }
        }
    }

    fn update_audio_focus(&mut self, ctx: &egui::Context) {
        let focused = ctx.input(|input| input.viewport().focused.unwrap_or(true));
        if focused == self.window_focused {
            return;
        }
        self.window_focused = focused;
        log::info!("window focus changed: focused={focused}");
        // Only the output volume changes on focus loss. The music player and
        // its in-memory loop keep running on the audio thread.
        self.music
            .set_focus_muted(!focused && self.config.mute_on_focus_loss);
    }

    fn poll_video(&mut self, ctx: &egui::Context) {
        if self.config.reduced_motion {
            if self.video.is_some() {
                self.video = None;
                self.background = self.procedural_background.clone();
            }
            return;
        }

        // Do not keep decoding/uploading frames while Sober owns the
        // foreground. The bounded frame channel blocks ffmpeg, so the last
        // frame stays visible and the game gets the CPU/GPU back. Losing
        // window focus is different: the shell keeps running and only mutes
        // its audio, so background video and music stay in sync.
        if self.game_running() || self.screen == Screen::Launch {
            return;
        }

        let Some(video) = &mut self.video else {
            return;
        };
        if let Some(texture) = video.poll(ctx) {
            self.background = texture;
        }
        if video.ended() {
            log::warn!("background video ended: {}", video.path().display());
            let still_procedural = self.background.id() == self.procedural_background.id();
            self.video = None;
            if still_procedural {
                self.video_error =
                    Some("The background video ended before producing a frame.".to_owned());
            }
        }
    }

    pub fn stop_video(&mut self) {
        self.video = None;
        self.video_error = None;
        self.background = self.procedural_background.clone();
    }

    pub fn reload_video(&mut self) {
        self.video = None;
        self.video_error = None;
        self.background = self.procedural_background.clone();
        let (video, error) = start_background_video(&self.paths, &self.config);
        self.video = video;
        self.video_error = error;
    }

    fn tick_toast(&mut self) {
        if let Some(toast) = &self.toast
            && toast.created.elapsed() > toast.duration
        {
            self.toast = None;
        }
    }

    fn request_screenshot(&mut self, path: Option<PathBuf>) {
        let path = path.unwrap_or_else(|| {
            self.paths.screenshot_dir().join(format!(
                "roblox-2016-{}.png",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|duration| duration.as_secs())
                    .unwrap_or(0)
            ))
        });
        self.auto_screenshot = Some(AutoScreenshot {
            path,
            deadline: Instant::now(),
            requested: false,
        });
    }

    fn handle_screenshot(&mut self, ctx: &egui::Context) {
        if self.auto_screenshot.is_none() {
            return;
        }
        let due = self
            .auto_screenshot
            .as_ref()
            .is_some_and(|shot| !shot.requested && Instant::now() >= shot.deadline);
        if due {
            if let Some(shot) = &mut self.auto_screenshot {
                shot.requested = true;
            }
            let tx = self.screenshot_tx.clone();
            ctx.request_screenshot(move |image| {
                let _ = tx.send(image);
            });
        }

        let image = match self.screenshot_rx.try_recv() {
            Ok(image) => Some(image),
            Err(_) => None,
        };

        if let Some(image) = image
            && let Some(shot) = self.auto_screenshot.take()
        {
            match save_screenshot(&shot.path, image.as_ref()) {
                Ok(()) => {
                    self.toast_silent(format!("Screenshot saved to {}", shot.path.display()));
                    if self.close_after_screenshot {
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                    }
                }
                Err(error) => {
                    log::warn!("Could not save screenshot: {error}");
                    self.toast(format!("Could not save screenshot: {error}"));
                }
            }
        }
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        let background = self.background.clone();
        let wordmark = self.wordmark.clone();
        let focus = std::mem::take(&mut self.focus);
        let lightbox_open = self.screen == Screen::Details && self.details.lightbox.is_some();
        let mut canvas = Canvas::new(ui, focus, self.confirm, !self.lock_input);
        if lightbox_open {
            canvas.input_enabled = false;
        }
        canvas.focus_visible = self.focus_visible;
        canvas.video_active = self.video.as_ref().is_some_and(|video| video.has_frame());
        canvas.video_luma = self.video.as_ref().map(|video| video.luma()).unwrap_or(0.0);

        match self.screen {
            Screen::Boot => self.show_boot(&mut canvas, &background, wordmark.as_ref()),
            Screen::SignIn => self.show_signin(&mut canvas, &background, wordmark.as_ref()),
            Screen::Home => self.show_home(&mut canvas, &background, wordmark.as_ref()),
            Screen::Games => self.show_games(&mut canvas, &background, wordmark.as_ref()),
            Screen::Favorites => self.show_favorites(&mut canvas, &background),
            Screen::Search => self.show_search(&mut canvas, &background, wordmark.as_ref()),
            Screen::Details => self.show_details(&mut canvas, &background),
            Screen::Settings => self.show_settings(&mut canvas, &background, wordmark.as_ref()),
            Screen::Launch => self.show_launch(&mut canvas, &background),
            Screen::Browse => self.show_browse(&mut canvas, &background, wordmark.as_ref()),
            Screen::Profile => self.show_profile(&mut canvas, &background),
            Screen::AddFriend => self.show_add_friend(&mut canvas, &background),
            Screen::Chat => self.show_chat(&mut canvas, &background, wordmark.as_ref()),
            Screen::Placeholder => {
                self.show_placeholder(&mut canvas, &background, wordmark.as_ref())
            }
        }

        let sfx = std::mem::take(&mut canvas.sfx);
        let (focus, registry) = canvas.finish();
        self.focus = focus;
        self.registry = registry;
        self.focus.ensure(&self.registry);

        if lightbox_open {
            self.draw_lightbox(ui);
        }

        for effect in sfx {
            self.music.play_sfx(effect, &self.paths, &self.config);
        }

        if self.transition.active() {
            let alpha = self.transition.alpha();
            ui.painter()
                .rect_filled(ui.max_rect(), 0, Color32::from_black_alpha(alpha));
        }

        if let Some(toast) = &self.toast {
            self.draw_toast(ui, toast);
        }
    }

    fn draw_toast(&self, ui: &mut egui::Ui, toast: &Toast) {
        let canvas = Canvas::overlay(ui, FocusState::default(), false, false);
        let width = (canvas.text_width(&toast.message, 19.0, false) + 56.0).min(900.0);
        let rect = Rect::from_center_size(
            Pos2::new(theme::CANVAS_W * 0.5, 958.0),
            Vec2::new(width, 56.0),
        );
        canvas.rect_filled(rect, 2, Color32::from_black_alpha(220));
        canvas.rect_stroke(rect, 2, egui::Stroke::new(1.0, theme::ACCENT));
        canvas.text_centered(rect, &toast.message, 19.0, theme::TEXT_PRIMARY, false);
        let _ = canvas.finish();
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll_login_process();
        self.poll_api();
        if self
            .avatar_refresh_at
            .is_some_and(|at| Instant::now() >= at)
        {
            self.avatar_refresh_at = None;
            if let Some(session) = &self.session {
                self.api
                    .load_user_avatar(session.user.id, self.api_tx.clone());
            }
        }
        self.images.poll(&ctx);
        self.poll_video(&ctx);
        self.handle_input(&ctx);
        self.ensure_screen_data();
        self.poll_game_process();
        self.music.set_launching(self.screen == Screen::Launch);
        self.music.set_game_running(self.game_running());
        // Keep the menu theme at a constant level; the old per-screen duck
        // made it sound like the volume was glitching whenever tabs changed.
        self.music.set_browsing(false);
        self.music
            .update(ctx.input(|input| input.stable_dt).min(0.1));
        self.draw(ui);
        self.handle_screenshot(&ctx);
        self.update_audio_focus(&ctx);
        if self.auto_screenshot.is_none()
            && let Some(maximized) = ctx.input(|input| input.viewport().maximized)
            && maximized != self.config.window_maximized
        {
            self.config.window_maximized = maximized;
            self.config.save(&self.paths);
        }
        self.tick_toast();
        // Full frame rate while the menu is in front; a slow heartbeat while
        // the shell is in the background or Sober owns the display.
        let active = !self.game_running() || self.screen == Screen::Launch;
        // The sliding rails use time-based easing; asking for a frame every
        // 8 ms keeps keyboard-driven motion from looking stepped next to a
        // continuous wheel gesture.
        let interval = if active && matches!(self.screen, Screen::Games | Screen::Browse) {
            8
        } else if active {
            16
        } else {
            250
        };
        ctx.request_repaint_after(Duration::from_millis(interval));
    }
}

fn load_outfits(paths: &Paths, user_id: u64) -> Vec<SavedOutfit> {
    let path = paths.outfits_file(user_id);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    match serde_json::from_str::<OutfitFile>(&text) {
        Ok(file) if file.user_id == 0 || file.user_id == user_id => file.outfits,
        Ok(_) => Vec::new(),
        Err(error) => {
            log::warn!("Ignoring invalid outfits file {}: {error}", path.display());
            Vec::new()
        }
    }
}

fn write_outfits(paths: &Paths, user_id: u64, outfits: &[SavedOutfit]) -> Result<(), String> {
    let path = paths.outfits_file(user_id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let file = OutfitFile {
        user_id,
        outfits: outfits.to_vec(),
    };
    let text = serde_json::to_string_pretty(&file).map_err(|error| error.to_string())?;
    std::fs::write(&path, text).map_err(|error| error.to_string())
}

fn load_texture(ctx: &egui::Context, name: &str, bytes: &[u8]) -> Option<TextureHandle> {
    let image = image::load_from_memory(bytes).ok()?.to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let pixels = image
        .pixels()
        .map(|pixel| Color32::from_rgba_unmultiplied(pixel[0], pixel[1], pixel[2], pixel[3]))
        .collect();
    Some(ctx.load_texture(
        name,
        egui::ColorImage::new(size, pixels),
        egui::TextureOptions::LINEAR,
    ))
}

fn save_screenshot(path: &PathBuf, image: &egui::ColorImage) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let width = image.size[0] as u32;
    let height = image.size[1] as u32;
    let mut bytes = Vec::with_capacity((width * height * 4) as usize);
    for pixel in &image.pixels {
        bytes.extend_from_slice(&[pixel.r(), pixel.g(), pixel.b(), pixel.a()]);
    }
    image::save_buffer(path, &bytes, width, height, image::ColorType::Rgba8)
        .map_err(|error| error.to_string())
}

fn start_background_video(
    paths: &Paths,
    config: &Config,
) -> (Option<VideoBackground>, Option<String>) {
    if config.reduced_motion {
        return (None, None);
    }
    let Some(path) = paths.background_video_file() else {
        return (None, None);
    };
    if !video_background::ffmpeg_available() {
        return (
            None,
            Some("ffmpeg is not installed. The procedural background is in use.".to_owned()),
        );
    }
    match VideoBackground::start(&path) {
        Ok(video) => {
            log::info!("background video: {}", path.display());
            (Some(video), None)
        }
        Err(error) => {
            log::warn!("background video failed: {error}");
            (None, Some(error))
        }
    }
}

fn detect_sober() -> SoberStatus {
    let cordial_installed = flatpak_installed("io.github.luohua97.Cordial");
    let output = std::process::Command::new("flatpak")
        .args(["info", "org.vinegarhq.Sober"])
        .output();
    match output {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout);
            let version = text
                .lines()
                .find_map(|line| line.trim().strip_prefix("Version:"))
                .map(|value| value.trim().to_owned())
                .unwrap_or_default();
            SoberStatus {
                installed: true,
                version,
                detail: "Sober Flatpak detected".to_owned(),
                cordial_installed,
            }
        }
        Ok(_) => SoberStatus {
            installed: false,
            version: String::new(),
            detail: "Sober Flatpak not installed".to_owned(),
            cordial_installed,
        },
        Err(error) => SoberStatus {
            installed: false,
            version: String::new(),
            detail: format!("Could not run flatpak: {error}"),
            cordial_installed,
        },
    }
}

fn flatpak_installed(id: &str) -> bool {
    std::process::Command::new("flatpak")
        .args(["info", id])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn open_url(url: &str) -> Result<(), String> {
    std::process::Command::new("xdg-open")
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("could not open {url}: {error}"))
}
