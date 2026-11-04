use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, TextureHandle, Vec2};

use crate::app::App;
use crate::audio::Sfx;
use crate::canvas::Canvas;
use crate::models::Load;
use crate::screens::{self, ButtonKind, NavTab, Screen};
use crate::theme;
use crate::util;

const GRID_X: f32 = 585.0;
const GRID_Y: f32 = 292.0;
const CARD_W: f32 = 300.0;
const CARD_H: f32 = 300.0;
const CARD_GAP: f32 = 12.0;
const CARD_STEP: f32 = CARD_W + CARD_GAP;
const GRID_W: f32 = CARD_W * 4.0 + CARD_GAP * 3.0;
const ROW0_H: f32 = CARD_H;
const ROW1_H: f32 = CARD_H;
const ROW_GAP: f32 = CARD_GAP;
const ROW1_Y: f32 = GRID_Y + ROW0_H + ROW_GAP;
const AVATAR_X: f32 = 104.0;
const AVATAR_Y: f32 = 292.0;
const AVATAR_W: f32 = 450.0;
const AVATAR_H: f32 = 290.0;
const FRIEND_LABEL_Y: f32 = 622.0;
const FRIEND_TOP: f32 = 654.0;
const FRIEND_ROW_STEP: f32 = 132.0;

struct AccountView {
    signed_in: bool,
    display_name: String,
    headshot_url: Option<String>,
    avatar_url: Option<String>,
    friends: Vec<FriendView>,
    friends_loading: bool,
    friends_error: Option<String>,
}

struct FriendView {
    user_id: u64,
    name: String,
    presence: String,
    online: bool,
    place_id: u64,
    headshot_url: Option<String>,
}

impl App {
    pub fn show_home(
        &mut self,
        canvas: &mut Canvas,
        background: &TextureHandle,
        wordmark: Option<&TextureHandle>,
    ) {
        let dim = screens::adaptive_dim(canvas, 48);
        screens::chrome(canvas, background, dim);

        if let Some(tab) = screens::nav(canvas, wordmark, NavTab::Home) {
            self.go_tab(tab);
            return;
        }

        if self.session.is_some() {
            let logout = Rect::from_min_size(
                Pos2::new(theme::CANVAS_W - theme::SAFE_X - 138.0, 180.0),
                Vec2::new(138.0, 32.0),
            );
            if screens::text_link(canvas, "home:logout", logout, "Log out", true) {
                self.sign_out();
                return;
            }
        }

        let account = self.account_view();
        let account_label = if account.signed_in {
            if account.display_name.is_empty() {
                "PLAYER".to_owned()
            } else {
                account.display_name.clone()
            }
        } else {
            "GUEST".to_owned()
        };
        let account_label = util::truncate_to_width(&account_label, AVATAR_W - 20.0, |text| {
            canvas.text_width(text, 24.0, false)
        });
        screens::section_label(canvas, 100.0, 252.0, &account_label);
        let avatar_rect =
            Rect::from_min_size(Pos2::new(AVATAR_X, AVATAR_Y), Vec2::new(AVATAR_W, AVATAR_H));
        let avatar = canvas.focusable("home:avatar", avatar_rect, Sense::click());
        canvas.gradient_v(
            avatar_rect,
            Color32::from_rgb(47, 103, 126),
            Color32::from_rgb(23, 56, 67),
        );

        if let Some(url) = &account.headshot_url {
            self.images.ensure(url);
        }
        if let Some(url) = &account.avatar_url {
            self.images.ensure(url);
        }
        for friend in &account.friends {
            if let Some(url) = &friend.headshot_url {
                self.images.ensure(url);
            }
        }
        let headshot_texture = account
            .headshot_url
            .as_ref()
            .and_then(|url| self.images.get(url))
            .cloned();
        let avatar_texture = account
            .avatar_url
            .as_ref()
            .and_then(|url| self.images.get(url))
            .cloned();
        let friend_textures: Vec<Option<TextureHandle>> = account
            .friends
            .iter()
            .map(|friend| {
                friend
                    .headshot_url
                    .as_ref()
                    .and_then(|url| self.images.get(url))
                    .cloned()
            })
            .collect();

        match &avatar_texture {
            Some(texture) => {
                canvas.image_cover_bias(texture, avatar_rect, Color32::WHITE, 0.12);
            }
            None => match &headshot_texture {
                Some(texture) => canvas.image_cover(texture, avatar_rect, Color32::WHITE),
                None => draw_avatar(canvas, avatar_rect),
            },
        }

        if !account.signed_in {
            canvas.text_with_shadow(
                Pos2::new(avatar_rect.min.x + 18.0, avatar_rect.max.y - 24.0),
                Align2::LEFT_CENTER,
                "GUEST",
                20.0,
                theme::TEXT_MUTED,
                false,
            );
        }
        if avatar.focused {
            canvas.focus_ring(avatar_rect, 0);
        } else {
            canvas.rect_stroke(
                avatar_rect,
                0,
                Stroke::new(1.5, Color32::from_rgba_unmultiplied(170, 228, 247, 120)),
            );
        }
        if avatar.activated {
            if account.signed_in {
                self.go_tab(NavTab::Avatar);
            } else {
                self.goto(Screen::SignIn);
            }
            return;
        }

        screens::section_label(canvas, 100.0, FRIEND_LABEL_Y, "Friend Activity");
        let friends_rect =
            Rect::from_min_size(Pos2::new(AVATAR_X, FRIEND_TOP), Vec2::new(AVATAR_W, 300.0));
        if let Some((name, place_id)) = draw_friend_activity(
            canvas,
            friends_rect,
            &account,
            &friend_textures,
            &mut self.home_friends_scroll,
            &mut self.scroll_drag,
        ) {
            self.open_launch(format!("{name}'s game"), place_id, Screen::Home);
            return;
        }
        if !account.signed_in {
            let sign_in_rect = Rect::from_center_size(
                Pos2::new(AVATAR_X + AVATAR_W * 0.5, 800.0),
                Vec2::new(220.0, 40.0),
            );
            let response = canvas.focusable("home:friends-signin", sign_in_rect, Sense::click());
            canvas.text_centered(
                sign_in_rect,
                "SIGN IN",
                22.0,
                if response.focused {
                    theme::FOCUS
                } else {
                    theme::TEXT_SECONDARY
                },
                false,
            );
            if response.focused {
                canvas.line(
                    Pos2::new(
                        sign_in_rect.center().x - 40.0,
                        sign_in_rect.center().y + 16.0,
                    ),
                    Pos2::new(
                        sign_in_rect.center().x + 40.0,
                        sign_in_rect.center().y + 16.0,
                    ),
                    Stroke::new(1.5, theme::FOCUS),
                );
            }
            if response.activated {
                self.goto(Screen::SignIn);
                return;
            }
        }

        screens::section_label(canvas, GRID_X, 252.0, "Featured");
        self.show_featured(canvas);
    }

