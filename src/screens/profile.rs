use egui::{Align2, Color32, Pos2, Rect, Stroke, TextureHandle, Vec2};

use crate::app::{App, open_url};
use crate::canvas::Canvas;
use crate::models::{AvatarAsset, Load};
use crate::screens::{self, ButtonKind, Screen};
use crate::theme;
use crate::util;

impl App {
    pub fn show_profile(&mut self, canvas: &mut Canvas, background: &TextureHandle) {
        let dim = screens::adaptive_dim(canvas, 48);
        screens::chrome(canvas, background, dim);

        let back_label = match self.profile.from {
            Screen::Placeholder => "Friends",
            Screen::Home => "Home",
            Screen::Chat => "Chat",
            _ => "Back",
        };
        if screens::back_button(canvas, "profile:back", back_label) {
            self.goto(self.profile.from);
            return;
        }

        let display = if self.profile.display_name.is_empty() {
            format!("User {}", self.profile.user_id)
        } else {
            self.profile.display_name.clone()
        };
        let username = self.profile.username.clone();
        let headshot_url = self.profile.headshot_url.clone();
        let bundle = self.profile.bundle.ready().cloned();

        if let Some(url) = &headshot_url {
            self.images.ensure(url);
        }
        if let Some(bundle) = &bundle {
            if let Some(url) = &bundle.avatar_url {
                self.images.ensure(url);
            }
            for url in bundle.asset_urls.values() {
                self.images.ensure(url);
            }
        }
        let headshot = headshot_url
            .as_ref()
            .and_then(|url| self.images.get(url))
            .cloned();
        let avatar = bundle
            .as_ref()
            .and_then(|bundle| bundle.avatar_url.as_ref())
            .and_then(|url| self.images.get(url))
            .cloned();

        let avatar_panel = Rect::from_min_size(Pos2::new(104.0, 170.0), Vec2::new(520.0, 660.0));
        canvas.gradient_v(
            avatar_panel,
            Color32::from_rgb(47, 103, 126),
            Color32::from_rgb(23, 56, 67),
        );
        match &avatar {
            Some(texture) => canvas.image_cover(texture, avatar_panel, Color32::WHITE),
            None => match &headshot {
                Some(texture) => {
                    canvas.image_contain(texture, avatar_panel.shrink(48.0), Color32::WHITE)
                }
                None => draw_profile_silhouette(canvas, avatar_panel.shrink(80.0)),
            },
        }
        canvas.rect_stroke(
            avatar_panel,
            0,
            Stroke::new(1.5, Color32::from_rgba_unmultiplied(170, 228, 247, 120)),
        );

        let head = Rect::from_min_size(Pos2::new(680.0, 180.0), Vec2::splat(108.0));
        match &headshot {
            Some(texture) => canvas.image_cover(texture, head, Color32::WHITE),
            None => draw_profile_silhouette(canvas, head),
        }
        canvas.rect_stroke(head, 0, Stroke::new(1.0, Color32::from_white_alpha(40)));

        canvas.text_with_shadow(
            Pos2::new(812.0, 202.0),
            Align2::LEFT_CENTER,
            util::truncate_to_width(&display, 620.0, |text| canvas.text_width(text, 42.0, true)),
            42.0,
            theme::TEXT_PRIMARY,
            true,
        );
        if !username.is_empty() {
            canvas.text_with_shadow(
                Pos2::new(812.0, 246.0),
                Align2::LEFT_CENTER,
                format!("@{username}"),
                22.0,
                theme::TEXT_SECONDARY,
                false,
            );
        }
        let presence = if self.profile.presence.is_empty() {
            "Offline".to_owned()
        } else {
            self.profile.presence.clone()
        };
        let online = self.profile.place_id > 0;
        canvas.circle_filled(
            Pos2::new(816.0, 286.0),
            4.0,
            if online {
                theme::SUCCESS
            } else {
                theme::TEXT_MUTED
            },
        );
        canvas.text_with_shadow(
            Pos2::new(828.0, 286.0),
            Align2::LEFT_CENTER,
            util::truncate_to_width(&presence, 560.0, |text| {
                canvas.text_width(text, 18.0, false)
            }),
            18.0,
            theme::TEXT_MUTED,
            false,
        );

        let counts = bundle
            .as_ref()
            .map(|bundle| (bundle.friends, bundle.followers, bundle.following));
        let count_y = 340.0;
        if let Some((friends, followers, following)) = counts {
            for (index, (value, label)) in [
                (friends, "Friends"),
                (followers, "Followers"),
                (following, "Following"),
            ]
            .into_iter()
            .enumerate()
            {
                let rect = Rect::from_min_size(
                    Pos2::new(680.0 + index as f32 * 202.0, count_y),
                    Vec2::new(188.0, 44.0),
                );
                canvas.rect_filled(rect, 0, Color32::from_black_alpha(110));
                canvas.rect_stroke(rect, 0, Stroke::new(1.0, Color32::from_white_alpha(25)));
                canvas.text_centered(
                    rect,
                    format!("{} {label}", util::format_count(value)),
                    18.0,
                    theme::TEXT_SECONDARY,
                    false,
                );
            }
        }

        let chat_rect = Rect::from_min_size(Pos2::new(680.0, 410.0), Vec2::new(188.0, 56.0));
        let add_rect = Rect::from_min_size(Pos2::new(884.0, 410.0), Vec2::new(214.0, 56.0));
        let join_rect = Rect::from_min_size(Pos2::new(1114.0, 410.0), Vec2::new(150.0, 56.0));

        if screens::button(
            canvas,
            "profile:chat",
            chat_rect,
            "Chat",
            ButtonKind::Primary,
            true,
        ) {
            self.open_chat_with(
                self.profile.user_id,
                display.clone(),
                username.clone(),
                headshot_url.clone(),
                self.profile.from,
            );
            return;
        }
        if screens::button(
            canvas,
            "profile:add",
            add_rect,
            "Add Friend",
            ButtonKind::Secondary,
            true,
        ) {
            self.open_add_friend(Some((self.profile.user_id, display.clone())));
            return;
        }
        if self.profile.place_id > 0
            && screens::button(
                canvas,
                "profile:join",
                join_rect,
                "Join",
                ButtonKind::Play,
                true,
            )
        {
            self.open_launch(
                format!("{display}'s game"),
                self.profile.place_id,
                Screen::Profile,
            );
            return;
        }

        screens::section_label(canvas, 680.0, 500.0, "Currently Wearing");
        let mut item_index = 0;
        if let Some(bundle) = &bundle {
            for asset in bundle.assets.iter().filter(|asset| is_wearable(asset)) {
                if item_index >= 8 {
                    break;
                }
                let column = item_index % 4;
                let row = item_index / 4;
                let rect = Rect::from_min_size(
                    Pos2::new(680.0 + column as f32 * 214.0, 540.0 + row as f32 * 178.0),
                    Vec2::new(196.0, 120.0),
                );
                let texture = bundle
                    .asset_urls
                    .get(&asset.id)
                    .and_then(|url| self.images.get(url))
                    .cloned();
                match &texture {
                    Some(texture) => {
                        canvas.rect_filled(rect, 0, theme::panel(210));
                        canvas.image_contain(texture, rect.shrink(6.0), Color32::WHITE);
                    }
                    None => {
                        canvas.rect_filled(rect, 0, theme::PANEL_SOLID);
                        canvas.text_centered(
                            rect,
                            util::initials(&asset.name),
                            28.0,
                            theme::TEXT_MUTED,
                            true,
                        );
                    }
                }
                canvas.rect_stroke(rect, 0, Stroke::new(1.0, Color32::from_white_alpha(25)));
                canvas.text_with_shadow(
                    Pos2::new(rect.min.x, rect.max.y + 12.0),
                    Align2::LEFT_TOP,
                    util::truncate_to_width(&asset.name, rect.width(), |text| {
                        canvas.text_width(text, 16.0, false)
                    }),
                    16.0,
                    theme::TEXT_SECONDARY,
                    false,
                );
                item_index += 1;
            }
        } else if matches!(self.profile.bundle, Load::Loading) {
            canvas.text(
                Pos2::new(680.0, 550.0),
                Align2::LEFT_TOP,
                "Loading profile...",
                20.0,
                theme::TEXT_MUTED,
                false,
            );
        } else if let Load::Failed(error) = &self.profile.bundle {
            canvas.text(
                Pos2::new(680.0, 550.0),
                Align2::LEFT_TOP,
                error,
                18.0,
                theme::NEGATIVE,
                false,
            );
            if screens::button(
                canvas,
                "profile:retry",
                Rect::from_min_size(Pos2::new(680.0, 600.0), Vec2::new(160.0, 48.0)),
                "Retry",
                ButtonKind::Secondary,
                true,
            ) {
                self.profile.requested = false;
                self.profile.bundle = Load::Idle;
            }
        }

        let view = Rect::from_min_size(Pos2::new(680.0, 960.0), Vec2::new(300.0, 48.0));
        if screens::button(
            canvas,
            "profile:view",
            view,
            "View full profile",
            ButtonKind::Ghost,
            true,
        ) {
            let url = format!(
                "https://www.roblox.com/users/{}/profile",
                self.profile.user_id
            );
            if let Err(error) = open_url(&url) {
                self.toast(error);
            }
        }
    }
}

fn is_wearable(asset: &AvatarAsset) -> bool {
    matches!(
        asset.asset_type.id,
        2 | 8 | 11 | 12 | 18 | 41 | 42 | 43 | 44 | 45 | 46 | 47
    )
}

fn draw_profile_silhouette(canvas: &Canvas, rect: Rect) {
    canvas.rect_filled(rect, 0, theme::PANEL_SOLID);
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
