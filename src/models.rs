// API models mirror the live responses; not every field is displayed yet.
#![allow(dead_code)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

fn string_or_empty<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?.unwrap_or_default())
}

fn vec_or_default<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Clone, Debug, Deserialize)]
pub struct GameSummary {
    #[serde(rename = "universeId")]
    pub universe_id: u64,
    #[serde(rename = "rootPlaceId")]
    pub root_place_id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "playerCount", default)]
    pub player_count: u64,
    #[serde(rename = "totalUpVotes", default)]
    pub up_votes: u64,
    #[serde(rename = "totalDownVotes", default)]
    pub down_votes: u64,
    #[serde(rename = "isSponsored", default)]
    pub is_sponsored: bool,
    #[serde(rename = "creatorName", default)]
    pub creator_name: String,
    #[serde(rename = "genreL1", default)]
    pub genre_l1: String,
    #[serde(rename = "contentType", default)]
    pub content_type: String,
}

impl GameSummary {
    pub fn is_game(&self) -> bool {
        self.universe_id > 0
            && self.root_place_id > 0
            && (self.content_type.is_empty() || self.content_type == "Game")
    }
}

#[derive(Clone, Debug)]
pub struct SortInfo {
    pub sort_id: String,
    pub display_name: String,
}

#[derive(Deserialize)]
pub struct SortsResponse {
    #[serde(default)]
    pub sorts: Vec<RawSort>,
}

#[derive(Deserialize)]
pub struct SortIdsResponse {
    #[serde(rename = "genreToSortId", default)]
    pub genre_to_sort_id: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct RawSort {
    #[serde(rename = "sortId")]
    pub sort_id: String,
    #[serde(rename = "sortDisplayName", default)]
    pub display_name: String,
    #[serde(rename = "contentType", default)]
    pub content_type: String,
    #[serde(rename = "treatmentType", default)]
    pub treatment_type: String,
}

#[derive(Deserialize)]
pub struct SortContentResponse {
    #[serde(default)]
    pub games: Vec<GameSummary>,
}

#[derive(Deserialize)]
pub struct SearchResponse {
    #[serde(rename = "searchResults", default)]
    pub search_results: Vec<SearchGroup>,
}

#[derive(Deserialize)]
pub struct SearchGroup {
    #[serde(default)]
    pub contents: Vec<GameSummary>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Creator {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "type", default)]
    pub creator_type: String,
    #[serde(rename = "hasVerifiedBadge", default)]
    pub verified: bool,
}

impl Default for Creator {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            creator_type: String::new(),
            verified: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GameDetails {
    pub id: u64,
    #[serde(rename = "rootPlaceId")]
    pub root_place_id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub creator: Creator,
    #[serde(default)]
    pub playing: u64,
    #[serde(default)]
    pub visits: u64,
    #[serde(rename = "maxPlayers", default)]
    pub max_players: u64,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub updated: String,
    #[serde(rename = "favoritedCount", default)]
    pub favorited_count: u64,
    #[serde(rename = "genre_l1", default)]
    pub genre_l1: String,
    #[serde(rename = "isSponsored", default)]
    pub is_sponsored: bool,
}

#[derive(Deserialize)]
pub struct GamesResponse {
    #[serde(default)]
    pub data: Vec<GameDetails>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub struct Votes {
    #[serde(rename = "upVotes", default)]
    pub up_votes: u64,
    #[serde(rename = "downVotes", default)]
    pub down_votes: u64,
}

impl Votes {
    pub fn total(self) -> u64 {
        self.up_votes + self.down_votes
    }