    fn account_view(&self) -> AccountView {
        let Some(session) = &self.session else {
            return AccountView {
                signed_in: false,
                display_name: String::new(),
                headshot_url: None,
                avatar_url: None,
                friends: Vec::new(),
                friends_loading: false,
                friends_error: None,
            };
        };

        let (friends, friends_loading, friends_error) = match &session.friends {
            Load::Ready(friends) => {
                let rows: Vec<FriendView> = friends
                    .iter()
                    .map(|friend| {
                        let presence = session
                            .presence
                            .get(&friend.id)
                            .cloned()
                            .unwrap_or_default();
                        FriendView {
                            user_id: friend.id,
                            name: if friend.display().is_empty() {
                                format!("User {}", friend.id)
                            } else {
                                friend.display().to_owned()
                            },
                            presence: presence.label().to_owned(),
                            online: presence.is_online(),
                            place_id: presence.place_id,
                            headshot_url: session.headshots.get(&friend.id).cloned(),
                        }
                    })
                    .filter(|friend| friend.online)
                    .collect();
                (rows, false, None)
            }
            Load::Loading | Load::Idle => (Vec::new(), true, None),
            Load::Failed(error) => (Vec::new(), false, Some(error.clone())),
        };

        AccountView {
            signed_in: true,
            display_name: session.user.display().to_owned(),
            headshot_url: session.headshots.get(&session.user.id).cloned(),
            avatar_url: session.avatar.clone(),
            friends,
            friends_loading,
            friends_error,
        }
    }

