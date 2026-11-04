use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::Client;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use tokio::runtime::{Handle, Runtime};

use crate::models::{
    AssetThumbnail, AssetThumbnailsResponse, AuthenticatedUser, AvatarBundle, AvatarBundlesBundle,
    AvatarDetails, BodyColors, ChatConversation, ChatMessage, CountResponse, CurrencyResponse,
    DetailsBundle, FavoriteGameEntry, FavoritesResponse, Friend, FriendsResponse, GameDetails,
    GameFavoriteResponse, GameSummary, GamesResponse, InventoryBundle, InventoryItem,
    InventoryResponse, Load, MediaItem, MediaResponse, PlaceUniverseResponse, PlatformOutfit,
    PlatformOutfitsBundle, Presence, PresenceResponse, ProfileBundle, PublicProfile, RawSort,
    SearchResponse, SortContentResponse, SortIdsResponse, SortInfo, SortsResponse, UsersResponse,
    Votes, VotesResponse,
};

const USER_AGENT: &str = "rbx2016/0.1 (Linux; Wayland; local fan build)";

#[derive(Debug, Clone, thiserror::Error)]
pub enum ApiError {
    #[error("network error: {0}")]
    Network(String),
    #[error("HTTP {status}: {detail}")]
    Http { status: u16, detail: String },
    #[error("invalid response: {0}")]
    Decode(String),
    #[error("no data: {0}")]
    Empty(String),
}

impl ApiError {
    pub fn user_message(&self) -> String {
        match self {
            Self::Network(_) => "Could not reach Roblox. Check your connection.".to_owned(),
            Self::Http { status: 429, .. } => {
                "Roblox is rate-limiting requests. Try again shortly.".to_owned()
            }
            Self::Http { status, .. } => format!("Roblox returned HTTP {status}."),
            Self::Decode(_) => "Roblox returned an unexpected response.".to_owned(),
            Self::Empty(_) => "Roblox returned no data.".to_owned(),
        }
    }

    pub fn detail(&self) -> String {
        self.to_string()
    }
}

#[derive(Clone)]
pub struct Api {
    client: Client,
    runtime: Arc<Runtime>,
    session_id: Arc<str>,
}

pub enum ApiEvent {
    Sorts(Result<Vec<SortInfo>, ApiError>),
    SortContent {
        sort_id: String,
        result: Result<Vec<GameSummary>, ApiError>,
    },
    Thumbnails {
        ids: Vec<u64>,
        result: Result<HashMap<u64, String>, ApiError>,
    },
    Search {
        query: String,
        result: Result<Vec<GameSummary>, ApiError>,
    },
    Details {
        universe_id: u64,
        result: Result<DetailsBundle, ApiError>,
    },
    #[allow(dead_code)]
    PlaceResolved {
        place_id: u64,
        result: Result<u64, ApiError>,
    },
    Authenticated {
        token: String,
        result: Result<AuthenticatedUser, ApiError>,
    },
    Friends {
        user_id: u64,
        result: Result<Vec<Friend>, ApiError>,
    },
    Presence {
        user_id: u64,
        result: Result<HashMap<u64, Presence>, ApiError>,
    },
    Robux {
        user_id: u64,
        result: Result<u64, ApiError>,
    },
    UserHeadshots {
        result: Result<HashMap<u64, String>, ApiError>,
    },
    UserAvatar {
        user_id: u64,
        result: Result<String, ApiError>,
    },
    Profile {
        user_id: u64,
        result: Result<ProfileBundle, ApiError>,
    },
    UsernameResolved {
        query: String,
        result: Result<AuthenticatedUser, ApiError>,
    },
    FriendRequest {
        user_id: u64,
        result: Result<(), ApiError>,
    },
    ChatConversations(Result<Vec<ChatConversation>, ApiError>),
    ChatMessages {
        conversation_id: String,
        result: Result<Vec<ChatMessage>, ApiError>,
    },
    ChatSent {
        conversation_id: String,
        result: Result<(), ApiError>,
    },
    ChatStarted {
        user_id: u64,
        result: Result<String, ApiError>,
    },
    Inventory {
        user_id: u64,
        asset_type: u64,
        result: Result<InventoryBundle, ApiError>,
    },
    AvatarUpdated(Result<(), ApiError>),
    BodyColorsUpdated(Result<(), ApiError>),
    AvatarTypeUpdated(Result<(), ApiError>),
    ScalesUpdated(Result<(), ApiError>),
    EmotesUpdated(Result<(), ApiError>),
    AvatarBundles {
        user_id: u64,
        result: Result<AvatarBundlesBundle, ApiError>,
    },
    PlatformOutfits {
        user_id: u64,
        result: Result<PlatformOutfitsBundle, ApiError>,
    },
    OutfitEquipped(Result<(), ApiError>),
    BundleEquipped(Result<(), ApiError>),
    Favorites {
        user_id: u64,
        result: Result<Vec<GameSummary>, ApiError>,
    },
    FavoriteStatus {
        universe_id: u64,
        result: Result<bool, ApiError>,
    },
    FavoriteUpdated {
        universe_id: u64,
        is_favorited: bool,
        result: Result<(), ApiError>,
    },
    SoberAccount {
        result: Result<AuthenticatedUser, ApiError>,
    },
}