    pub fn positive_ratio(self) -> f32 {
        let total = self.total();
        if total == 0 {
            0.0
        } else {
            self.up_votes as f32 / total as f32
        }
    }
}

#[derive(Deserialize)]
pub struct VotesResponse {
    #[serde(default)]
    pub data: Vec<VotesEntry>,
}

#[derive(Deserialize)]
pub struct VotesEntry {
    #[serde(rename = "upVotes", default)]
    pub up_votes: u64,
    #[serde(rename = "downVotes", default)]
    pub down_votes: u64,
}

#[derive(Deserialize)]
pub struct MediaResponse {
    #[serde(default)]
    pub data: Vec<MediaAsset>,
}

#[derive(Deserialize)]
pub struct MediaAsset {
    #[serde(rename = "assetType", default)]
    pub asset_type: String,
    #[serde(rename = "imageId", default)]
    pub image_id: u64,
    #[serde(rename = "altText", default)]
    pub alt_text: String,
}

#[derive(Clone, Debug)]
pub struct MediaItem {
    pub asset_id: u64,
    pub url: Option<String>,
    pub alt_text: String,
}

#[derive(Deserialize)]
pub struct AssetThumbnailsResponse {
    #[serde(default)]
    pub data: Vec<AssetThumbnail>,
}

#[derive(Deserialize)]
pub struct AssetThumbnail {
    #[serde(rename = "targetId", default)]
    pub target_id: u64,
    #[serde(rename = "state", default)]
    pub state: String,
    #[serde(rename = "imageUrl", default)]
    pub image_url: Option<String>,
}

#[derive(Deserialize)]
pub struct PlaceUniverseResponse {
    #[serde(rename = "universeId", default)]
    pub universe_id: u64,
}

#[derive(Clone, Debug)]
pub struct DetailsBundle {
    pub details: GameDetails,
    pub votes: Option<Votes>,
    pub thumbnail_url: Option<String>,
    pub media: Vec<MediaItem>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AuthenticatedUser {
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "displayName", default)]
    pub display_name: String,
    #[serde(default)]
    pub created: String,
}

impl AuthenticatedUser {
    pub fn display(&self) -> &str {
        if self.display_name.is_empty() {
            &self.name
        } else {
            &self.display_name
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Friend {
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "displayName", default)]
    pub display_name: String,
}

impl Friend {
    pub fn display(&self) -> &str {
        if self.display_name.is_empty() {
            &self.name
        } else {
            &self.display_name
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Presence {
    pub presence_type: u8,
    pub last_location: String,
    pub place_id: u64,
    pub universe_id: u64,
}

impl Presence {
    pub fn is_online(&self) -> bool {
        self.presence_type > 0
    }

    pub fn label(&self) -> &str {
        // Roblox reports "Website" as the last location for offline users.
        // The 2016 shell showed that state as Offline.
        if self.presence_type == 0 {
            return "Offline";
        }
        if self.last_location.trim().eq_ignore_ascii_case("website") {
            return "Online";
        }
        if !self.last_location.is_empty() {
            &self.last_location
        } else {
            match self.presence_type {
                2 => "In game",
                1 => "Online",
                3 => "In Studio",
                _ => "Offline",
            }
        }
    }
}

#[derive(Deserialize)]
pub struct FriendsResponse {
    #[serde(default)]
    pub data: Vec<Friend>,
}

#[derive(Deserialize)]
pub struct UsersResponse {
    #[serde(default)]
    pub data: Vec<AuthenticatedUser>,
}

#[derive(Deserialize)]
pub struct PresenceResponse {
    #[serde(rename = "userPresences", default)]
    pub entries: Vec<PresenceEntry>,
}

#[derive(Deserialize)]
pub struct PresenceEntry {
    #[serde(rename = "userId")]
    pub user_id: u64,
    #[serde(rename = "userPresenceType", default)]
    pub presence_type: Option<u8>,
    #[serde(rename = "lastLocation", default)]
    pub last_location: Option<String>,
    #[serde(rename = "placeId", default)]
    pub place_id: Option<u64>,
    #[serde(rename = "universeId", default)]
    pub universe_id: Option<u64>,
}

#[derive(Deserialize)]
pub struct CurrencyResponse {
    #[serde(default)]
    pub robux: u64,
}

#[derive(Clone, Debug)]
pub enum Load<T> {
    Idle,
    Loading,
    Ready(T),
    Failed(String),
}

impl<T> Default for Load<T> {
    fn default() -> Self {
        Self::Idle
    }
}

impl<T> Load<T> {
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    pub fn ready(&self) -> Option<&T> {
        match self {
            Self::Ready(value) => Some(value),
            _ => None,
        }
    }

    pub fn error(&self) -> Option<&str> {
        match self {
            Self::Failed(message) => Some(message),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_website_location_reads_as_offline() {
        let presence = Presence {
            presence_type: 0,
            last_location: "Website".to_owned(),
            ..Presence::default()
        };
        assert_eq!(presence.label(), "Offline");
        assert!(!presence.is_online());
    }

    #[test]
    fn online_website_location_reads_as_online() {
        let presence = Presence {
            presence_type: 1,
            last_location: "Website".to_owned(),
            ..Presence::default()
        };
        assert_eq!(presence.label(), "Online");
        assert!(presence.is_online());
    }

    #[test]
    fn online_location_is_kept() {
        let presence = Presence {
            presence_type: 2,
            last_location: "Natural Disaster Survival".to_owned(),
            ..Presence::default()
        };
        assert_eq!(presence.label(), "Natural Disaster Survival");
        assert!(presence.is_online());
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct PublicProfile {
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "displayName", default)]
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub created: String,
}

#[derive(Deserialize)]
pub struct CountResponse {
    #[serde(default)]
    pub count: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AvatarAsset {
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "assetType")]
    pub asset_type: AvatarAssetType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AvatarAssetType {
    pub id: u64,
    #[serde(default)]
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SavedOutfit {
    pub name: String,
    pub assets: Vec<AvatarAsset>,
}

#[derive(Serialize, Deserialize)]
pub struct OutfitFile {
    #[serde(default)]
    pub user_id: u64,
    #[serde(default)]
    pub outfits: Vec<SavedOutfit>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FavoriteCreator {
    #[serde(default)]
    pub name: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FavoriteRootPlace {
    #[serde(default)]
    pub id: u64,
}

#[derive(Deserialize)]
pub struct FavoriteGameEntry {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub creator: Option<FavoriteCreator>,
    #[serde(rename = "rootPlace", default)]
    pub root_place: Option<FavoriteRootPlace>,
}

#[derive(Deserialize)]
pub struct FavoritesResponse {
    #[serde(default)]
    pub data: Vec<FavoriteGameEntry>,
}

#[derive(Deserialize)]
pub struct GameFavoriteResponse {
    #[serde(rename = "isFavorited", default)]
    pub is_favorited: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct AvatarDetails {
    #[serde(default)]
    pub assets: Vec<AvatarAsset>,
    #[serde(default)]
    pub emotes: Vec<EquippedEmote>,
    #[serde(rename = "bodyColors", default)]
    pub body_colors: Option<BodyColors>,
    #[serde(default)]
    pub scales: Option<AvatarScales>,
    #[serde(rename = "playerAvatarType", default)]
    pub player_avatar_type: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct BodyColors {
    #[serde(rename = "headColorId", default)]
    pub head: u64,
    #[serde(rename = "torsoColorId", default)]
    pub torso: u64,
    #[serde(rename = "rightArmColorId", default)]
    pub right_arm: u64,
    #[serde(rename = "leftArmColorId", default)]
    pub left_arm: u64,
    #[serde(rename = "rightLegColorId", default)]
    pub right_leg: u64,
    #[serde(rename = "leftLegColorId", default)]
    pub left_leg: u64,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct AvatarScales {
    #[serde(default)]
    pub height: f64,
    #[serde(default)]
    pub width: f64,
    #[serde(default)]
    pub head: f64,
    #[serde(default)]
    pub depth: f64,
    #[serde(default)]
    pub proportion: f64,
    #[serde(rename = "bodyType", default)]
    pub body_type: f64,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct AvatarBundle {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "bundleType", default)]
    pub bundle_type: String,
}

#[derive(Clone, Debug)]
pub struct AvatarBundlesBundle {
    pub bundles: Vec<AvatarBundle>,
    pub urls: HashMap<u64, String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct PlatformOutfit {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "isEditable", default)]
    pub is_editable: bool,
    #[serde(rename = "outfitType", default)]
    pub outfit_type: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PlatformOutfitsBundle {
    pub outfits: Vec<PlatformOutfit>,
    pub urls: HashMap<u64, String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EquippedEmote {
    #[serde(rename = "assetId", default)]
    pub asset_id: u64,
    #[serde(rename = "assetName", default)]
    pub asset_name: String,
    #[serde(default)]
    pub position: u64,
}

#[derive(Clone, Debug)]
pub struct ProfileBundle {
    pub profile: PublicProfile,
    pub friends: u64,
    pub followers: u64,
    pub following: u64,
    pub avatar_url: Option<String>,
    pub assets: Vec<AvatarAsset>,
    pub emotes: Vec<EquippedEmote>,
    pub body_colors: Option<BodyColors>,
    pub scales: Option<AvatarScales>,
    pub player_avatar_type: Option<String>,
    pub asset_urls: HashMap<u64, String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChatUserData {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "displayName", alias = "display_name", default)]
    pub display_name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChatConversation {
    #[serde(default, deserialize_with = "string_or_empty")]
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(
        rename = "participant_user_ids",
        default,
        deserialize_with = "vec_or_default"
    )]
    pub participant_user_ids: Vec<u64>,
    #[serde(rename = "user_data", default)]
    pub user_data: HashMap<String, ChatUserData>,
    #[serde(default, deserialize_with = "vec_or_default")]
    pub messages: Vec<ChatMessage>,
    #[serde(rename = "updated_at", default, deserialize_with = "string_or_empty")]
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChatMessage {
    #[serde(default, deserialize_with = "string_or_empty")]
    pub id: String,
    #[serde(rename = "sender_user_id", alias = "senderTargetId", default)]
    pub sender_user_id: u64,
    #[serde(default)]
    pub content: String,
    #[serde(rename = "created_at", alias = "created", default)]
    pub created_at: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct InventoryEntry {
    #[serde(rename = "assetId", alias = "id", default)]
    pub asset_id: u64,
    #[serde(default, alias = "assetName", alias = "displayName")]
    pub name: String,
}

#[derive(Deserialize)]
pub struct InventoryResponse {
    #[serde(default)]
    pub data: Vec<InventoryEntry>,
}

#[derive(Clone, Debug)]
pub struct InventoryItem {
    pub asset_id: u64,
    pub name: String,
    pub asset_type: u64,
}

#[derive(Clone, Debug)]
pub struct InventoryBundle {
    pub items: Vec<InventoryItem>,
    pub urls: HashMap<u64, String>,
}