    fn show_featured(&mut self, canvas: &mut Canvas) {
        let grid = Rect::from_min_size(
            Pos2::new(GRID_X, GRID_Y),
            Vec2::new(GRID_W, ROW0_H + ROW_GAP + ROW1_H),
        );

        let sorts = match &self.catalog.sorts {
            Load::Ready(sorts) => sorts.clone(),
            Load::Loading | Load::Idle => {
                self.draw_home_skeletons(canvas);
                return;
            }
            Load::Failed(message) => {
                if screens::error_panel(canvas, grid, message, None, "home:retry-sorts") {
                    self.retry_sorts();
                }
                return;
            }
        };

        let Some(first_sort) = sorts.first() else {
            screens::panel(
                canvas,
                Rect::from_min_size(Pos2::new(GRID_X, GRID_Y), Vec2::new(GRID_W, 120.0)),
            );
            canvas.text_centered(
                Rect::from_min_size(Pos2::new(GRID_X, GRID_Y), Vec2::new(GRID_W, 120.0)),
                "No featured experiences right now.",
                22.0,
                theme::TEXT_SECONDARY,
                false,
            );
            return;
        };

        let games = match self.catalog.sort_games.get(&first_sort.sort_id) {
            Some(Load::Ready(games)) => games.clone(),
            Some(Load::Loading) | Some(Load::Idle) | None => {
                self.draw_home_skeletons(canvas);
                return;
            }
            Some(Load::Failed(message)) => {
                if screens::error_panel(canvas, grid, message, None, "home:retry-content") {
                    self.retry_sort_content(&first_sort.sort_id);
                }
                return;
            }
        };

        if games.is_empty() {
            screens::panel(
                canvas,
                Rect::from_min_size(Pos2::new(GRID_X, GRID_Y), Vec2::new(GRID_W, 120.0)),
            );
            canvas.text_centered(
                Rect::from_min_size(Pos2::new(GRID_X, GRID_Y), Vec2::new(GRID_W, 120.0)),
                "No featured experiences right now.",
                22.0,
                theme::TEXT_SECONDARY,
                false,
            );
            return;
        }

        self.ensure_thumbnails_for(&games);

        let mut activate = None;
        let offset = self.games.home_offset % games.len();
        for index in 0..8 {
            let game = &games[(offset + index) % games.len()];
            let column = index % 4;
            let row = index / 4;
            let (row_y, row_h) = if row == 0 {
                (GRID_Y, ROW0_H)
            } else {
                (ROW1_Y, ROW1_H)
            };
            let rect = Rect::from_min_size(
                Pos2::new(GRID_X + column as f32 * CARD_STEP, row_y),
                Vec2::new(CARD_W, row_h),
            );
            let id = format!("home:card:{index}");
            let clicked =
                screens::game_card(canvas, &id, rect, game, self.thumbnail(game.universe_id));
            if clicked {
                activate = Some((game.universe_id, game.root_place_id));
            }
        }

        let overflow_rect = Rect::from_min_size(
            Pos2::new(
                GRID_X + GRID_W - 132.0,
                GRID_Y + ROW0_H + ROW_GAP + ROW1_H + 14.0,
            ),
            Vec2::new(132.0, 44.0),
        );
        if screens::ellipsis_button(canvas, "home:more", overflow_rect) {
            if matches!(canvas.sfx.last(), Some(Sfx::Select)) {
                canvas.sfx.pop();
            }
            self.open_browse(
                first_sort.sort_id.clone(),
                "Featured".to_owned(),
                Screen::Home,
            );
            return;
        }

        if let Some((universe_id, root_place_id)) = activate {
            self.open_details(universe_id, root_place_id, Screen::Home);
        }
    }

    fn draw_home_skeletons(&self, canvas: &mut Canvas) {
        for index in 0..8 {
            let column = index % 4;
            let row = index / 4;
            let (row_y, row_h) = if row == 0 {
                (GRID_Y, ROW0_H)
            } else {
                (ROW1_Y, ROW1_H)
            };
            let rect = Rect::from_min_size(
                Pos2::new(GRID_X + column as f32 * CARD_STEP, row_y),
                Vec2::new(CARD_W, row_h),
            );
            screens::skeleton_card(canvas, rect);
        }
        canvas.text(
            Pos2::new(GRID_X + 8.0, GRID_Y + 18.0),
            Align2::LEFT_TOP,
            "Loading featured experiences...",
            16.0,
            theme::TEXT_MUTED,
            false,
        );
    }
}