impl Api {
    pub fn new() -> Result<Self, String> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(|error| format!("could not start network runtime: {error}"))?;
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|error| format!("could not create HTTP client: {error}"))?;
        Ok(Self {
            client,
            runtime: Arc::new(runtime),
            session_id: Arc::from(uuid::Uuid::new_v4().to_string()),
        })
    }

    pub fn runtime_handle(&self) -> Handle {
        self.runtime.handle().clone()
    }

    pub fn spawn<F>(&self, future: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        self.runtime.spawn(future);
    }

    pub fn load_sorts(&self, tx: std::sync::mpsc::Sender<ApiEvent>) {
        let client = self.client.clone();
        let session = self.session_id.to_string();
        self.spawn(async move {
            let result = fetch_sorts(&client, &session).await;
            let _ = tx.send(ApiEvent::Sorts(result));
        });
    }

    pub fn load_sort_content(&self, sort_id: String, tx: std::sync::mpsc::Sender<ApiEvent>) {
        let client = self.client.clone();
        let session = self.session_id.to_string();
        self.spawn(async move {
            let result = fetch_sort_content(&client, &session, &sort_id).await;
            let _ = tx.send(ApiEvent::SortContent { sort_id, result });
        });
    }

    pub fn load_thumbnails(&self, ids: Vec<u64>, tx: std::sync::mpsc::Sender<ApiEvent>) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = fetch_icons(&client, &ids).await;
            let _ = tx.send(ApiEvent::Thumbnails { ids, result });
        });
    }

    pub fn search(&self, query: String, tx: std::sync::mpsc::Sender<ApiEvent>) {
        let client = self.client.clone();
        let session = self.session_id.to_string();
        self.spawn(async move {
            let result = fetch_search(&client, &session, &query).await;
            let _ = tx.send(ApiEvent::Search { query, result });
        });
    }

    pub fn load_details(&self, universe_id: u64, tx: std::sync::mpsc::Sender<ApiEvent>) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = fetch_details(&client, universe_id).await;
            let _ = tx.send(ApiEvent::Details {
                universe_id,
                result,
            });
        });
    }

    /// Phase 3: resolve a place ID when only the place is known.
    #[allow(dead_code)]
    pub fn resolve_place(&self, place_id: u64, tx: std::sync::mpsc::Sender<ApiEvent>) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = fetch_place_universe(&client, place_id).await;
            let _ = tx.send(ApiEvent::PlaceResolved { place_id, result });
        });
    }

    pub fn load_account(&self, token: String, tx: std::sync::mpsc::Sender<ApiEvent>) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = fetch_account(&client, &token).await;
            let _ = tx.send(ApiEvent::Authenticated { token, result });
        });
    }

    pub fn check_account(&self, token: String, tx: std::sync::mpsc::Sender<ApiEvent>) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = fetch_account(&client, &token).await;
            let _ = tx.send(ApiEvent::SoberAccount { result });
        });
    }

    pub fn load_friends(&self, token: String, user_id: u64, tx: std::sync::mpsc::Sender<ApiEvent>) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = fetch_friends(&client, &token, user_id).await;
            let _ = tx.send(ApiEvent::Friends { user_id, result });
        });
    }

    pub fn load_presence(
        &self,
        token: String,
        user_id: u64,
        friend_ids: Vec<u64>,
        tx: std::sync::mpsc::Sender<ApiEvent>,
    ) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = fetch_presence(&client, &token, &friend_ids).await;
            let _ = tx.send(ApiEvent::Presence { user_id, result });
        });
    }

    pub fn load_robux(&self, token: String, user_id: u64, tx: std::sync::mpsc::Sender<ApiEvent>) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = fetch_robux(&client, &token, user_id).await;
            let _ = tx.send(ApiEvent::Robux { user_id, result });
        });
    }

    pub fn load_user_headshots(&self, ids: Vec<u64>, tx: std::sync::mpsc::Sender<ApiEvent>) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = fetch_user_headshots(&client, &ids).await;
            let _ = tx.send(ApiEvent::UserHeadshots { result });
        });
    }

    pub fn load_user_avatar(&self, user_id: u64, tx: std::sync::mpsc::Sender<ApiEvent>) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = fetch_user_avatar(&client, user_id).await;
            let _ = tx.send(ApiEvent::UserAvatar { user_id, result });
        });
    }

    pub fn load_profile(&self, user_id: u64, tx: std::sync::mpsc::Sender<ApiEvent>) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = fetch_profile(&client, user_id).await;
            let _ = tx.send(ApiEvent::Profile { user_id, result });
        });
    }

    pub fn resolve_username(&self, query: String, tx: std::sync::mpsc::Sender<ApiEvent>) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = fetch_username(&client, &query).await;
            let _ = tx.send(ApiEvent::UsernameResolved { query, result });
        });
    }

    pub fn send_friend_request(
        &self,
        token: String,
        user_id: u64,
        tx: std::sync::mpsc::Sender<ApiEvent>,
    ) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = post_friend_request(&client, &token, user_id).await;
            let _ = tx.send(ApiEvent::FriendRequest { user_id, result });
        });
    }

    pub fn load_chat_conversations(&self, token: String, tx: std::sync::mpsc::Sender<ApiEvent>) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = fetch_chat_conversations(&client, &token).await;
            let _ = tx.send(ApiEvent::ChatConversations(result));
        });
    }

    pub fn load_chat_messages(
        &self,
        token: String,
        conversation_id: String,
        tx: std::sync::mpsc::Sender<ApiEvent>,
    ) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = fetch_chat_messages(&client, &token, &conversation_id).await;
            let _ = tx.send(ApiEvent::ChatMessages {
                conversation_id,
                result,
            });
        });
    }

    pub fn send_chat_message(
        &self,
        token: String,
        conversation_id: String,
        message: String,
        tx: std::sync::mpsc::Sender<ApiEvent>,
    ) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = post_chat_message(&client, &token, &conversation_id, &message).await;
            let _ = tx.send(ApiEvent::ChatSent {
                conversation_id,
                result,
            });
        });
    }

    pub fn start_chat(
        &self,
        token: String,
        owner_id: u64,
        user_id: u64,
        tx: std::sync::mpsc::Sender<ApiEvent>,
    ) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = post_start_chat(&client, &token, owner_id, user_id).await;
            let _ = tx.send(ApiEvent::ChatStarted { user_id, result });
        });
    }

    pub fn load_inventory(
        &self,
        token: String,
        user_id: u64,
        asset_type: u64,
        tx: std::sync::mpsc::Sender<ApiEvent>,
    ) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = fetch_inventory(&client, &token, user_id, asset_type).await;
            let _ = tx.send(ApiEvent::Inventory {
                user_id,
                asset_type,
                result,
            });
        });
    }

    pub fn set_avatar(
        &self,
        token: String,
        assets: Vec<crate::models::AvatarAsset>,
        tx: std::sync::mpsc::Sender<ApiEvent>,
    ) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = post_avatar(&client, &token, &assets).await;
            let _ = tx.send(ApiEvent::AvatarUpdated(result));
        });
    }

    pub fn set_body_colors(
        &self,
        token: String,
        colors: BodyColors,
        tx: std::sync::mpsc::Sender<ApiEvent>,
    ) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = post_body_colors(&client, &token, &colors).await;
            let _ = tx.send(ApiEvent::BodyColorsUpdated(result));
        });
    }

    pub fn set_avatar_type(
        &self,
        token: String,
        avatar_type: String,
        tx: std::sync::mpsc::Sender<ApiEvent>,
    ) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = post_avatar_type(&client, &token, &avatar_type).await;
            let _ = tx.send(ApiEvent::AvatarTypeUpdated(result));
        });
    }

    pub fn set_scales(
        &self,
        token: String,
        height: f64,
        width: f64,
        head: f64,
        depth: f64,
        proportion: f64,
        body_type: f64,
        tx: std::sync::mpsc::Sender<ApiEvent>,
    ) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = post_scales(
                &client, &token, height, width, head, depth, proportion, body_type,
            )
            .await;
            let _ = tx.send(ApiEvent::ScalesUpdated(result));
        });
    }

    pub fn load_avatar_bundles(&self, user_id: u64, tx: std::sync::mpsc::Sender<ApiEvent>) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = fetch_avatar_bundles(&client, user_id).await;
            let _ = tx.send(ApiEvent::AvatarBundles { user_id, result });
        });
    }

    pub fn load_platform_outfits(
        &self,
        token: String,
        user_id: u64,
        tx: std::sync::mpsc::Sender<ApiEvent>,
    ) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = fetch_platform_outfits(&client, &token, user_id).await;
            let _ = tx.send(ApiEvent::PlatformOutfits { user_id, result });
        });
    }

    pub fn equip_outfit(
        &self,
        token: String,
        outfit_id: u64,
        tx: std::sync::mpsc::Sender<ApiEvent>,
    ) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = equip_platform_outfit(&client, &token, outfit_id).await;
            let _ = tx.send(ApiEvent::OutfitEquipped(result));
        });
    }

    pub fn equip_bundle(
        &self,
        token: String,
        bundle_id: u64,
        tx: std::sync::mpsc::Sender<ApiEvent>,
    ) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = equip_avatar_bundle(&client, &token, bundle_id).await;
            let _ = tx.send(ApiEvent::BundleEquipped(result));
        });
    }

    pub fn set_emote(
        &self,
        token: String,
        asset_id: u64,
        position: u64,
        tx: std::sync::mpsc::Sender<ApiEvent>,
    ) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = post_emote(&client, &token, asset_id, position).await;
            let _ = tx.send(ApiEvent::EmotesUpdated(result));
        });
    }

    pub fn remove_emote(
        &self,
        token: String,
        position: u64,
        tx: std::sync::mpsc::Sender<ApiEvent>,
    ) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = delete_emote(&client, &token, position).await;
            let _ = tx.send(ApiEvent::EmotesUpdated(result));
        });
    }

    pub fn load_favorites(&self, user_id: u64, tx: std::sync::mpsc::Sender<ApiEvent>) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = fetch_favorites(&client, user_id).await;
            let _ = tx.send(ApiEvent::Favorites { user_id, result });
        });
    }

    pub fn load_favorite_status(
        &self,
        token: String,
        universe_id: u64,
        tx: std::sync::mpsc::Sender<ApiEvent>,
    ) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = fetch_favorite_status(&client, &token, universe_id).await;
            let _ = tx.send(ApiEvent::FavoriteStatus {
                universe_id,
                result,
            });
        });
    }

    pub fn set_favorite(
        &self,
        token: String,
        universe_id: u64,
        is_favorited: bool,
        tx: std::sync::mpsc::Sender<ApiEvent>,
    ) {
        let client = self.client.clone();
        self.spawn(async move {
            let result = post_favorite(&client, &token, universe_id, is_favorited).await;
            let _ = tx.send(ApiEvent::FavoriteUpdated {
                universe_id,
                is_favorited,
                result,
            });
        });
    }
}

async fn fetch_sorts(client: &Client, session: &str) -> Result<Vec<SortInfo>, ApiError> {
    let response: SortsResponse = get_json(
        client,
        "https://apis.roblox.com/explore-api/v1/get-sorts",
        &[
            ("sessionId", session.to_owned()),
            ("device", "console".to_owned()),
            ("country", "all".to_owned()),
        ],
    )
    .await?;

    let mut sorts: Vec<SortInfo> = response
        .sorts
        .into_iter()
        .filter(|sort: &RawSort| sort.content_type == "Games" && sort.treatment_type == "Carousel")
        .map(|sort| SortInfo {
            display_name: if sort.display_name.is_empty() {
                sort.sort_id.clone()
            } else {
                sort.display_name
            },
            sort_id: sort.sort_id,
        })
        .collect();

    // The five carousel rows above are the front-page charts. The genre
    // sorts are a separate endpoint; the old shell exposed them alongside
    // the main rows, so add them after the carousels instead of silently
    // dropping every other browsing category.
    if let Ok(genre_ids) = fetch_sort_ids(client).await {
        const GENRES: &[(&str, &str)] = &[
            ("action", "Action"),
            ("adventure", "Adventure"),
            ("entertainment", "Entertainment"),
            ("obby_and_platformer", "Obby & Platformer"),
            ("party_and_casual", "Party & Casual"),
            ("puzzle", "Puzzle"),
            ("rpg", "RPG"),
            ("roleplay_and_avatar_sim", "Roleplay & Avatar Sim"),
            ("shooter", "Shooter"),
            ("shopping", "Shopping"),
            ("simulation", "Simulation"),
            ("sports_and_racing", "Sports & Racing"),
            ("strategy", "Strategy"),
            ("survival", "Survival"),
        ];
        for (key, label) in GENRES {
            if let Some(sort_id) = genre_ids.genre_to_sort_id.get(*key) {
                sorts.push(SortInfo {
                    sort_id: sort_id.clone(),
                    display_name: (*label).to_owned(),
                });
            }
        }
    }

    if sorts.is_empty() {
        return Err(ApiError::Empty("no game sorts returned".to_owned()));
    }
    Ok(sorts)
}

async fn fetch_sort_ids(client: &Client) -> Result<SortIdsResponse, ApiError> {
    get_json(
        client,
        "https://apis.roblox.com/explore-api/v1/get-sort-ids",
        &[],
    )
    .await
}

async fn fetch_sort_content(
    client: &Client,
    session: &str,
    sort_id: &str,
) -> Result<Vec<GameSummary>, ApiError> {
    let response: SortContentResponse = get_json(
        client,
        "https://apis.roblox.com/explore-api/v1/get-sort-content",
        &[
            ("sessionId", session.to_owned()),
            ("sortId", sort_id.to_owned()),
            ("device", "console".to_owned()),
        ],
    )
    .await?;
    Ok(response
        .games
        .into_iter()
        .filter(GameSummary::is_game)
        .collect())
}

async fn fetch_search(
    client: &Client,
    session: &str,
    query: &str,
) -> Result<Vec<GameSummary>, ApiError> {
    let response: SearchResponse = get_json(
        client,
        "https://apis.roblox.com/search-api/omni-search",
        &[
            ("searchQuery", query.to_owned()),
            ("sessionId", session.to_owned()),
            ("pageToken", String::new()),
            ("pageType", "all".to_owned()),
        ],
    )
    .await?;

    let mut seen = HashSet::new();
    let mut games = Vec::new();
    for group in response.search_results {
        for game in group.contents {
            if game.is_game() && seen.insert(game.universe_id) {
                games.push(game);
            }
        }
    }
    Ok(games)
}