fn draw_friend_activity(
    canvas: &mut Canvas,
    rect: Rect,
    account: &AccountView,
    textures: &[Option<TextureHandle>],
    scroll: &mut f32,
    scroll_drag: &mut Option<screens::ScrollDrag>,
) -> Option<(String, u64)> {
    if !account.signed_in {
        canvas.text_centered_with_shadow(
            Rect::from_min_size(
                Pos2::new(rect.min.x, rect.min.y + 36.0),
                Vec2::new(rect.width(), 48.0),
            ),
            "Sign in to see friend activity",
            26.0,
            theme::TEXT_MUTED,
            false,
        );
        return None;
    }
    if account.friends_loading {
        canvas.text(
            Pos2::new(rect.min.x, rect.min.y + 48.0),
            Align2::LEFT_CENTER,
            "Loading friend activity...",
            24.0,
            theme::TEXT_MUTED,
            false,
        );
        return None;
    }
    if let Some(error) = &account.friends_error {
        canvas.text(
            Pos2::new(rect.min.x, rect.min.y + 48.0),
            Align2::LEFT_CENTER,
            error,
            18.0,
            theme::NEGATIVE,
            false,
        );
        return None;
    }
    if account.friends.is_empty() {
        canvas.text_centered_with_shadow(
            Rect::from_min_size(
                Pos2::new(rect.min.x, rect.min.y + 36.0),
                Vec2::new(rect.width(), 48.0),
            ),
            "Your friends are not online",
            26.0,
            theme::TEXT_MUTED,
            false,
        );
        return None;
    }

    let content_height = account.friends.len() as f32 * FRIEND_ROW_STEP;
    let max_scroll = (content_height - rect.height()).max(0.0);
    let wheel = canvas.ui.input(|input| input.smooth_scroll_delta.y);
    if wheel != 0.0 && canvas.ui.rect_contains_pointer(canvas.screen_rect(rect)) {
        *scroll = (*scroll - wheel).clamp(0.0, max_scroll);
    }
    *scroll = (*scroll).clamp(0.0, max_scroll);

    let clip = canvas.screen_rect(rect);
    let clipped = canvas.painter.with_clip_rect(clip);
    let original_painter = std::mem::replace(&mut canvas.painter, clipped);
    for (index, friend) in account.friends.iter().enumerate() {
        let y = rect.min.y + index as f32 * FRIEND_ROW_STEP - *scroll;
        if y + 108.0 < rect.min.y || y > rect.max.y {
            continue;
        }
        let head = Rect::from_min_size(Pos2::new(rect.min.x, y), Vec2::splat(72.0));
        match textures.get(index).and_then(|texture| texture.as_ref()) {
            Some(texture) => canvas.image_cover(texture, head, Color32::WHITE),
            None => draw_friend_icon(canvas, head),
        }
        let name = util::truncate_to_width(&friend.name, rect.width() - 240.0, |text| {
            canvas.text_width(text, 34.0, false)
        });
        canvas.text_with_shadow(
            Pos2::new(rect.min.x + 96.0, y + 22.0),
            Align2::LEFT_CENTER,
            name,
            34.0,
            theme::TEXT_PRIMARY,
            false,
        );
        let dot = Pos2::new(rect.min.x + 100.0, y + 54.0);
        canvas.circle_filled(
            dot,
            4.0,
            if friend.online {
                theme::SUCCESS
            } else {
                theme::TEXT_MUTED
            },
        );
        let presence = util::truncate_to_width(&friend.presence, rect.width() - 250.0, |text| {
            canvas.text_width(text, 24.0, false)
        });
        canvas.text_with_shadow(
            Pos2::new(dot.x + 12.0, dot.y),
            Align2::LEFT_CENTER,
            presence,
            24.0,
            theme::TEXT_MUTED,
            false,
        );
        if friend.online
            && friend.place_id > 0
            && screens::button(
                canvas,
                &format!("home:friend-join:{}", friend.user_id),
                Rect::from_min_size(
                    Pos2::new(rect.max.x - 112.0, y + 24.0),
                    Vec2::new(112.0, 44.0),
                ),
                "Join",
                ButtonKind::Play,
                true,
            )
        {
            canvas.painter = original_painter;
            return Some((friend.name.clone(), friend.place_id));
        }
        if index + 1 < account.friends.len() {
            canvas.line(
                Pos2::new(rect.min.x, y + 108.0),
                Pos2::new(rect.max.x, y + 108.0),
                Stroke::new(1.0, Color32::from_white_alpha(24)),
            );
        }
    }
    canvas.painter = original_painter;

    let track = Rect::from_min_size(
        Pos2::new(rect.max.x - 6.0, rect.min.y),
        Vec2::new(5.0, rect.height()),
    );
    screens::drag_scrollbar(
        canvas,
        "home:friend-activity-scrollbar",
        track,
        content_height,
        rect.height(),
        scroll,
        scroll_drag,
    );
    None
}

fn draw_friend_icon(canvas: &Canvas, rect: Rect) {
    draw_profile_silhouette(canvas, rect);
}

fn draw_avatar(canvas: &Canvas, rect: Rect) {
    draw_profile_silhouette(canvas, rect);
}

fn draw_profile_silhouette(canvas: &Canvas, rect: Rect) {
    canvas.rect_filled(rect, 0, Color32::from_rgb(40, 61, 71));
    let size = rect.width().min(rect.height());
    let silhouette = Color32::from_rgb(74, 100, 116);
    let center_x = rect.center().x;
    let head_radius = size * 0.19;
    let head_center = Pos2::new(center_x, rect.min.y + size * 0.37);
    canvas.circle_filled(head_center, head_radius, silhouette);
    let body = Rect::from_min_max(
        Pos2::new(center_x - size * 0.30, head_center.y + head_radius * 0.75),
        Pos2::new(center_x + size * 0.30, rect.max.y),
    );
    canvas.rect_filled(body, (size * 0.16) as u8, silhouette);
}