async fn fetch_icons(client: &Client, ids: &[u64]) -> Result<HashMap<u64, String>, ApiError> {
    let mut result = HashMap::new();
    for chunk in ids.chunks(50) {
        let id_list = chunk
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let response: AssetThumbnailsResponse = get_json(
            client,
            "https://thumbnails.roblox.com/v1/games/icons",
            &[
                ("universeIds", id_list),
                ("size", "512x512".to_owned()),
                ("format", "Png".to_owned()),
                ("isCircular", "false".to_owned()),
            ],
        )
        .await?;
        for entry in response.data {
            if entry.state == "Completed"
                && let Some(url) = entry.image_url
            {
                result.insert(entry.target_id, url);
            }
        }
    }
    Ok(result)
}

async fn fetch_asset_icon_urls(client: &Client, ids: &[u64]) -> HashMap<u64, String> {
    let mut result = HashMap::new();
    for chunk in ids.chunks(50) {
        let id_list = chunk
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let response: AssetThumbnailsResponse = match get_json(
            client,
            "https://thumbnails.roblox.com/v1/assets",
            &[
                ("assetIds", id_list),
                ("size", "420x420".to_owned()),
                ("format", "Png".to_owned()),
            ],
        )
        .await
        {
            Ok(response) => response,
            Err(_) => continue,
        };
        for entry in response.data {
            if entry.state == "Completed"
                && let Some(url) = entry.image_url
            {
                result.insert(entry.target_id, url);
            }
        }
    }
    result
}

async fn fetch_details(client: &Client, universe_id: u64) -> Result<DetailsBundle, ApiError> {
    let id = universe_id.to_string();
    let details_query = [("universeIds", id.clone())];
    let votes_query = [("universeIds", id.clone())];
    let media_url = format!("https://games.roblox.com/v2/games/{universe_id}/media");
    let empty_query: [(&str, String); 0] = [];
    let icon_ids = [universe_id];
    let (details_res, votes_res, media_res, icons_res) = tokio::join!(
        get_json::<GamesResponse>(client, "https://games.roblox.com/v1/games", &details_query,),
        get_json::<VotesResponse>(
            client,
            "https://games.roblox.com/v1/games/votes",
            &votes_query,
        ),
        get_json::<MediaResponse>(client, &media_url, &empty_query),
        fetch_icons(client, &icon_ids),
    );

    let details: GameDetails =
        details_res?.data.into_iter().next().ok_or_else(|| {
            ApiError::Empty(format!("no game details for universe {universe_id}"))
        })?;

    let mut warnings = Vec::new();
    let votes = match votes_res {
        Ok(response) => response.data.first().map(|entry| Votes {
            up_votes: entry.up_votes,
            down_votes: entry.down_votes,
        }),
        Err(error) => {
            warnings.push(format!("Votes unavailable: {}", error.user_message()));
            None
        }
    };

    let media_assets = match media_res {
        Ok(response) => response
            .data
            .into_iter()
            .filter(|asset| asset.asset_type == "Image" && asset.image_id > 0)
            .take(6)
            .collect::<Vec<_>>(),
        Err(error) => {
            warnings.push(format!("Media unavailable: {}", error.user_message()));
            Vec::new()
        }
    };

    let media_urls = if media_assets.is_empty() {
        HashMap::new()
    } else {
        match fetch_asset_thumbnails(
            client,
            &media_assets
                .iter()
                .map(|asset| asset.image_id)
                .collect::<Vec<_>>(),
        )
        .await
        {
            Ok(urls) => urls,
            Err(error) => {
                warnings.push(format!(
                    "Media thumbnails unavailable: {}",
                    error.user_message()
                ));
                HashMap::new()
            }
        }
    };

    let media = media_assets
        .into_iter()
        .map(|asset| MediaItem {
            asset_id: asset.image_id,
            url: media_urls.get(&asset.image_id).cloned(),
            alt_text: asset.alt_text,
        })
        .collect();

    let thumbnail_url = match icons_res {
        Ok(icons) => icons.get(&universe_id).cloned(),
        Err(error) => {
            warnings.push(format!("Game icon unavailable: {}", error.user_message()));
            None
        }
    };

    Ok(DetailsBundle {
        details,
        votes,
        thumbnail_url,
        media,
        warnings,
    })
}

async fn fetch_asset_thumbnails(
    client: &Client,
    ids: &[u64],
) -> Result<HashMap<u64, String>, ApiError> {
    let id_list = ids.iter().map(u64::to_string).collect::<Vec<_>>().join(",");
    let response: AssetThumbnailsResponse = get_json(
        client,
        "https://thumbnails.roblox.com/v1/assets",
        &[
            ("assetIds", id_list),
            ("size", "768x432".to_owned()),
            ("format", "Png".to_owned()),
        ],
    )
    .await?;
    Ok(response
        .data
        .into_iter()
        .filter_map(|thumb: AssetThumbnail| {
            if thumb.state == "Completed" {
                thumb.image_url.map(|url| (thumb.target_id, url))
            } else {
                None
            }
        })
        .collect())
}

#[allow(dead_code)]
async fn fetch_place_universe(client: &Client, place_id: u64) -> Result<u64, ApiError> {
    let response: PlaceUniverseResponse = get_json(
        client,
        &format!("https://apis.roblox.com/universes/v1/places/{place_id}/universe"),
        &[],
    )
    .await?;
    if response.universe_id == 0 {
        return Err(ApiError::Empty(format!("no universe for place {place_id}")));
    }
    Ok(response.universe_id)
}

async fn fetch_account(client: &Client, token: &str) -> Result<AuthenticatedUser, ApiError> {
    get_json_cookie(
        client,
        "https://users.roblox.com/v1/users/authenticated",
        token,
        &[],
    )
    .await
}

async fn fetch_friends(
    client: &Client,
    token: &str,
    user_id: u64,
) -> Result<Vec<Friend>, ApiError> {
    let response: FriendsResponse = get_json_cookie(
        client,
        &format!("https://friends.roblox.com/v1/users/{user_id}/friends"),
        token,
        &[],
    )
    .await?;
    let mut friends = response.data;
    if !friends.is_empty() {
        let ids: Vec<u64> = friends.iter().take(100).map(|friend| friend.id).collect();
        let body = serde_json::json!({
            "userIds": ids,
            "excludeBannedUsers": true,
        });
        match post_json_public::<UsersResponse>(client, "https://users.roblox.com/v1/users", &body)
            .await
        {
            Ok(profiles) => {
                let profiles: HashMap<u64, AuthenticatedUser> = profiles
                    .data
                    .into_iter()
                    .map(|profile| (profile.id, profile))
                    .collect();
                for friend in &mut friends {
                    if let Some(profile) = profiles.get(&friend.id) {
                        friend.name = profile.name.clone();
                        friend.display_name = profile.display_name.clone();
                    }
                }
            }
            Err(error) => log::warn!("Could not resolve friend names: {}", error.user_message()),
        }
    }
    Ok(friends)
}

async fn fetch_presence(
    client: &Client,
    token: &str,
    friend_ids: &[u64],
) -> Result<HashMap<u64, Presence>, ApiError> {
    let body = serde_json::json!({ "userIds": friend_ids });
    let response: PresenceResponse = post_json_cookie(
        client,
        "https://presence.roblox.com/v1/presence/users",
        token,
        &body,
    )
    .await?;
    let mut map = HashMap::new();
    for entry in response.entries {
        map.insert(
            entry.user_id,
            Presence {
                presence_type: entry.presence_type.unwrap_or(0),
                last_location: entry.last_location.unwrap_or_default(),
                place_id: entry.place_id.unwrap_or(0),
                universe_id: entry.universe_id.unwrap_or(0),
            },
        );
    }
    Ok(map)
}

async fn fetch_robux(client: &Client, token: &str, user_id: u64) -> Result<u64, ApiError> {
    let response: CurrencyResponse = get_json_cookie(
        client,
        &format!("https://economy.roblox.com/v1/users/{user_id}/currency"),
        token,
        &[],
    )
    .await?;
    Ok(response.robux)
}

async fn fetch_user_headshots(
    client: &Client,
    ids: &[u64],
) -> Result<HashMap<u64, String>, ApiError> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let id_list = ids.iter().map(u64::to_string).collect::<Vec<_>>().join(",");
    let response: AssetThumbnailsResponse = get_json(
        client,
        "https://thumbnails.roblox.com/v1/users/avatar-headshot",
        &[
            ("userIds", id_list),
            ("size", "420x420".to_owned()),
            ("format", "Png".to_owned()),
            ("isCircular", "false".to_owned()),
            (
                "v",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
                    .to_string(),
            ),
        ],
    )
    .await?;
    Ok(response
        .data
        .into_iter()
        .filter_map(|thumb: AssetThumbnail| {
            if thumb.state == "Completed" {
                thumb.image_url.map(|url| (thumb.target_id, url))
            } else {
                None
            }
        })
        .collect())
}

async fn fetch_user_avatar(client: &Client, user_id: u64) -> Result<String, ApiError> {
    let response: AssetThumbnailsResponse = get_json(
        client,
        "https://thumbnails.roblox.com/v1/users/avatar",
        &[
            ("userIds", user_id.to_string()),
            ("size", "420x420".to_owned()),
            ("format", "Png".to_owned()),
            ("isCircular", "false".to_owned()),
        ],
    )
    .await?;
    response
        .data
        .into_iter()
        .find_map(|thumb| {
            if thumb.state == "Completed" && thumb.target_id == user_id {
                thumb.image_url
            } else {
                None
            }
        })
        .ok_or_else(|| ApiError::Empty("no full-body avatar render".to_owned()))
}

async fn fetch_profile(client: &Client, user_id: u64) -> Result<ProfileBundle, ApiError> {
    let profile: PublicProfile = get_json(
        client,
        &format!("https://users.roblox.com/v1/users/{user_id}"),
        &[],
    )
    .await?;
    let friends: CountResponse = get_json(
        client,
        &format!("https://friends.roblox.com/v1/users/{user_id}/friends/count"),
        &[],
    )
    .await
    .unwrap_or(CountResponse { count: 0 });
    let followers: CountResponse = get_json(
        client,
        &format!("https://friends.roblox.com/v1/users/{user_id}/followers/count"),
        &[],
    )
    .await
    .unwrap_or(CountResponse { count: 0 });
    let following: CountResponse = get_json(
        client,
        &format!("https://friends.roblox.com/v1/users/{user_id}/followings/count"),
        &[],
    )
    .await
    .unwrap_or(CountResponse { count: 0 });
    let avatar: AvatarDetails = get_json(
        client,
        &format!("https://avatar.roblox.com/v1/users/{user_id}/avatar"),
        &[],
    )
    .await
    .unwrap_or_default();
    let avatar_url = fetch_user_avatar(client, user_id).await.ok();
    let asset_ids: Vec<u64> = avatar.assets.iter().map(|asset| asset.id).collect();
    let asset_urls = fetch_asset_icon_urls(client, &asset_ids).await;
    Ok(ProfileBundle {
        profile,
        friends: friends.count,
        followers: followers.count,
        following: following.count,
        avatar_url,
        assets: avatar.assets,
        emotes: avatar.emotes,
        body_colors: avatar.body_colors,
        scales: avatar.scales,
        player_avatar_type: avatar.player_avatar_type,
        asset_urls,
    })
}

#[derive(Deserialize)]
struct BundlesResponse {
    #[serde(default)]
    data: Vec<AvatarBundle>,
}

#[derive(Deserialize)]
struct BundleDetails {
    #[serde(default)]
    items: Vec<BundleItem>,
}

#[derive(Deserialize)]
struct BundleItem {
    #[serde(default)]
    id: u64,
    #[serde(rename = "assetType", default)]
    asset_type: u64,
}

#[derive(Deserialize)]
struct PlatformOutfitsResponse {
    #[serde(default)]
    data: Vec<PlatformOutfit>,
}

#[derive(Deserialize)]
struct OutfitDetails {
    #[serde(default)]
    assets: Vec<crate::models::AvatarAsset>,
}

async fn fetch_avatar_bundles(
    client: &Client,
    user_id: u64,
) -> Result<AvatarBundlesBundle, ApiError> {
    let response: BundlesResponse = get_json(
        client,
        &format!("https://catalog.roblox.com/v1/users/{user_id}/bundles"),
        &[("limit", "100".to_owned())],
    )
    .await?;
    let ids: Vec<u64> = response.data.iter().map(|bundle| bundle.id).collect();
    let urls = fetch_bundle_icon_urls(client, &ids).await;
    Ok(AvatarBundlesBundle {
        bundles: response.data,
        urls,
    })
}

async fn fetch_platform_outfits(
    client: &Client,
    token: &str,
    user_id: u64,
) -> Result<PlatformOutfitsBundle, ApiError> {
    let response: PlatformOutfitsResponse = get_json_cookie(
        client,
        &format!("https://avatar.roblox.com/v2/avatar/users/{user_id}/outfits"),
        token,
        &[("page", "1".to_owned()), ("itemsPerPage", "100".to_owned())],
    )
    .await?;
    let ids: Vec<u64> = response.data.iter().map(|outfit| outfit.id).collect();
    let urls = fetch_outfit_icon_urls(client, &ids).await;
    Ok(PlatformOutfitsBundle {
        outfits: response.data,
        urls,
    })
}

async fn fetch_bundle_icon_urls(client: &Client, ids: &[u64]) -> HashMap<u64, String> {
    let mut result = HashMap::new();
    for chunk in ids.chunks(50) {
        let id_list = chunk
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let response: AssetThumbnailsResponse = match get_json(
            client,
            "https://thumbnails.roblox.com/v1/bundles/thumbnails",
            &[
                ("bundleIds", id_list),
                ("size", "420x420".to_owned()),
                ("format", "Png".to_owned()),
            ],
        )
        .await
        {
            Ok(response) => response,
            Err(_) => continue,
        };
        for entry in response.data {
            if entry.state == "Completed"
                && let Some(url) = entry.image_url
            {
                result.insert(entry.target_id, url);
            }
        }
    }
    result
}

async fn fetch_outfit_icon_urls(client: &Client, ids: &[u64]) -> HashMap<u64, String> {
    let mut result = HashMap::new();
    for chunk in ids.chunks(50) {
        let id_list = chunk
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let response: AssetThumbnailsResponse = match get_json(
            client,
            "https://thumbnails.roblox.com/v1/users/outfits",
            &[
                ("userOutfitIds", id_list),
                ("size", "420x420".to_owned()),
                ("format", "Png".to_owned()),
            ],
        )
        .await
        {
            Ok(response) => response,
            Err(_) => continue,
        };
        for entry in response.data {
            if entry.state == "Completed"
                && let Some(url) = entry.image_url
            {
                result.insert(entry.target_id, url);
            }
        }
    }
    result
}

async fn equip_avatar_bundle(client: &Client, token: &str, bundle_id: u64) -> Result<(), ApiError> {
    let details: BundleDetails = get_json(
        client,
        &format!("https://catalog.roblox.com/v1/bundles/{bundle_id}/details"),
        &[],
    )
    .await?;
    let current: AvatarDetails =
        get_json_cookie(client, "https://avatar.roblox.com/v1/avatar", token, &[])
            .await
            .unwrap_or_default();
    let replaced: HashSet<u64> = details
        .items
        .iter()
        .filter(|item| item.asset_type > 0)
        .map(|item| item.asset_type)
        .collect();
    let mut ids: Vec<u64> = current
        .assets
        .into_iter()
        .filter(|asset| !replaced.contains(&asset.asset_type.id))
        .map(|asset| asset.id)
        .collect();
    ids.extend(
        details
            .items
            .into_iter()
            .filter(|item| item.id > 0 && item.asset_type > 0)
            .map(|item| item.id),
    );
    ids.sort_unstable();
    ids.dedup();
    post_wearing_ids(client, token, &ids).await
}

async fn equip_platform_outfit(
    client: &Client,
    token: &str,
    outfit_id: u64,
) -> Result<(), ApiError> {
    let details: OutfitDetails = get_json_cookie(
        client,
        &format!("https://avatar.roblox.com/v3/outfits/{outfit_id}/details"),
        token,
        &[],
    )
    .await?;
    let ids: Vec<u64> = details.assets.iter().map(|asset| asset.id).collect();
    post_wearing_ids(client, token, &ids).await
}

async fn post_wearing_ids(client: &Client, token: &str, ids: &[u64]) -> Result<(), ApiError> {
    let assets: Vec<serde_json::Value> = ids
        .iter()
        .map(|id| serde_json::json!({ "id": id }))
        .collect();
    post_empty_cookie_csrf(
        client,
        "https://avatar.roblox.com/v2/avatar/set-wearing-assets",
        token,
        &serde_json::json!({ "assets": assets }),
    )
    .await
}

async fn fetch_inventory(
    client: &Client,
    token: &str,
    user_id: u64,
    asset_type: u64,
) -> Result<InventoryBundle, ApiError> {
    let response: InventoryResponse = get_json_cookie(
        client,
        &format!("https://inventory.roblox.com/v2/users/{user_id}/inventory/{asset_type}"),
        token,
        &[("limit", "100".to_owned()), ("sortOrder", "Asc".to_owned())],
    )
    .await?;
    let items: Vec<InventoryItem> = response
        .data
        .into_iter()
        .filter(|entry| entry.asset_id > 0)
        .map(|entry| InventoryItem {
            asset_id: entry.asset_id,
            name: entry.name,
            asset_type,
        })
        .collect();
    let ids: Vec<u64> = items.iter().map(|item| item.asset_id).collect();
    let urls = fetch_asset_icon_urls(client, &ids).await;
    Ok(InventoryBundle { items, urls })
}

async fn post_avatar(
    client: &Client,
    token: &str,
    assets: &[crate::models::AvatarAsset],
) -> Result<(), ApiError> {
    // The v3 route no longer exists on the live site (it returns a bare
    // 404). The current web avatar editor uses v2; the request only needs
    // the asset ids, not the asset type metadata.
    let assets: Vec<serde_json::Value> = assets
        .iter()
        .map(|asset| serde_json::json!({ "id": asset.id }))
        .collect();
    post_empty_cookie_csrf(
        client,
        "https://avatar.roblox.com/v2/avatar/set-wearing-assets",
        token,
        &serde_json::json!({ "assets": assets }),
    )
    .await
}

async fn post_body_colors(
    client: &Client,
    token: &str,
    colors: &BodyColors,
) -> Result<(), ApiError> {
    post_empty_cookie_csrf(
        client,
        "https://avatar.roblox.com/v1/avatar/set-body-colors",
        token,
        &serde_json::json!({
            "headColorId": colors.head,
            "torsoColorId": colors.torso,
            "rightArmColorId": colors.right_arm,
            "leftArmColorId": colors.left_arm,
            "rightLegColorId": colors.right_leg,
            "leftLegColorId": colors.left_leg,
        }),
    )
    .await
}

async fn post_avatar_type(client: &Client, token: &str, avatar_type: &str) -> Result<(), ApiError> {
    post_empty_cookie_csrf(
        client,
        "https://avatar.roblox.com/v1/avatar/set-player-avatar-type",
        token,
        &serde_json::json!({ "playerAvatarType": avatar_type }),
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn post_emote(
    client: &Client,
    token: &str,
    asset_id: u64,
    position: u64,
) -> Result<(), ApiError> {
    post_empty_cookie_csrf(
        client,
        &format!("https://avatar.roblox.com/v1/emotes/{asset_id}/{position}"),
        token,
        &serde_json::json!({}),
    )
    .await
}

async fn delete_emote(client: &Client, token: &str, position: u64) -> Result<(), ApiError> {
    let csrf = csrf_token(client, token).await?;
    let response = client
        .delete(format!("https://avatar.roblox.com/v1/emotes/{position}"))
        .header("Accept", "application/json")
        .header("Cookie", format!(".ROBLOSECURITY={token}"))
        .header("X-CSRF-TOKEN", csrf)
        .send()
        .await
        .map_err(|error| ApiError::Network(error.to_string()))?;
    let status = response.status();
    if status.is_success() {
        return Ok(());
    }
    let detail = response
        .text()
        .await
        .unwrap_or_default()
        .replace(['\n', '\r'], " ");
    Err(ApiError::Http {
        status: status.as_u16(),
        detail: detail.chars().take(160).collect(),
    })
}

async fn post_scales(
    client: &Client,
    token: &str,
    height: f64,
    width: f64,
    head: f64,
    depth: f64,
    proportion: f64,
    body_type: f64,
) -> Result<(), ApiError> {
    post_empty_cookie_csrf(
        client,
        "https://avatar.roblox.com/v1/avatar/set-scales",
        token,
        &serde_json::json!({
            "height": height,
            "width": width,
            "head": head,
            "depth": depth,
            "proportion": proportion,
            "bodyType": body_type,
        }),
    )
    .await
}

async fn fetch_favorites(client: &Client, user_id: u64) -> Result<Vec<GameSummary>, ApiError> {
    let response: FavoritesResponse = get_json(
        client,
        &format!("https://games.roblox.com/v2/users/{user_id}/favorite/games"),
        &[("limit", "50".to_owned()), ("sortOrder", "Desc".to_owned())],
    )
    .await?;
    Ok(response
        .data
        .into_iter()
        .filter_map(favorite_to_summary)
        .collect())
}

async fn fetch_favorite_status(
    client: &Client,
    token: &str,
    universe_id: u64,
) -> Result<bool, ApiError> {
    let response: GameFavoriteResponse = get_json_cookie(
        client,
        &format!("https://games.roblox.com/v1/games/{universe_id}/favorites"),
        token,
        &[],
    )
    .await?;
    Ok(response.is_favorited)
}

async fn post_favorite(
    client: &Client,
    token: &str,
    universe_id: u64,
    is_favorited: bool,
) -> Result<(), ApiError> {
    post_empty_cookie_csrf(
        client,
        &format!("https://games.roblox.com/v1/games/{universe_id}/favorites"),
        token,
        &serde_json::json!({ "isFavorited": is_favorited }),
    )
    .await
}

fn favorite_to_summary(entry: FavoriteGameEntry) -> Option<GameSummary> {
    let root_place_id = entry.root_place.map(|place| place.id).unwrap_or_default();
    if entry.id == 0 || root_place_id == 0 {
        return None;
    }
    Some(GameSummary {
        universe_id: entry.id,
        root_place_id,
        name: entry.name,
        player_count: 0,
        up_votes: 0,
        down_votes: 0,
        is_sponsored: false,
        creator_name: entry
            .creator
            .map(|creator| creator.name)
            .unwrap_or_default(),
        genre_l1: String::new(),
        content_type: "Game".to_owned(),
    })
}

async fn fetch_chat_conversations(
    client: &Client,
    token: &str,
) -> Result<Vec<ChatConversation>, ApiError> {
    let value: serde_json::Value = get_json_cookie(
        client,
        "https://apis.roblox.com/platform-chat-api/v1/get-user-conversations",
        token,
        &[
            ("include_user_data", "true".to_owned()),
            ("pageSize", "100".to_owned()),
        ],
    )
    .await?;
    let entries = value
        .get("conversations")
        .and_then(serde_json::Value::as_array)
        .or_else(|| value.as_array())
        .cloned()
        .unwrap_or_default();
    Ok(entries
        .into_iter()
        .filter_map(|entry| serde_json::from_value(entry).ok())
        .collect())
}

async fn fetch_chat_messages(
    client: &Client,
    token: &str,
    conversation_id: &str,
) -> Result<Vec<ChatMessage>, ApiError> {
    let value: serde_json::Value = get_json_cookie(
        client,
        "https://apis.roblox.com/platform-chat-api/v1/get-conversation-messages",
        token,
        &[
            ("conversation_id", conversation_id.to_owned()),
            ("pageSize", "100".to_owned()),
        ],
    )
    .await?;
    let entries = value
        .get("messages")
        .and_then(serde_json::Value::as_array)
        .or_else(|| {
            value
                .get("conversation")
                .and_then(|conversation| conversation.get("messages"))
                .and_then(serde_json::Value::as_array)
        })
        .or_else(|| value.as_array())
        .cloned()
        .unwrap_or_default();
    Ok(entries
        .into_iter()
        .filter_map(|entry| serde_json::from_value(entry).ok())
        .collect())
}

async fn post_chat_message(
    client: &Client,
    token: &str,
    conversation_id: &str,
    message: &str,
) -> Result<(), ApiError> {
    post_empty_cookie_csrf(
        client,
        "https://apis.roblox.com/platform-chat-api/v1/send-messages",
        token,
        &serde_json::json!({
            "conversation_id": conversation_id,
            "messages": [{ "content": message }]
        }),
    )
    .await
}

async fn post_start_chat(
    client: &Client,
    token: &str,
    owner_id: u64,
    user_id: u64,
) -> Result<String, ApiError> {
    let csrf = csrf_token(client, token).await?;
    let mut last_detail = String::new();
    let mut last_status = 400;
    // The current platform API is inconsistent about whether a direct
    // conversation is a one-to-one or a two-person group. Try the direct
    // form first, then the old web form, then a group containing both
    // participants. Rejected requests do not create anything, so the
    // fallbacks are safe.
    let mut candidates: Vec<(&str, Vec<u64>)> =
        vec![("one_to_one", vec![user_id]), ("group", vec![user_id])];
    if owner_id != user_id {
        candidates.push(("group", vec![owner_id, user_id]));
    }
    for (kind, participants) in candidates {
        let mut conversation = serde_json::json!({
            "type": kind,
            "participant_user_ids": participants
        });
        if kind == "group" {
            conversation["name"] = serde_json::json!("");
        }
        let response = client
            .post("https://apis.roblox.com/platform-chat-api/v1/create-conversations")
            .header("Accept", "application/json")
            .header("Cookie", format!(".ROBLOSECURITY={token}"))
            .header("X-CSRF-TOKEN", csrf.clone())
            .json(&serde_json::json!({
                "conversations": [conversation],
                "include_user_data": false
            }))
            .send()
            .await
            .map_err(|error| ApiError::Network(error.to_string()))?;
        let status = response.status();
        if status.is_success() {
            let value: serde_json::Value = response
                .json()
                .await
                .map_err(|error| ApiError::Decode(error.to_string()))?;
            if let Some(id) = value
                .get("conversations")
                .and_then(serde_json::Value::as_array)
                .and_then(|conversations| conversations.first())
                .and_then(|conversation| conversation.get("id"))
                .and_then(|id| {
                    id.as_str()
                        .map(str::to_owned)
                        .or_else(|| id.as_u64().map(|id| id.to_string()))
                })
            {
                return Ok(id);
            }
            return Err(ApiError::Empty("no conversation id".to_owned()));
        }
        last_status = status.as_u16();
        last_detail = response
            .text()
            .await
            .unwrap_or_default()
            .replace(['\n', '\r'], " ");
        log::warn!(
            "create-conversations type={kind} participants={participants:?} failed: HTTP {last_status}: {}",
            last_detail.chars().take(300).collect::<String>()
        );
        if last_status != 400 {
            break;
        }
    }
    Err(ApiError::Http {
        status: last_status,
        detail: last_detail.chars().take(300).collect(),
    })
}

async fn fetch_username(client: &Client, query: &str) -> Result<AuthenticatedUser, ApiError> {
    let body = serde_json::json!({
        "usernames": [query],
        "excludeBannedUsers": true
    });
    let response: UsersResponse =
        post_json_public(client, "https://users.roblox.com/v1/usernames/users", &body).await?;
    response
        .data
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::Empty(format!("no user named {query}")))
}

async fn post_friend_request(client: &Client, token: &str, user_id: u64) -> Result<(), ApiError> {
    post_empty_cookie_csrf(
        client,
        &format!("https://friends.roblox.com/v1/users/{user_id}/request-friendship"),
        token,
        &serde_json::json!({}),
    )
    .await
}

async fn csrf_token(client: &Client, token: &str) -> Result<String, ApiError> {
    let response = client
        .post("https://auth.roblox.com/v2/logout")
        .header("Accept", "application/json")
        .header("Cookie", format!(".ROBLOSECURITY={token}"))
        .send()
        .await
        .map_err(|error| ApiError::Network(error.to_string()))?;
    if let Some(value) = response.headers().get("x-csrf-token")
        && let Ok(value) = value.to_str()
    {
        return Ok(value.to_owned());
    }
    Err(ApiError::Http {
        status: response.status().as_u16(),
        detail: "Roblox did not return an X-CSRF-TOKEN".to_owned(),
    })
}

async fn post_empty_cookie_csrf(
    client: &Client,
    url: &str,
    token: &str,
    body: &serde_json::Value,
) -> Result<(), ApiError> {
    let csrf = csrf_token(client, token).await?;
    let response = client
        .post(url)
        .header("Accept", "application/json")
        .header("Cookie", format!(".ROBLOSECURITY={token}"))
        .header("X-CSRF-TOKEN", csrf)
        .json(body)
        .send()
        .await
        .map_err(|error| ApiError::Network(error.to_string()))?;
    let status = response.status();
    if status.is_success() {
        return Ok(());
    }
    let detail = response
        .text()
        .await
        .unwrap_or_default()
        .replace(['\n', '\r'], " ");
    Err(ApiError::Http {
        status: status.as_u16(),
        detail: detail.chars().take(160).collect(),
    })
}

async fn get_json<T: DeserializeOwned>(
    client: &Client,
    url: &str,
    query: &[(&str, String)],
) -> Result<T, ApiError> {
    get_json_inner(client, url, query, None).await
}

async fn get_json_cookie<T: DeserializeOwned>(
    client: &Client,
    url: &str,
    token: &str,
    query: &[(&str, String)],
) -> Result<T, ApiError> {
    get_json_inner(client, url, query, Some(token)).await
}

async fn get_json_inner<T: DeserializeOwned>(
    client: &Client,
    url: &str,
    query: &[(&str, String)],
    token: Option<&str>,
) -> Result<T, ApiError> {
    let mut last_error = None;
    for attempt in 0..3 {
        let mut request = client
            .get(url)
            .query(query)
            .header("Accept", "application/json");
        if let Some(token) = token {
            request = request.header("Cookie", format!(".ROBLOSECURITY={token}"));
        }
        let response = request.send().await;
        let response = match response {
            Ok(response) => response,
            Err(error) => {
                last_error = Some(ApiError::Network(error.to_string()));
                if attempt < 2 {
                    tokio::time::sleep(Duration::from_millis(400 * (1 << attempt))).await;
                    continue;
                }
                break;
            }
        };

        let status = response.status();
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
            last_error = Some(ApiError::Http {
                status: status.as_u16(),
                detail: format!("retryable response from {url}"),
            });
            if attempt < 2 {
                tokio::time::sleep(Duration::from_millis(400 * (1 << attempt))).await;
                continue;
            }
            break;
        }

        if !status.is_success() {
            let detail = response
                .text()
                .await
                .unwrap_or_default()
                .replace(['\n', '\r'], " ");
            let detail = detail.chars().take(160).collect::<String>();
            return Err(ApiError::Http {
                status: status.as_u16(),
                detail,
            });
        }

        return response
            .json::<T>()
            .await
            .map_err(|error| ApiError::Decode(format!("{url}: {error}")));
    }

    Err(last_error.unwrap_or_else(|| ApiError::Network("request failed".to_owned())))
}

async fn post_json_cookie<T: DeserializeOwned>(
    client: &Client,
    url: &str,
    token: &str,
    body: &serde_json::Value,
) -> Result<T, ApiError> {
    post_json_inner(client, url, Some(token), body).await
}

async fn post_json_public<T: DeserializeOwned>(
    client: &Client,
    url: &str,
    body: &serde_json::Value,
) -> Result<T, ApiError> {
    post_json_inner(client, url, None, body).await
}

async fn post_json_inner<T: DeserializeOwned>(
    client: &Client,
    url: &str,
    token: Option<&str>,
    body: &serde_json::Value,
) -> Result<T, ApiError> {
    let mut request = client
        .post(url)
        .header("Accept", "application/json")
        .json(body);
    if let Some(token) = token {
        request = request.header("Cookie", format!(".ROBLOSECURITY={token}"));
    }
    let response = request
        .send()
        .await
        .map_err(|error| ApiError::Network(error.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        let detail = response
            .text()
            .await
            .unwrap_or_default()
            .replace(['\n', '\r'], " ");
        let detail = detail.chars().take(160).collect::<String>();
        return Err(ApiError::Http {
            status: status.as_u16(),
            detail,
        });
    }

    response
        .json::<T>()
        .await
        .map_err(|error| ApiError::Decode(format!("{url}: {error}")))
}

pub fn load_state_from_result<T>(result: Result<T, ApiError>) -> Load<T> {
    match result {
        Ok(value) => Load::Ready(value),
        Err(error) => {
            log::warn!("API error: {}", error.detail());
            Load::Failed(error.user_message())
        }
    }
}
