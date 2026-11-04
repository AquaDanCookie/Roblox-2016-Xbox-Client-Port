use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, TextureHandle, Vec2};

use crate::app::{App, AvatarCategory, SkinPart};
use crate::canvas::Canvas;
use crate::models::{AvatarBundle, BodyColors, Friend, Load, Presence};
use crate::screens::{self, ButtonKind, NavTab, Screen};
use crate::theme;
use crate::util;
use secrecy::ExposeSecret;

const SKIN_TONES: &[(u64, &str, u32)] = &[
    (361, "Dirt brown", 0x564236),
    (192, "Reddish brown", 0x694028),
    (217, "Brown", 0x7c5c46),
    (153, "Sand red", 0x957977),
    (359, "Linen", 0xaf9483),
    (352, "Burlap", 0xc7ac78),
    (5, "Brick yellow", 0xd7c59a),
    (101, "Medium red", 0xda867a),
    (1007, "Dusty Rose", 0xa34b4b),
    (1014, "CGA brown", 0xaa5500),
    (38, "Dark orange", 0xa05f35),
    (18, "Nougat", 0xcc8e69),
    (125, "Light orange", 0xeab892),
    (1030, "Pastel brown", 0xffcc99),
    (133, "Neon orange", 0xd5733d),
    (106, "Bright orange", 0xda8541),
    (105, "Br. yellowish orange", 0xe29b40),
    (1017, "Deep orange", 0xffaf00),
    (24, "Bright yellow", 0xf5cd30),
    (334, "Daisy orange", 0xf8d96d),
    (226, "Cool yellow", 0xfdea8d),
    (141, "Earth green", 0x27462d),
    (1021, "Camo", 0x3a7d15),
    (28, "Dark green", 0x287f47),
    (37, "Bright green", 0x4b974b),
    (310, "Shamrock", 0x5b9a4c),
    (317, "Moss", 0x7c9c6b),
    (119, "Br. yellowish green", 0xa4bd47),
    (1011, "Navy blue", 0x002060),
    (1012, "Deep blue", 0x2154b9),
    (1010, "Really blue", 0x0000ff),
    (23, "Bright blue", 0x0d69ac),
    (305, "Steel blue", 0x527cae),
    (102, "Medium blue", 0x6e99ca),
    (45, "Light blue", 0xb4d2e4),
    (107, "Bright bluish green", 0x008f9c),
    (1018, "Teal", 0x12eed4),
    (1027, "Pastel blue-green", 0x9ff3e9),
    (1019, "Toothpaste", 0x00ffff),
    (1013, "Cyan", 0x04afec),
    (11, "Pastel Blue", 0x80bbdc),
    (1024, "Pastel light blue", 0xafddff),
    (104, "Bright violet", 0x6b327c),
    (1023, "Lavender", 0x8c5b9f),
    (321, "Lilac", 0xa75e9b),
    (1015, "Magenta", 0xaa00aa),
    (1031, "Royal purple", 0x6225d1),
    (1006, "Alder", 0xb480ff),
    (1026, "Pastel violet", 0xb1a7ff),
    (21, "Bright red", 0xc4281c),
    (1004, "Really red", 0xff0000),
    (1032, "Hot pink", 0xff00bf),
    (1016, "Pink", 0xff66cc),
    (330, "Carnation pink", 0xff98dc),
    (9, "Light reddish violet", 0xe8bac8),
    (1025, "Pastel orange", 0xffc9c9),
    (364, "Dark taupe", 0x5a4c42),
    (351, "Cork", 0xbc9b5d),
    (1008, "Olive", 0xc1be42),
    (29, "Medium green", 0xa1c48c),
    (1022, "Grime", 0x7f8e64),
    (151, "Sand green", 0x789082),
    (135, "Sand blue", 0x74869d),
    (1020, "Lime green", 0x00ff00),
    (1028, "Pastel green", 0xccffcc),
    (1009, "New Yeller", 0xffff00),
    (1029, "Pastel yellow", 0xffffcc),
    (1003, "Really black", 0x111111),
    (26, "Black", 0x1b2a35),
    (199, "Dark stone grey", 0x635f62),
    (194, "Medium stone grey", 0xa3a2a5),
    (1002, "Mid gray", 0xcdcdcd),
    (208, "Light stone grey", 0xe5e4df),
    (1, "White", 0xf2f3f3),
    (1001, "Institutional white", 0xf8f8f8),
];

struct FriendRow {
    user_id: u64,
    name: String,
    username: String,
    presence: String,
    online: bool,
    place_id: u64,
    headshot_url: Option<String>,
}

impl App {
    pub fn show_placeholder(
        &mut self,
        canvas: &mut Canvas,
        background: &TextureHandle,
        wordmark: Option<&TextureHandle>,
    ) {
        let dim = screens::adaptive_dim(canvas, 40);
        screens::chrome(canvas, background, dim);

        if let Some(tab) = screens::nav(canvas, wordmark, self.placeholder_tab) {
            self.go_tab(tab);
            return;
        }

        match self.placeholder_tab {
            NavTab::Avatar => self.show_avatar_tab(canvas),
            NavTab::Friends => self.show_friends_tab(canvas),
            NavTab::Robux => self.show_robux_tab(canvas),
            NavTab::Home | NavTab::Games | NavTab::Chat | NavTab::Settings => {
                let panel = Rect::from_min_size(Pos2::new(96.0, 300.0), Vec2::new(1728.0, 200.0));
                screens::panel(canvas, panel);
                canvas.text_centered(
                    panel,
                    "Choose a destination from the top navigation.",
                    24.0,
                    theme::TEXT_SECONDARY,
                    false,
                );
            }
        }
    }

    fn show_avatar_tab(&mut self, canvas: &mut Canvas) {
        let data = self.session.as_ref().map(|session| {
            (
                session.user.display().to_owned(),
                session.user.name.clone(),
                session.user.id,
                session.headshots.get(&session.user.id).cloned(),
                session.avatar.clone(),
            )
        });

        let Some((display, username, _user_id, headshot_url, avatar_url)) = data else {
            self.signed_out_tab(canvas, "Avatar", "Sign in to see your avatar.");
            return;
        };

        if let Some(url) = &headshot_url {
            self.images.ensure(url);
        }
        if let Some(url) = &avatar_url {
            self.images.ensure(url);
        }
        let headshot_texture = headshot_url
            .as_ref()
            .and_then(|url| self.images.get(url))
            .cloned();
        let avatar_texture = avatar_url
            .as_ref()
            .and_then(|url| self.images.get(url))
            .cloned();

        let body = Rect::from_min_size(Pos2::new(96.0, 278.0), Vec2::new(430.0, 640.0));
        for (factor, alpha) in [(0.9, 10), (0.65, 14), (0.42, 18)] {
            canvas.circle_filled(
                Pos2::new(body.center().x, body.center().y - 20.0),
                body.width() * factor,
                theme::FOCUS.gamma_multiply(alpha as f32 / 255.0),
            );
        }
        if let Some(texture) = &avatar_texture {
            canvas.image_contain(texture, body, Color32::WHITE);
        } else if let Some(texture) = &headshot_texture {
            canvas.image_cover(texture, body, Color32::WHITE);
        } else {
            draw_avatar_box(canvas, body);
        }
        canvas.text_with_shadow(
            Pos2::new(96.0, 972.0),
            Align2::LEFT_CENTER,
            display,
            30.0,
            theme::TEXT_PRIMARY,
            true,
        );
        canvas.text_with_shadow(
            Pos2::new(96.0, 1004.0),
            Align2::LEFT_CENTER,
            format!("@{username}"),
            18.0,
            theme::TEXT_MUTED,
            false,
        );

        screens::section_label(canvas, 765.0, 252.0, "My Items");
        if self.draw_avatar_category_nav(canvas) {
            return;
        }
        match self.avatar_edit.category {
            AvatarCategory::All => {
                self.show_all_items(canvas);
                return;
            }
            AvatarCategory::Avatars => {
                self.show_platform_outfits(canvas);
                return;
            }
            _ => {}
        }
        if self.draw_avatar_subcategory_nav(canvas) {
            return;
        }
        if self.avatar_edit.body_part_menu {
            self.show_body_part_menu(canvas);
            return;
        }
        match self.avatar_edit.selected_type {
            1000 => {
                self.show_skin_tones(canvas);
                return;
            }
            1001 => {
                self.show_build(canvas);
                return;
            }
            2000 => {
                self.show_body_parts(canvas);
                return;
            }
            2001 => {
                self.show_animation_bundles(canvas);
                return;
            }
            _ => {}
        }

        let grid_top = if self.avatar_edit.selected_type == 2000 {
            446.0
        } else {
            392.0
        };
        let inventory = self
            .avatar_edit
            .inventory
            .get(&self.avatar_edit.selected_type)
            .cloned();
        match inventory {
            Some(Load::Ready(bundle)) => {
                for url in bundle.urls.values() {
                    self.images.ensure(url);
                }
                for (index, item) in bundle.items.iter().take(15).enumerate() {
                    let column = index % 5;
                    let row = index / 5;
                    let tile = Rect::from_min_size(
                        Pos2::new(765.0 + column as f32 * 182.0, grid_top + row as f32 * 182.0),
                        Vec2::new(150.0, 150.0),
                    );
                    let texture = bundle
                        .urls
                        .get(&item.asset_id)
                        .and_then(|url| self.images.get(url))
                        .cloned();
                    let worn = if item.asset_type == 61 {
                        self.avatar_edit
                            .emotes
                            .iter()
                            .any(|emote| emote.asset_id == item.asset_id)
                    } else {
                        self.avatar_edit.worn.iter().any(|asset| {
                            asset.id == item.asset_id && asset.asset_type.id == item.asset_type
                        })
                    };
                    let response = canvas.focusable(
                        format!("avatar:item:{}", item.asset_id),
                        tile,
                        Sense::click(),
                    );
                    match &texture {
                        Some(texture) => canvas.image_cover(texture, tile, Color32::WHITE),
                        None => {
                            canvas.rect_filled(tile, 0, Color32::from_rgb(37, 70, 82));
                            canvas.text_centered(
                                tile,
                                util::initials(&item.name),
                                28.0,
                                theme::TEXT_MUTED,
                                true,
                            );
                        }
                    }
                    canvas.rect_stroke(
                        tile,
                        0,
                        if worn {
                            Stroke::new(3.0, theme::SUCCESS)
                        } else {
                            Stroke::new(1.0, Color32::from_white_alpha(22))
                        },
                    );
                    if response.focused {
                        canvas.focus_ring(tile, 0);
                    }
                    let item_label = if item.name.is_empty() {
                        format!("Item {}", item.asset_id)
                    } else {
                        item.name.clone()
                    };
                    canvas.text(
                        Pos2::new(tile.min.x, tile.max.y + 12.0),
                        Align2::LEFT_TOP,
                        util::truncate_to_width(&item_label, tile.width(), |text| {
                            canvas.text_width(text, 15.0, false)
                        }),
                        15.0,
                        if worn {
                            theme::SUCCESS
                        } else {
                            theme::TEXT_SECONDARY
                        },
                        false,
                    );
                    if response.activated {
                        self.toggle_avatar_asset(item);
                    }
                }
                if bundle.items.is_empty() {
                    canvas.text(
                        Pos2::new(765.0, 402.0),
                        Align2::LEFT_TOP,
                        "No items of this type in your inventory.",
                        18.0,
                        theme::TEXT_MUTED,
                        false,
                    );
                }
            }
            Some(Load::Loading) => {
                canvas.text(
                    Pos2::new(765.0, 402.0),
                    Align2::LEFT_TOP,
                    "Loading inventory...",
                    18.0,
                    theme::TEXT_MUTED,
                    false,
                );
            }
            Some(Load::Failed(error)) => {
                canvas.text(
                    Pos2::new(765.0, 402.0),
                    Align2::LEFT_TOP,
                    error,
                    17.0,
                    theme::NEGATIVE,
                    false,
                );
                if screens::button(
                    canvas,
                    "avatar:retry-inventory",
                    Rect::from_min_size(Pos2::new(765.0, 450.0), Vec2::new(180.0, 44.0)),
                    "Retry",
                    ButtonKind::Secondary,
                    true,
                ) {
                    self.avatar_edit
                        .inventory
                        .remove(&self.avatar_edit.selected_type);
                }
            }
            Some(Load::Idle) | None => {
                canvas.text(
                    Pos2::new(765.0, 402.0),
                    Align2::LEFT_TOP,
                    "Loading inventory...",
                    18.0,
                    theme::TEXT_MUTED,
                    false,
                );
            }
        }

        self.draw_avatar_status(canvas, "Changes apply automatically.");
    }

    fn show_all_items(&mut self, canvas: &mut Canvas) {
        for bundle in self.avatar_edit.inventory.values() {
            if let Load::Ready(bundle) = bundle {
                for url in bundle.urls.values() {
                    self.images.ensure(url);
                }
            }
        }
        let mut items = Vec::new();
        let mut any_loading = false;
        for asset_type in crate::app::AVATAR_INVENTORY_TYPES {
            match self.avatar_edit.inventory.get(asset_type) {
                Some(Load::Ready(bundle)) => {
                    for item in &bundle.items {
                        items.push((
                            item.clone(),
                            bundle
                                .urls
                                .get(&item.asset_id)
                                .and_then(|url| self.images.get(url))
                                .cloned(),
                        ));
                    }
                }
                Some(Load::Loading) | Some(Load::Idle) | None => any_loading = true,
                Some(Load::Failed(_)) => {}
            }
        }
        items.sort_by(|a, b| {
            a.0.asset_type
                .cmp(&b.0.asset_type)
                .then_with(|| a.0.name.to_lowercase().cmp(&b.0.name.to_lowercase()))
        });
        items.dedup_by(|a, b| a.0.asset_id == b.0.asset_id && a.0.asset_type == b.0.asset_type);
        if items.is_empty() {
            canvas.text(
                Pos2::new(765.0, 402.0),
                Align2::LEFT_TOP,
                if any_loading {
                    "Loading your inventory..."
                } else {
                    "No avatar items found in your inventory."
                },
                18.0,
                theme::TEXT_MUTED,
                false,
            );
            return;
        }
        let viewport = Rect::from_min_size(Pos2::new(765.0, 392.0), Vec2::new(940.0, 538.0));
        let columns = 6;
        let tile = 132.0;
        let step = 154.0;
        let row_step = 166.0;
        let content_height = (items.len().div_ceil(columns)) as f32 * row_step;
        let max_scroll = (content_height - viewport.height()).max(0.0);
        let wheel = canvas.ui.input(|input| input.smooth_scroll_delta.y);
        if wheel != 0.0
            && canvas
                .ui
                .rect_contains_pointer(canvas.screen_rect(viewport))
        {
            self.avatar_edit.scroll = (self.avatar_edit.scroll - wheel).clamp(0.0, max_scroll);
        }
        self.avatar_edit.scroll = self.avatar_edit.scroll.clamp(0.0, max_scroll);

        let clip = canvas.screen_rect(viewport);
        let clipped = canvas.painter.with_clip_rect(clip);
        let original = std::mem::replace(&mut canvas.painter, clipped);
        let mut activate = None;
        for (index, (item, texture)) in items.iter().enumerate() {
            let column = index % columns;
            let row = index / columns;
            let y = viewport.min.y + row as f32 * row_step - self.avatar_edit.scroll;
            if y + tile < viewport.min.y || y > viewport.max.y {
                continue;
            }
            let rect = Rect::from_min_size(
                Pos2::new(viewport.min.x + column as f32 * step, y),
                Vec2::splat(tile),
            );
            let response = canvas.focusable(
                format!("avatar:all:item:{}:{}", item.asset_type, item.asset_id),
                rect,
                Sense::click(),
            );
            let worn = if item.asset_type == 61 {
                self.avatar_edit
                    .emotes
                    .iter()
                    .any(|emote| emote.asset_id == item.asset_id)
            } else {
                self.avatar_edit.worn.iter().any(|asset| {
                    asset.id == item.asset_id && asset.asset_type.id == item.asset_type
                })
            };
            match texture {
                Some(texture) => canvas.image_cover(texture, rect, Color32::WHITE),
                None => {
                    canvas.rect_filled(rect, 0, Color32::from_rgb(37, 70, 82));
                    canvas.text_centered(
                        rect,
                        util::initials(&item.name),
                        24.0,
                        theme::TEXT_MUTED,
                        true,
                    );
                }
            }
            canvas.rect_stroke(
                rect,
                0,
                if worn {
                    Stroke::new(3.0, theme::SUCCESS)
                } else {
                    Stroke::new(1.0, Color32::from_white_alpha(22))
                },
            );
            if response.focused {
                canvas.focus_ring(rect, 0);
            }
            let label = if item.name.is_empty() {
                format!("Item {}", item.asset_id)
            } else {
                item.name.clone()
            };
            canvas.text(
                Pos2::new(rect.min.x, rect.max.y + 7.0),
                Align2::LEFT_TOP,
                util::truncate_to_width(&label, rect.width(), |text| {
                    canvas.text_width(text, 13.5, false)
                }),
                13.5,
                if worn {
                    theme::SUCCESS
                } else {
                    theme::TEXT_SECONDARY
                },
                false,
            );
            if response.activated {
                activate = Some(item.clone());
            }
        }
        canvas.painter = original;
        screens::drag_scrollbar(
            canvas,
            "avatar:all-scrollbar",
            Rect::from_min_size(
                Pos2::new(viewport.max.x + 12.0, viewport.min.y),
                Vec2::new(5.0, viewport.height()),
            ),
            content_height,
            viewport.height(),
            &mut self.avatar_edit.scroll,
            &mut self.scroll_drag,
        );
        if let Some(item) = activate {
            self.toggle_avatar_asset(&item);
        }
        self.draw_avatar_status(canvas, "Click an item to wear or remove it.");
    }

    fn show_platform_outfits(&mut self, canvas: &mut Canvas) {
        let state = self.avatar_edit.platform_outfits.clone();
        let (outfits, urls) = match state {
            Load::Ready(bundle) => (bundle.outfits, bundle.urls),
            Load::Loading | Load::Idle => {
                canvas.text(
                    Pos2::new(765.0, 402.0),
                    Align2::LEFT_TOP,
                    "Loading your Roblox outfits...",
                    18.0,
                    theme::TEXT_MUTED,
                    false,
                );
                return;
            }
            Load::Failed(error) => {
                canvas.text(
                    Pos2::new(765.0, 402.0),
                    Align2::LEFT_TOP,
                    error,
                    17.0,
                    theme::NEGATIVE,
                    false,
                );
                if screens::button(
                    canvas,
                    "avatar:retry-outfits",
                    Rect::from_min_size(Pos2::new(765.0, 450.0), Vec2::new(180.0, 44.0)),
                    "Retry",
                    ButtonKind::Secondary,
                    true,
                ) {
                    self.avatar_edit.outfits_requested = false;
                    self.avatar_edit.platform_outfits = Load::Idle;
                }
                return;
            }
        };
        if outfits.is_empty() {
            canvas.text(
                Pos2::new(765.0, 402.0),
                Align2::LEFT_TOP,
                "No saved Roblox outfits yet.",
                18.0,
                theme::TEXT_MUTED,
                false,
            );
            return;
        }
        for url in urls.values() {
            self.images.ensure(url);
        }
        let columns = 8;
        let tile = 108.0;
        let step = 116.0;
        let row_step = 158.0;
        let viewport = Rect::from_min_size(Pos2::new(765.0, 392.0), Vec2::new(940.0, 538.0));
        let created: Vec<&crate::models::PlatformOutfit> =
            outfits.iter().filter(|outfit| outfit.is_editable).collect();
        let catalog: Vec<&crate::models::PlatformOutfit> = outfits
            .iter()
            .filter(|outfit| !outfit.is_editable)
            .collect();
        let mut headings: Vec<(&str, f32)> = Vec::new();
        let mut cards: Vec<(&crate::models::PlatformOutfit, f32, f32)> = Vec::new();
        let mut y = 392.0;
        for (label, group) in [
            ("Created Avatars", created),
            ("Owned Catalog Avatars", catalog),
        ] {
            if group.is_empty() {
                continue;
            }
            headings.push((label, y));
            let grid_y = y + 30.0;
            let rows = group.len().div_ceil(columns);
            for (index, outfit) in group.iter().enumerate() {
                let column = index % columns;
                let row = index / columns;
                cards.push((
                    *outfit,
                    765.0 + column as f32 * step,
                    grid_y + row as f32 * row_step,
                ));
            }
            y = grid_y + rows as f32 * row_step + 18.0;
        }
        let content_height = (y - viewport.min.y).max(1.0);
        let max_scroll = (content_height - viewport.height()).max(0.0);
        let wheel = canvas.ui.input(|input| input.smooth_scroll_delta.y);
        if wheel != 0.0
            && canvas
                .ui
                .rect_contains_pointer(canvas.screen_rect(viewport))
        {
            self.avatar_edit.scroll = (self.avatar_edit.scroll - wheel).clamp(0.0, max_scroll);
        }
        self.avatar_edit.scroll = self.avatar_edit.scroll.clamp(0.0, max_scroll);
        let clip = canvas.screen_rect(viewport);
        let clipped = canvas.painter.with_clip_rect(clip);
        let original = std::mem::replace(&mut canvas.painter, clipped);
        for (label, heading_y) in &headings {
            let heading_screen_y = *heading_y - self.avatar_edit.scroll;
            if heading_screen_y + 24.0 >= viewport.min.y && heading_screen_y <= viewport.max.y {
                screens::section_label(canvas, viewport.min.x, heading_screen_y, label);
            }
        }
        let mut equip = None;
        for (outfit, x, base_y) in &cards {
            let y = *base_y - self.avatar_edit.scroll;
            if y + tile < viewport.min.y || y > viewport.max.y {
                continue;
            }
            let rect = Rect::from_min_size(Pos2::new(*x, y), Vec2::splat(tile));
            let response =
                canvas.focusable(format!("avatar:outfit:{}", outfit.id), rect, Sense::click());
            if let Some(url) = urls.get(&outfit.id)
                && let Some(texture) = self.images.get(url)
            {
                canvas.image_cover(texture, rect, Color32::WHITE);
            } else {
                canvas.rect_filled(rect, 0, theme::panel(190));
                canvas.text_centered(
                    rect,
                    util::initials(&outfit.name),
                    24.0,
                    theme::TEXT_MUTED,
                    true,
                );
            }
            canvas.rect_stroke(
                rect,
                0,
                if response.focused {
                    Stroke::new(3.0, theme::FOCUS)
                } else {
                    Stroke::new(1.0, Color32::from_white_alpha(24))
                },
            );
            canvas.text(
                Pos2::new(rect.min.x, rect.max.y + 8.0),
                Align2::LEFT_TOP,
                util::truncate_to_width(&outfit.name, tile, |text| {
                    canvas.text_width(text, 13.0, false)
                }),
                13.0,
                if response.focused {
                    theme::FOCUS
                } else {
                    theme::TEXT_SECONDARY
                },
                false,
            );
            if response.activated {
                equip = Some(outfit.id);
            }
        }
        canvas.painter = original;
        screens::drag_scrollbar(
            canvas,
            "avatar:outfits-scrollbar",
            Rect::from_min_size(
                Pos2::new(viewport.max.x + 12.0, viewport.min.y),
                Vec2::new(5.0, viewport.height()),
            ),
            content_height,
            viewport.height(),
            &mut self.avatar_edit.scroll,
            &mut self.scroll_drag,
        );
        if let Some(outfit_id) = equip {
            self.equip_platform_outfit(outfit_id);
        }
        self.draw_avatar_status(canvas, "Click an outfit to wear it.");
    }

    fn equip_platform_outfit(&mut self, outfit_id: u64) {
        if self.avatar_edit.appearance_saving {
            return;
        }
        let Some(token) = self
            .session
            .as_ref()
            .map(|session| session.token.expose_secret().to_owned())
        else {
            self.avatar_edit.status = Some(("Sign in to wear an outfit.".to_owned(), true));
            return;
        };
        self.avatar_edit.appearance_saving = true;
        self.avatar_edit.status = Some(("Equipping outfit...".to_owned(), false));
        self.api.equip_outfit(token, outfit_id, self.api_tx.clone());
    }

    fn body_part_label(&self) -> &'static str {
        match self.avatar_edit.body_part {
            27 => "Torso",
            28 => "Right Arm",
            29 => "Left Arm",
            30 => "Right Leg",
            31 => "Left Leg",
            _ => "Full Body",
        }
    }

    fn show_body_parts(&mut self, canvas: &mut Canvas) {
        canvas.text(
            Pos2::new(765.0, 410.0),
            Align2::LEFT_CENTER,
            "Body Parts",
            18.0,
            theme::TEXT_SECONDARY,
            false,
        );
        let dropdown = Rect::from_min_size(Pos2::new(902.0, 388.0), Vec2::new(250.0, 44.0));
        let response = canvas.focusable("avatar:body-parts-menu", dropdown, Sense::click());
        canvas.rect_filled(
            dropdown,
            0,
            if response.focused {
                theme::ACCENT
            } else {
                theme::SECONDARY_BUTTON
            },
        );
        canvas.text(
            Pos2::new(dropdown.min.x + 18.0, dropdown.center().y),
            Align2::LEFT_CENTER,
            self.body_part_label(),
            18.0,
            theme::TEXT_PRIMARY,
            false,
        );
        let chevron = Pos2::new(dropdown.max.x - 24.0, dropdown.center().y);
        canvas.line(
            Pos2::new(chevron.x - 6.0, chevron.y - 3.0),
            Pos2::new(chevron.x, chevron.y + 3.0),
            Stroke::new(2.0, theme::TEXT_PRIMARY),
        );
        canvas.line(
            Pos2::new(chevron.x, chevron.y + 3.0),
            Pos2::new(chevron.x + 6.0, chevron.y - 3.0),
            Stroke::new(2.0, theme::TEXT_PRIMARY),
        );
        if response.focused {
            canvas.focus_ring(dropdown, 0);
        }
        if response.activated {
            self.avatar_edit.body_part_choice = self.avatar_edit.body_part;
            self.avatar_edit.body_part_menu = true;
        }

        if self.avatar_edit.body_part == 0 {
            self.show_bundle_grid(canvas, "BodyParts", "No full body packages found.");
            return;
        }
        let inventory = self
            .avatar_edit
            .inventory
            .get(&self.avatar_edit.body_part)
            .cloned();
        match inventory {
            Some(Load::Ready(bundle)) => {
                for url in bundle.urls.values() {
                    self.images.ensure(url);
                }
                let mut activate = None;
                for (index, item) in bundle.items.iter().take(10).enumerate() {
                    let column = index % 5;
                    let row = index / 5;
                    let rect = Rect::from_min_size(
                        Pos2::new(765.0 + column as f32 * 182.0, 460.0 + row as f32 * 182.0),
                        Vec2::new(150.0, 150.0),
                    );
                    let response = canvas.focusable(
                        format!("avatar:body-part:{}", item.asset_id),
                        rect,
                        Sense::click(),
                    );
                    if let Some(texture) = bundle
                        .urls
                        .get(&item.asset_id)
                        .and_then(|url| self.images.get(url))
                    {
                        canvas.image_cover(texture, rect, Color32::WHITE);
                    } else {
                        canvas.rect_filled(rect, 0, Color32::from_rgb(37, 70, 82));
                        canvas.text_centered(
                            rect,
                            util::initials(&item.name),
                            24.0,
                            theme::TEXT_MUTED,
                            true,
                        );
                    }
                    let worn = self.avatar_edit.worn.iter().any(|asset| {
                        asset.id == item.asset_id && asset.asset_type.id == item.asset_type
                    });
                    canvas.rect_stroke(
                        rect,
                        0,
                        if worn {
                            Stroke::new(3.0, theme::SUCCESS)
                        } else {
                            Stroke::new(1.0, Color32::from_white_alpha(22))
                        },
                    );
                    if response.focused {
                        canvas.focus_ring(rect, 0);
                    }
                    canvas.text(
                        Pos2::new(rect.min.x, rect.max.y + 10.0),
                        Align2::LEFT_TOP,
                        util::truncate_to_width(&item.name, rect.width(), |text| {
                            canvas.text_width(text, 14.0, false)
                        }),
                        14.0,
                        if worn {
                            theme::SUCCESS
                        } else {
                            theme::TEXT_SECONDARY
                        },
                        false,
                    );
                    if response.activated {
                        activate = Some(item.clone());
                    }
                }
                if bundle.items.is_empty() {
                    canvas.text(
                        Pos2::new(765.0, 460.0),
                        Align2::LEFT_TOP,
                        "No items for this body part.",
                        18.0,
                        theme::TEXT_MUTED,
                        false,
                    );
                }
                if let Some(item) = activate {
                    self.toggle_avatar_asset(&item);
                }
            }
            Some(Load::Loading) | Some(Load::Idle) | None => {
                canvas.text(
                    Pos2::new(765.0, 460.0),
                    Align2::LEFT_TOP,
                    "Loading body parts...",
                    18.0,
                    theme::TEXT_MUTED,
                    false,
                );
            }
            Some(Load::Failed(error)) => {
                canvas.text(
                    Pos2::new(765.0, 460.0),
                    Align2::LEFT_TOP,
                    error,
                    17.0,
                    theme::NEGATIVE,
                    false,
                );
            }
        }
        self.draw_avatar_status(canvas, "Click a body part to wear it.");
    }

    fn show_body_part_menu(&mut self, canvas: &mut Canvas) {
        let overlay = Rect::from_min_size(Pos2::ZERO, Vec2::new(1920.0, 1080.0));
        canvas.rect_filled(overlay, 0, Color32::from_black_alpha(180));
        let panel = Rect::from_center_size(Pos2::new(1260.0, 560.0), Vec2::new(500.0, 610.0));
        canvas.rect_filled(panel, 0, theme::PANEL_SOLID);
        canvas.rect_stroke(panel, 0, Stroke::new(1.0, Color32::from_white_alpha(35)));
        canvas.text(
            Pos2::new(panel.min.x + 34.0, panel.min.y + 42.0),
            Align2::LEFT_CENTER,
            "Show Body Parts",
            26.0,
            theme::TEXT_PRIMARY,
            false,
        );
        let close = Rect::from_min_size(
            Pos2::new(panel.max.x - 56.0, panel.min.y + 22.0),
            Vec2::splat(34.0),
        );
        let close_response = canvas.focusable("avatar:body-parts-close", close, Sense::click());
        canvas.text_centered(close, "X", 20.0, theme::TEXT_SECONDARY, false);
        if close_response.activated {
            self.avatar_edit.body_part_menu = false;
            return;
        }
        let mut selected = self.avatar_edit.body_part_choice;
        for (index, (part, label)) in [
            (0u64, "Full Body"),
            (27, "Torso"),
            (28, "Right Arm"),
            (29, "Left Arm"),
            (30, "Right Leg"),
            (31, "Left Leg"),
        ]
        .iter()
        .enumerate()
        {
            let rect = Rect::from_min_size(
                Pos2::new(panel.min.x + 34.0, panel.min.y + 96.0 + index as f32 * 68.0),
                Vec2::new(panel.width() - 68.0, 52.0),
            );
            let response = canvas.focusable(
                format!("avatar:body-part-option:{part}"),
                rect,
                Sense::click(),
            );
            let center = Pos2::new(rect.min.x + 18.0, rect.center().y);
            canvas.circle_stroke(center, 11.0, Stroke::new(2.0, theme::TEXT_SECONDARY));
            if selected == *part {
                canvas.circle_filled(center, 6.0, theme::TEXT_PRIMARY);
            }
            canvas.text(
                Pos2::new(rect.min.x + 46.0, rect.center().y),
                Align2::LEFT_CENTER,
                *label,
                22.0,
                theme::TEXT_PRIMARY,
                false,
            );
            if response.activated {
                self.avatar_edit.body_part_choice = *part;
                selected = *part;
            }
        }
        if screens::button(
            canvas,
            "avatar:body-parts-apply",
            Rect::from_min_size(
                Pos2::new(panel.min.x + 34.0, panel.max.y - 84.0),
                Vec2::new(panel.width() - 68.0, 52.0),
            ),
            "Apply",
            ButtonKind::Primary,
            true,
        ) {
            self.avatar_edit.body_part = selected;
            self.avatar_edit.body_part_menu = false;
            self.avatar_edit.scroll = 0.0;
            self.avatar_edit.status = None;
        }
    }

    fn show_animation_bundles(&mut self, canvas: &mut Canvas) {
        let none = Rect::from_min_size(Pos2::new(765.0, 460.0), Vec2::new(208.0, 208.0));
        let response = canvas.focusable("avatar:animation-none", none, Sense::click());
        canvas.rect_filled(
            none,
            0,
            if response.focused {
                Color32::from_white_alpha(20)
            } else {
                theme::panel(190)
            },
        );
        canvas.rect_stroke(
            none,
            0,
            if response.focused {
                Stroke::new(3.0, theme::FOCUS)
            } else {
                Stroke::new(1.0, Color32::from_white_alpha(24))
            },
        );
        canvas.text_centered(
            none,
            "None",
            30.0,
            if response.focused {
                theme::FOCUS
            } else {
                theme::TEXT_SECONDARY
            },
            false,
        );
        canvas.text(
            Pos2::new(none.min.x, none.max.y + 9.0),
            Align2::LEFT_TOP,
            "None",
            15.0,
            theme::TEXT_SECONDARY,
            false,
        );
        if response.activated {
            self.clear_animations();
        }
        self.show_bundle_grid_offset(
            canvas,
            "AvatarAnimations",
            "No animation packages found.",
            1,
        );
    }

    fn clear_animations(&mut self) {
        self.avatar_edit
            .worn
            .retain(|asset| !matches!(asset.asset_type.id, 48 | 50 | 51 | 52 | 53 | 54 | 55));
        self.avatar_edit.status = Some(("Clearing animations...".to_owned(), false));
        self.queue_avatar_save();
    }

    fn show_bundle_grid(&mut self, canvas: &mut Canvas, bundle_type: &str, empty: &str) {
        self.show_bundle_grid_offset(canvas, bundle_type, empty, 0);
    }

    fn show_bundle_grid_offset(
        &mut self,
        canvas: &mut Canvas,
        bundle_type: &str,
        empty: &str,
        offset_slots: usize,
    ) {
        let state = self.avatar_edit.bundles.clone();
        let bundles = match state {
            Load::Ready(bundle) => bundle
                .bundles
                .into_iter()
                .filter(|bundle| bundle.bundle_type == bundle_type)
                .collect::<Vec<AvatarBundle>>(),
            Load::Loading | Load::Idle => {
                canvas.text(
                    Pos2::new(765.0, 460.0),
                    Align2::LEFT_TOP,
                    "Loading packages...",
                    18.0,
                    theme::TEXT_MUTED,
                    false,
                );
                return;
            }
            Load::Failed(error) => {
                canvas.text(
                    Pos2::new(765.0, 460.0),
                    Align2::LEFT_TOP,
                    error,
                    17.0,
                    theme::NEGATIVE,
                    false,
                );
                return;
            }
        };
        if bundles.is_empty() {
            canvas.text(
                Pos2::new(765.0, 460.0),
                Align2::LEFT_TOP,
                empty,
                18.0,
                theme::TEXT_MUTED,
                false,
            );
            return;
        }
        let urls = match &self.avatar_edit.bundles {
            Load::Ready(bundle) => bundle.urls.clone(),
            _ => Default::default(),
        };
        let mut equip = None;
        for (index, bundle) in bundles.iter().enumerate() {
            let index = index + offset_slots;
            let column = index % 4;
            let row = index / 4;
            let rect = Rect::from_min_size(
                Pos2::new(765.0 + column as f32 * 230.0, 460.0 + row as f32 * 236.0),
                Vec2::new(208.0, 208.0),
            );
            let response =
                canvas.focusable(format!("avatar:bundle:{}", bundle.id), rect, Sense::click());
            if let Some(url) = urls.get(&bundle.id)
                && let Some(texture) = self.images.get(url)
            {
                canvas.image_cover(texture, rect, Color32::WHITE);
            } else {
                canvas.rect_filled(rect, 0, theme::panel(190));
                canvas.text_centered(
                    rect,
                    util::initials(&bundle.name),
                    28.0,
                    theme::TEXT_MUTED,
                    true,
                );
            }
            canvas.rect_stroke(
                rect,
                0,
                if response.focused {
                    Stroke::new(3.0, theme::FOCUS)
                } else {
                    Stroke::new(1.0, Color32::from_white_alpha(24))
                },
            );
            canvas.text(
                Pos2::new(rect.min.x, rect.max.y + 9.0),
                Align2::LEFT_TOP,
                util::truncate_to_width(&bundle.name, rect.width(), |text| {
                    canvas.text_width(text, 15.0, false)
                }),
                15.0,
                theme::TEXT_SECONDARY,
                false,
            );
            if response.activated {
                equip = Some(bundle.id);
            }
        }
        if let Some(bundle_id) = equip {
            self.equip_avatar_bundle(bundle_id);
        }
    }

    fn equip_avatar_bundle(&mut self, bundle_id: u64) {
        if self.avatar_edit.appearance_saving {
            return;
        }
        let Some(token) = self
            .session
            .as_ref()
            .map(|session| session.token.expose_secret().to_owned())
        else {
            self.avatar_edit.status = Some(("Sign in to wear a package.".to_owned(), true));
            return;
        };
        self.avatar_edit.appearance_saving = true;
        self.avatar_edit.status = Some(("Equipping package...".to_owned(), false));
        self.api.equip_bundle(token, bundle_id, self.api_tx.clone());
    }

    fn draw_avatar_category_nav(&mut self, canvas: &mut Canvas) -> bool {
        let mut x = 765.0;
        let mut selected = None;
        for category in AvatarCategory::ALL {
            let label = category.label();
            let width = canvas.text_width(label, 18.0, false) + 24.0;
            let rect = Rect::from_min_size(Pos2::new(x, 288.0), Vec2::new(width, 40.0));
            let response =
                canvas.focusable(format!("avatar:category:{}", label), rect, Sense::click());
            let active = self.avatar_edit.category == category;
            canvas.text_centered(
                rect,
                label,
                18.0,
                if active {
                    theme::TEXT_PRIMARY
                } else if response.focused {
                    theme::FOCUS
                } else {
                    theme::TEXT_SECONDARY
                },
                false,
            );
            if active {
                canvas.line(
                    Pos2::new(rect.min.x + 12.0, rect.max.y - 2.0),
                    Pos2::new(rect.max.x - 12.0, rect.max.y - 2.0),
                    Stroke::new(2.0, theme::ACCENT_BRIGHT),
                );
            }
            if response.activated {
                selected = Some(category);
            }
            x += width + 8.0;
        }
        if let Some(category) = selected
            && category != self.avatar_edit.category
        {
            self.avatar_edit.category = category;
            self.avatar_edit.selected_type = category.first_type();
            self.avatar_edit.scroll = 0.0;
            self.avatar_edit.body_part_menu = false;
            self.avatar_edit.status = None;
            return true;
        }
        false
    }

    fn draw_avatar_subcategory_nav(&mut self, canvas: &mut Canvas) -> bool {
        let subcategories = self.avatar_edit.category.subcategories();
        let mut x = 765.0;
        let mut selected = None;
        for (asset_type, label) in subcategories {
            let width = canvas.text_width(label, 15.0, false) + 22.0;
            let rect = Rect::from_min_size(Pos2::new(x, 340.0), Vec2::new(width, 32.0));
            let response =
                canvas.focusable(format!("avatar:type:{asset_type}"), rect, Sense::click());
            let active = self.avatar_edit.selected_type == *asset_type;
            canvas.rect_filled(
                rect,
                0,
                if active {
                    theme::ACCENT
                } else {
                    theme::SECONDARY_BUTTON
                },
            );
            canvas.text_centered(
                rect,
                label,
                15.0,
                if active {
                    theme::TEXT_PRIMARY
                } else if response.focused {
                    theme::FOCUS
                } else {
                    theme::TEXT_SECONDARY
                },
                false,
            );
            if response.focused {
                canvas.focus_ring(rect, 0);
            }
            if response.activated {
                selected = Some(*asset_type);
            }
            x += width + 8.0;
        }
        if let Some(asset_type) = selected
            && asset_type != self.avatar_edit.selected_type
        {
            self.avatar_edit.selected_type = asset_type;
            self.avatar_edit.body_part_menu = false;
            self.avatar_edit.status = None;
            return true;
        }
        false
    }

    fn show_skin_tones(&mut self, canvas: &mut Canvas) {
        let part_x = 765.0;
        for (index, part) in SkinPart::ALL.iter().enumerate() {
            let rect = Rect::from_min_size(
                Pos2::new(part_x, 438.0 + index as f32 * 48.0),
                Vec2::new(190.0, 40.0),
            );
            let response =
                canvas.focusable(format!("avatar:skin-part:{index}"), rect, Sense::click());
            let center = Pos2::new(rect.min.x + 14.0, rect.center().y);
            canvas.circle_stroke(center, 10.0, Stroke::new(2.0, theme::TEXT_SECONDARY));
            if self.avatar_edit.skin_part == *part {
                canvas.circle_filled(center, 5.5, theme::TEXT_PRIMARY);
            }
            canvas.text(
                Pos2::new(rect.min.x + 38.0, rect.center().y),
                Align2::LEFT_CENTER,
                part.label(),
                19.0,
                theme::TEXT_PRIMARY,
                false,
            );
            if response.activated {
                self.avatar_edit.skin_part = *part;
                return;
            }
        }

        let sv = Rect::from_min_size(Pos2::new(1000.0, 430.0), Vec2::new(280.0, 210.0));
        let hue_bar = Rect::from_min_size(Pos2::new(1294.0, 430.0), Vec2::new(24.0, 210.0));
        let (pointer, down, pressed) = canvas.ui.input(|input| {
            (
                input.pointer.interact_pos(),
                input.pointer.primary_down(),
                input.pointer.primary_pressed(),
            )
        });
        let sv_screen = canvas.screen_rect(sv);
        let hue_screen = canvas.screen_rect(hue_bar);
        if pressed && let Some(pointer) = pointer {
            if sv_screen.contains(pointer) {
                self.avatar_edit.picker_drag = Some(1);
            } else if hue_screen.contains(pointer) {
                self.avatar_edit.picker_drag = Some(2);
            }
        }
        if !down {
            self.avatar_edit.picker_drag = None;
        }
        if down
            && let Some(pointer) = pointer
            && let Some(kind) = self.avatar_edit.picker_drag
        {
            if kind == 1 {
                self.avatar_edit.picker_sat =
                    ((pointer.x - sv_screen.min.x) / sv_screen.width()).clamp(0.0, 1.0);
                self.avatar_edit.picker_val =
                    (1.0 - (pointer.y - sv_screen.min.y) / sv_screen.height()).clamp(0.0, 1.0);
            } else {
                self.avatar_edit.picker_hue =
                    ((pointer.y - hue_screen.min.y) / hue_screen.height()).clamp(0.0, 1.0);
            }
        }
        let (r, g, b) = hsv_to_rgb(
            self.avatar_edit.picker_hue,
            self.avatar_edit.picker_sat,
            self.avatar_edit.picker_val,
        );
        self.avatar_edit.picker_hex = format!("#{r:02X}{g:02X}{b:02X}");

        let hue_rgb = hsv_to_rgb(self.avatar_edit.picker_hue, 1.0, 1.0);
        let hue_color = Color32::from_rgb(hue_rgb.0, hue_rgb.1, hue_rgb.2);
        let mut mesh = egui::Mesh::default();
        for (pos, color) in [
            (Pos2::new(sv.min.x, sv.min.y), Color32::WHITE),
            (Pos2::new(sv.max.x, sv.min.y), hue_color),
            (Pos2::new(sv.min.x, sv.max.y), Color32::BLACK),
            (Pos2::new(sv.max.x, sv.max.y), Color32::BLACK),
        ] {
            mesh.vertices.push(egui::epaint::Vertex {
                pos: canvas.pos(pos.x, pos.y),
                uv: Pos2::ZERO,
                color,
            });
        }
        mesh.indices.extend_from_slice(&[0, 1, 2, 2, 1, 3]);
        canvas.painter.add(egui::Shape::mesh(mesh));
        canvas.rect_stroke(sv, 0, Stroke::new(1.0, Color32::from_white_alpha(40)));
        for index in 0..24 {
            let t0 = index as f32 / 24.0;
            let t1 = (index + 1) as f32 / 24.0;
            let color = hsv_to_rgb(t0, 1.0, 1.0);
            canvas.rect_filled(
                Rect::from_min_max(
                    Pos2::new(hue_bar.min.x, hue_bar.min.y + hue_bar.height() * t0),
                    Pos2::new(hue_bar.max.x, hue_bar.min.y + hue_bar.height() * t1),
                ),
                0,
                Color32::from_rgb(color.0, color.1, color.2),
            );
        }
        canvas.rect_stroke(hue_bar, 0, Stroke::new(1.0, Color32::from_white_alpha(40)));
        canvas.circle_stroke(
            Pos2::new(
                sv.min.x + self.avatar_edit.picker_sat * sv.width(),
                sv.min.y + (1.0 - self.avatar_edit.picker_val) * sv.height(),
            ),
            8.0,
            Stroke::new(2.0, Color32::WHITE),
        );
        canvas.line(
            Pos2::new(
                hue_bar.min.x - 4.0,
                hue_bar.min.y + self.avatar_edit.picker_hue * hue_bar.height(),
            ),
            Pos2::new(
                hue_bar.max.x + 4.0,
                hue_bar.min.y + self.avatar_edit.picker_hue * hue_bar.height(),
            ),
            Stroke::new(3.0, Color32::WHITE),
        );

        let fields = [
            ("Hex", self.avatar_edit.picker_hex.clone()),
            ("R", r.to_string()),
            ("G", g.to_string()),
            ("B", b.to_string()),
        ];
        for (index, (label, value)) in fields.iter().enumerate() {
            let rect = Rect::from_min_size(
                Pos2::new(1000.0 + index as f32 * 118.0, 666.0),
                Vec2::new(106.0, 46.0),
            );
            canvas.rect_filled(rect, 0, theme::panel(220));
            canvas.rect_stroke(rect, 0, Stroke::new(1.0, Color32::from_white_alpha(30)));
            canvas.text(
                Pos2::new(rect.min.x + 10.0, rect.center().y),
                Align2::LEFT_CENTER,
                *label,
                14.0,
                theme::TEXT_MUTED,
                false,
            );
            canvas.text(
                Pos2::new(rect.min.x + 40.0, rect.center().y),
                Align2::LEFT_CENTER,
                value,
                17.0,
                theme::TEXT_PRIMARY,
                false,
            );
        }

        if screens::button(
            canvas,
            "avatar:apply-skin",
            Rect::from_min_size(Pos2::new(1000.0, 740.0), Vec2::new(220.0, 52.0)),
            "Apply Colour",
            ButtonKind::Primary,
            true,
        ) {
            let (id, _) = nearest_skin_tone(r, g, b);
            self.apply_skin_color(id);
        }
        let current = self
            .avatar_edit
            .body_colors
            .clone()
            .map(|colors| skin_color_for_part(&colors, self.avatar_edit.skin_part))
            .unwrap_or(101);
        canvas.text(
            Pos2::new(1000.0, 816.0),
            Align2::LEFT_CENTER,
            format!("Current {} colour", self.avatar_edit.skin_part.label()),
            16.0,
            theme::TEXT_MUTED,
            false,
        );
        if let Some((_, _, hex)) = SKIN_TONES.iter().find(|(id, _, _)| *id == current) {
            let color = Color32::from_rgb(
                ((hex >> 16) & 0xff) as u8,
                ((hex >> 8) & 0xff) as u8,
                (hex & 0xff) as u8,
            );
            canvas.circle_filled(Pos2::new(1030.0, 850.0), 18.0, color);
        }
        self.draw_avatar_status(canvas, "Choose a body part and a colour.");
    }

    fn show_build(&mut self, canvas: &mut Canvas) {
        let mut scales =
            self.avatar_edit
                .scales
                .clone()
                .unwrap_or_else(|| crate::models::AvatarScales {
                    height: 1.0,
                    width: 1.0,
                    head: 1.0,
                    depth: 1.0,
                    proportion: 0.0,
                    body_type: 0.0,
                });

        canvas.text(
            Pos2::new(765.0, 410.0),
            Align2::LEFT_CENTER,
            "Body Type",
            20.0,
            theme::TEXT_PRIMARY,
            false,
        );
        let triangle = Rect::from_min_size(Pos2::new(765.0, 432.0), Vec2::new(300.0, 190.0));
        let top_left = Pos2::new(triangle.center().x - 95.0, triangle.min.y + 8.0);
        let top_right = Pos2::new(triangle.center().x + 95.0, triangle.min.y + 8.0);
        let bottom = Pos2::new(triangle.center().x, triangle.max.y - 8.0);
        canvas.painter.add(egui::Shape::convex_polygon(
            vec![
                canvas.pos(top_left.x, top_left.y),
                canvas.pos(top_right.x, top_right.y),
                canvas.pos(bottom.x, bottom.y),
            ],
            Color32::from_rgb(31, 48, 58),
            Stroke::new(1.0, Color32::from_white_alpha(25)),
        ));
        canvas.text(
            Pos2::new(top_left.x - 18.0, top_left.y + 18.0),
            Align2::CENTER_CENTER,
            "Slim",
            14.0,
            theme::TEXT_MUTED,
            false,
        );
        canvas.text(
            Pos2::new(top_right.x + 18.0, top_right.y + 18.0),
            Align2::CENTER_CENTER,
            "Broad",
            14.0,
            theme::TEXT_MUTED,
            false,
        );
        canvas.text(
            Pos2::new(bottom.x, bottom.y + 20.0),
            Align2::CENTER_CENTER,
            "Rthro",
            14.0,
            theme::TEXT_MUTED,
            false,
        );
        let half_width = (top_right.x - top_left.x) * 0.5;
        let body_type = scales.body_type.clamp(0.0, 1.0) as f32;
        let proportion = scales.proportion.clamp(0.0, 1.0) as f32;
        let knob_y = top_left.y + (bottom.y - top_left.y) * body_type;
        let knob_x =
            triangle.center().x + (proportion * 2.0 - 1.0) * half_width * (1.0 - body_type);
        let triangle_screen = canvas.screen_rect(triangle);
        let (pointer, down, pressed) = canvas.ui.input(|input| {
            (
                input.pointer.interact_pos(),
                input.pointer.primary_down(),
                input.pointer.primary_pressed(),
            )
        });
        if pressed
            && let Some(pointer) = pointer
            && triangle_screen.contains(pointer)
            && self.avatar_edit.build_drag.is_none()
        {
            self.avatar_edit.build_drag = Some(1);
        }
        if down
            && self.avatar_edit.build_drag == Some(1)
            && let Some(pointer) = pointer
        {
            let body_type = ((pointer.y - triangle_screen.min.y - 8.0)
                / (triangle_screen.height() - 16.0))
                .clamp(0.0, 1.0);
            let half = (triangle_screen.width() * 0.5) * (1.0 - body_type);
            let proportion = if half > 1.0 {
                (((pointer.x - triangle_screen.center().x) / half) * 0.5 + 0.5).clamp(0.0, 1.0)
            } else {
                0.5
            };
            scales.proportion = proportion as f64;
            scales.body_type = body_type as f64;
        }
        canvas.circle_filled(Pos2::new(knob_x, knob_y), 10.0, Color32::WHITE);

        canvas.text(
            Pos2::new(1120.0, 410.0),
            Align2::LEFT_CENTER,
            "Body Size",
            20.0,
            theme::TEXT_PRIMARY,
            false,
        );
        let pad = Rect::from_min_size(Pos2::new(1120.0, 432.0), Vec2::new(230.0, 230.0));
        canvas.rect_filled(pad, 0, Color32::from_rgb(31, 48, 58));
        canvas.rect_stroke(pad, 0, Stroke::new(1.0, Color32::from_white_alpha(25)));
        let pad_knob = Pos2::new(
            pad.min.x + ((scales.width as f32 - 0.70) / 0.30).clamp(0.0, 1.0) * pad.width(),
            pad.min.y
                + (1.0 - ((scales.height as f32 - 0.90) / 0.15).clamp(0.0, 1.0)) * pad.height(),
        );
        let pad_screen = canvas.screen_rect(pad);
        if pressed
            && let Some(pointer) = pointer
            && pad_screen.contains(pointer)
            && self.avatar_edit.build_drag.is_none()
        {
            self.avatar_edit.build_drag = Some(2);
        }
        if down
            && self.avatar_edit.build_drag == Some(2)
            && let Some(pointer) = pointer
        {
            scales.width = (0.70
                + ((pointer.x - pad_screen.min.x) / pad_screen.width()).clamp(0.0, 1.0) * 0.30)
                as f64;
            scales.height = (0.90
                + (1.0 - (pointer.y - pad_screen.min.y) / pad_screen.height()).clamp(0.0, 1.0)
                    * 0.15) as f64;
        }
        canvas.circle_filled(pad_knob, 10.0, Color32::WHITE);
        canvas.text(
            Pos2::new(pad.center().x, pad.max.y + 18.0),
            Align2::CENTER_CENTER,
            "Width / Height",
            14.0,
            theme::TEXT_MUTED,
            false,
        );

        canvas.text(
            Pos2::new(765.0, 700.0),
            Align2::LEFT_CENTER,
            "Head Size",
            20.0,
            theme::TEXT_PRIMARY,
            false,
        );
        let slider = Rect::from_min_size(Pos2::new(765.0, 730.0), Vec2::new(680.0, 6.0));
        canvas.rect_filled(slider, 3, Color32::from_white_alpha(45));
        let head_t = ((scales.head as f32 - 0.95) / 0.05).clamp(0.0, 1.0);
        let slider_screen = canvas.screen_rect(slider.expand(18.0));
        if pressed
            && let Some(pointer) = pointer
            && slider_screen.contains(pointer)
            && self.avatar_edit.build_drag.is_none()
        {
            self.avatar_edit.build_drag = Some(3);
        }
        if down
            && self.avatar_edit.build_drag == Some(3)
            && let Some(pointer) = pointer
        {
            scales.head = (0.95
                + ((pointer.x - slider_screen.min.x) / slider_screen.width()).clamp(0.0, 1.0)
                    * 0.05) as f64;
        }
        canvas.circle_filled(
            Pos2::new(slider.min.x + head_t * slider.width(), slider.center().y),
            9.0,
            Color32::WHITE,
        );

        canvas.text(
            Pos2::new(765.0, 810.0),
            Align2::LEFT_CENTER,
            "R6 / R15",
            20.0,
            theme::TEXT_PRIMARY,
            false,
        );
        let current_type = self
            .avatar_edit
            .player_avatar_type
            .clone()
            .unwrap_or_else(|| "R15".to_owned());
        for (index, kind) in ["R6", "R15"].iter().enumerate() {
            let rect = Rect::from_min_size(
                Pos2::new(765.0 + index as f32 * 112.0, 834.0),
                Vec2::new(100.0, 42.0),
            );
            let active = current_type.eq_ignore_ascii_case(kind);
            if screens::button(
                canvas,
                &format!("avatar:avatar-type:{kind}"),
                rect,
                kind,
                if active {
                    ButtonKind::Primary
                } else {
                    ButtonKind::Secondary
                },
                true,
            ) {
                self.set_avatar_type(kind);
            }
        }

        if !down {
            self.avatar_edit.build_drag = None;
        }
        self.avatar_edit.scales = Some(scales.clone());
        if screens::button(
            canvas,
            "avatar:apply-build",
            Rect::from_min_size(Pos2::new(1420.0, 800.0), Vec2::new(220.0, 52.0)),
            "Apply Build",
            ButtonKind::Primary,
            true,
        ) {
            self.set_build(
                scales.height,
                scales.width,
                scales.head,
                scales.depth,
                scales.proportion,
                scales.body_type,
            );
        }
        self.draw_avatar_status(canvas, "Drag the controls, then apply.");
    }

    fn draw_avatar_status(&self, canvas: &mut Canvas, fallback: &str) {
        if let Some((message, error)) = &self.avatar_edit.status {
            canvas.text(
                Pos2::new(765.0, 966.0),
                Align2::LEFT_CENTER,
                message,
                17.0,
                if *error {
                    theme::NEGATIVE
                } else {
                    theme::SUCCESS
                },
                false,
            );
        } else {
            canvas.text(
                Pos2::new(765.0, 966.0),
                Align2::LEFT_CENTER,
                if self.avatar_edit.saving || self.avatar_edit.appearance_saving {
                    "Updating avatar..."
                } else {
                    fallback
                },
                17.0,
                theme::TEXT_MUTED,
                false,
            );
        }
    }

    fn show_outfits(&mut self, canvas: &mut Canvas) {
        let outfits = self.avatar_edit.outfits.clone();
        if outfits.is_empty() {
            canvas.text(
                Pos2::new(765.0, 378.0),
                Align2::LEFT_TOP,
                "No saved outfits yet. Put together a look, then save it below.",
                20.0,
                theme::TEXT_MUTED,
                false,
            );
        }
        let mut equip = None;
        for (index, outfit) in outfits.iter().take(9).enumerate() {
            let column = index % 3;
            let row = index / 3;
            let rect = Rect::from_min_size(
                Pos2::new(765.0 + column as f32 * 318.0, 348.0 + row as f32 * 116.0),
                Vec2::new(300.0, 96.0),
            );
            let response = canvas.focusable(format!("avatar:outfit:{index}"), rect, Sense::click());
            canvas.rect_filled(
                rect,
                0,
                if response.focused {
                    Color32::from_white_alpha(20)
                } else {
                    theme::panel(190)
                },
            );
            canvas.rect_stroke(
                rect,
                0,
                if response.focused {
                    Stroke::new(2.0, theme::FOCUS)
                } else {
                    Stroke::new(1.0, Color32::from_white_alpha(22))
                },
            );
            canvas.text(
                Pos2::new(rect.min.x + 20.0, rect.min.y + 32.0),
                Align2::LEFT_CENTER,
                util::truncate_to_width(&outfit.name, rect.width() - 118.0, |text| {
                    canvas.text_width(text, 22.0, false)
                }),
                22.0,
                theme::TEXT_PRIMARY,
                false,
            );
            canvas.text(
                Pos2::new(rect.min.x + 20.0, rect.min.y + 66.0),
                Align2::LEFT_CENTER,
                format!("{} items", outfit.assets.len()),
                16.0,
                theme::TEXT_MUTED,
                false,
            );
            canvas.text(
                Pos2::new(rect.max.x - 26.0, rect.center().y),
                Align2::RIGHT_CENTER,
                "EQUIP",
                17.0,
                if response.focused {
                    theme::FOCUS
                } else {
                    theme::ACCENT_BRIGHT
                },
                false,
            );
            if response.activated {
                equip = Some(index);
            }
        }

        let field = Rect::from_min_size(Pos2::new(765.0, 708.0), Vec2::new(420.0, 52.0));
        canvas.rect_filled(field, 0, theme::panel(220));
        canvas.rect_stroke(
            field,
            0,
            if canvas.is_focused("avatar:outfit-name") {
                Stroke::new(2.0, theme::ACCENT)
            } else {
                Stroke::new(1.0, Color32::from_white_alpha(28))
            },
        );
        let response = canvas.ui.put(
            canvas.screen_rect(field),
            egui::TextEdit::singleline(&mut self.avatar_edit.outfit_name)
                .frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(16, 14)))
                .hint_text("Outfit name")
                .font(egui::FontId::new(
                    20.0 * canvas.scale,
                    egui::FontFamily::Proportional,
                ))
                .text_color(theme::TEXT_PRIMARY)
                .align(egui::Align2::LEFT_CENTER)
                .min_size(canvas.screen_rect(field).size()),
        );
        if response.clicked() {
            canvas.focus.set("avatar:outfit-name");
        }
        if screens::button(
            canvas,
            "avatar:save-outfit",
            Rect::from_min_size(Pos2::new(1205.0, 708.0), Vec2::new(240.0, 52.0)),
            "Save Outfit",
            ButtonKind::Primary,
            true,
        ) {
            self.save_current_outfit();
        }

        if let Some(index) = equip {
            self.equip_outfit(index);
            return;
        }
        self.draw_avatar_status(canvas, "Select an outfit to equip it.");
    }

    fn show_friends_tab(&mut self, canvas: &mut Canvas) {
        let data = self.session.as_ref().map(|session| {
            (
                session.friends.ready().cloned(),
                session.friends.error().map(str::to_owned),
                session.presence.clone(),
                session.headshots.clone(),
            )
        });

        let Some((friends, error, presence, headshots)) = data else {
            self.signed_out_tab(canvas, "Friends", "Sign in to see your friends.");
            return;
        };

        screens::section_label(canvas, 100.0, 252.0, "Friends");

        let add_rect = Rect::from_min_size(Pos2::new(1580.0, 160.0), Vec2::new(240.0, 100.0));
        let add = canvas.focusable("friends:add", add_rect, Sense::click());
        let add_center = Pos2::new(add_rect.center().x, add_rect.min.y + 32.0);
        canvas.circle_filled(
            add_center,
            30.0,
            Color32::from_rgba_unmultiplied(40, 52, 60, 220),
        );
        canvas.circle_stroke(
            add_center,
            30.0,
            Stroke::new(1.0, Color32::from_white_alpha(45)),
        );
        let icon_color = theme::TEXT_PRIMARY;
        // A clean outlined person with a separate plus, instead of the old
        // lone line that read as a minus sign.
        canvas.circle_stroke(
            Pos2::new(add_center.x - 7.0, add_center.y - 8.0),
            6.0,
            Stroke::new(1.6, icon_color),
        );
        canvas.rect_stroke(
            Rect::from_min_size(
                Pos2::new(add_center.x - 15.0, add_center.y + 2.0),
                Vec2::new(16.0, 11.0),
            ),
            4,
            Stroke::new(1.6, icon_color),
        );
        canvas.line(
            Pos2::new(add_center.x + 9.0, add_center.y - 8.0),
            Pos2::new(add_center.x + 17.0, add_center.y - 8.0),
            Stroke::new(2.0, icon_color),
        );
        canvas.line(
            Pos2::new(add_center.x + 13.0, add_center.y - 12.0),
            Pos2::new(add_center.x + 13.0, add_center.y - 4.0),
            Stroke::new(2.0, icon_color),
        );
        canvas.text_centered(
            Rect::from_min_size(
                Pos2::new(add_rect.min.x, add_rect.min.y + 66.0),
                Vec2::new(add_rect.width(), 28.0),
            ),
            "Add Friends",
            18.0,
            if add.focused {
                theme::FOCUS
            } else {
                theme::TEXT_SECONDARY
            },
            false,
        );
        if add.focused || add.hovered {
            let outline = Rect::from_center_size(
                Pos2::new(add_center.x, add_center.y + 18.0),
                Vec2::splat(112.0),
            );
            canvas.rect_stroke(outline, 0, Stroke::new(4.0, theme::FOCUS));
        }
        if add.activated {
            self.open_add_friend(None);
            return;
        }

        let Some(friends) = friends else {
            canvas.text(
                Pos2::new(104.0, 320.0),
                Align2::LEFT_TOP,
                if let Some(error) = error {
                    error
                } else {
                    "Loading friends...".to_owned()
                },
                22.0,
                theme::TEXT_SECONDARY,
                false,
            );
            if screens::button(
                canvas,
                "placeholder:friends-retry",
                Rect::from_min_size(Pos2::new(104.0, 390.0), Vec2::new(180.0, 48.0)),
                "Retry",
                ButtonKind::Secondary,
                true,
            ) {
                self.load_account_data();
            }
            return;
        };

        if friends.is_empty() {
            canvas.text(
                Pos2::new(104.0, 320.0),
                Align2::LEFT_TOP,
                "No friends to show.",
                24.0,
                theme::TEXT_SECONDARY,
                false,
            );
            return;
        }

        let mut rows: Vec<FriendRow> = friends
            .iter()
            .map(|friend| friend_row(friend, &presence, &headshots))
            .collect();
        rows.sort_by(|a, b| {
            b.online
                .cmp(&a.online)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        for row in &rows {
            if let Some(url) = &row.headshot_url {
                self.images.ensure(url);
            }
        }
        let textures: Vec<Option<TextureHandle>> = rows
            .iter()
            .map(|row| {
                row.headshot_url
                    .as_ref()
                    .and_then(|url| self.images.get(url))
                    .cloned()
            })
            .collect();

        let viewport = Rect::from_min_size(Pos2::new(96.0, 292.0), Vec2::new(1728.0, 718.0));
        let content_height = rows.len() as f32 * 86.0;
        let max_scroll = (content_height - viewport.height()).max(0.0);
        let wheel = canvas.ui.input(|input| input.smooth_scroll_delta.y);
        if wheel != 0.0 {
            self.friends_scroll = (self.friends_scroll - wheel).clamp(0.0, max_scroll);
        }
        self.friends_scroll = self.friends_scroll.clamp(0.0, max_scroll);

        let clip = canvas.screen_rect(viewport);
        let clipped = canvas.painter.with_clip_rect(clip);
        let original_painter = std::mem::replace(&mut canvas.painter, clipped);
        for (index, row) in rows.iter().enumerate() {
            let x = 104.0;
            let y = viewport.min.y + index as f32 * 86.0 - self.friends_scroll;
            if y + 64.0 < viewport.min.y || y > viewport.max.y {
                continue;
            }
            let row_rect = Rect::from_min_size(Pos2::new(104.0, y - 8.0), Vec2::new(1280.0, 80.0));
            let row_response = canvas.focusable(
                format!("friends:row:{}", row.user_id),
                row_rect,
                Sense::click(),
            );
            if row_response.focused {
                canvas.rect_filled(row_rect, 0, Color32::from_white_alpha(16));
                canvas.rect_filled(
                    Rect::from_min_size(Pos2::new(96.0, y - 8.0), Vec2::new(4.0, 80.0)),
                    0,
                    theme::ACCENT_BRIGHT,
                );
            }
            let head = Rect::from_min_size(Pos2::new(x, y), Vec2::splat(64.0));
            match textures.get(index).and_then(|texture| texture.as_ref()) {
                Some(texture) => canvas.image_cover(texture, head, Color32::WHITE),
                None => draw_avatar_box(canvas, head),
            }
            canvas.text_with_shadow(
                Pos2::new(x + 80.0, y + 18.0),
                Align2::LEFT_CENTER,
                &row.name,
                22.0,
                theme::TEXT_PRIMARY,
                false,
            );
            let dot = Pos2::new(x + 84.0, y + 46.0);
            canvas.circle_filled(
                dot,
                4.0,
                if row.online {
                    theme::SUCCESS
                } else {
                    theme::TEXT_MUTED
                },
            );
            canvas.text_with_shadow(
                Pos2::new(dot.x + 12.0, dot.y),
                Align2::LEFT_CENTER,
                &row.presence,
                16.0,
                theme::TEXT_MUTED,
                false,
            );
            if row.online
                && row.place_id > 0
                && screens::button(
                    canvas,
                    &format!("friends:join:{}", row.user_id),
                    Rect::from_min_size(Pos2::new(1500.0, y + 12.0), Vec2::new(140.0, 44.0)),
                    "Join",
                    ButtonKind::Play,
                    true,
                )
            {
                self.open_launch(
                    format!("{}'s game", row.name),
                    row.place_id,
                    Screen::Placeholder,
                );
                return;
            }
            if row_response.activated {
                self.open_profile(
                    row.user_id,
                    row.name.clone(),
                    row.username.clone(),
                    row.headshot_url.clone(),
                    row.presence.clone(),
                    row.place_id,
                    Screen::Placeholder,
                );
                return;
            }
        }
        canvas.painter = original_painter;

        let track = Rect::from_min_size(
            Pos2::new(viewport.max.x - 8.0, viewport.min.y),
            Vec2::new(5.0, viewport.height()),
        );
        screens::drag_scrollbar(
            canvas,
            "friends:scrollbar",
            track,
            content_height,
            viewport.height(),
            &mut self.friends_scroll,
            &mut self.scroll_drag,
        );
    }

    fn show_robux_tab(&mut self, canvas: &mut Canvas) {
        let state = self.session.as_ref().map(|session| match &session.robux {
            Load::Ready(balance) => (Some(*balance), None),
            Load::Failed(error) => (None, Some(error.clone())),
            Load::Loading | Load::Idle => (None, None),
        });

        let Some((balance, error)) = state else {
            self.signed_out_tab(canvas, "Robux", "Sign in to view your Robux balance.");
            return;
        };

        screens::section_label(canvas, 100.0, 252.0, "Robux");
        canvas.text(
            Pos2::new(104.0, 322.0),
            Align2::LEFT_CENTER,
            "MY BALANCE",
            20.0,
            theme::TEXT_MUTED,
            false,
        );

        if let Some(balance) = balance {
            canvas.text(
                Pos2::new(104.0, 412.0),
                Align2::LEFT_CENTER,
                format!("R$ {}", util::format_number(balance)),
                72.0,
                theme::TEXT_PRIMARY,
                true,
            );
            canvas.text(
                Pos2::new(104.0, 500.0),
                Align2::LEFT_CENTER,
                "Purchases and trades are not available in this build.",
                18.0,
                theme::TEXT_MUTED,
                false,
            );
        } else if let Some(error) = error {
            canvas.text(
                Pos2::new(104.0, 412.0),
                Align2::LEFT_CENTER,
                error,
                20.0,
                theme::TEXT_SECONDARY,
                false,
            );
            if screens::button(
                canvas,
                "placeholder:robux-retry",
                Rect::from_min_size(Pos2::new(104.0, 470.0), Vec2::new(180.0, 48.0)),
                "Retry",
                ButtonKind::Secondary,
                true,
            ) {
                self.load_account_data();
            }
        } else {
            canvas.text(
                Pos2::new(104.0, 412.0),
                Align2::LEFT_CENTER,
                "Loading balance...",
                24.0,
                theme::TEXT_SECONDARY,
                false,
            );
        }
    }

    fn signed_out_tab(&mut self, canvas: &mut Canvas, title: &str, message: &str) {
        canvas.text(
            Pos2::new(104.0, 300.0),
            Align2::LEFT_CENTER,
            title,
            40.0,
            theme::TEXT_PRIMARY,
            true,
        );
        canvas.text(
            Pos2::new(104.0, 360.0),
            Align2::LEFT_TOP,
            message,
            22.0,
            theme::TEXT_SECONDARY,
            false,
        );
        if let Some(error) = &self.auth_error {
            canvas.text(
                Pos2::new(104.0, 420.0),
                Align2::LEFT_TOP,
                error,
                16.0,
                theme::NEGATIVE,
                false,
            );
        }

        if screens::button(
            canvas,
            "placeholder:signin",
            Rect::from_min_size(Pos2::new(104.0, 480.0), Vec2::new(220.0, 56.0)),
            "Sign in",
            ButtonKind::Primary,
            true,
        ) {
            self.start_webview_login();
        }

        if self.paths.sober_cookie_file().is_file()
            && screens::button(
                canvas,
                "placeholder:sober",
                Rect::from_min_size(Pos2::new(344.0, 480.0), Vec2::new(260.0, 56.0)),
                "Use Sober session",
                ButtonKind::Ghost,
                true,
            )
        {
            self.start_sober_import();
        }
    }
}

fn friend_row(
    friend: &Friend,
    presence: &std::collections::HashMap<u64, Presence>,
    headshots: &std::collections::HashMap<u64, String>,
) -> FriendRow {
    let presence = presence.get(&friend.id).cloned().unwrap_or_default();
    FriendRow {
        user_id: friend.id,
        name: if friend.display().is_empty() {
            format!("User {}", friend.id)
        } else {
            friend.display().to_owned()
        },
        username: friend.name.clone(),
        presence: presence.label().to_owned(),
        online: presence.is_online(),
        place_id: presence.place_id,
        headshot_url: headshots.get(&friend.id).cloned(),
    }
}

fn draw_avatar_box(canvas: &Canvas, rect: Rect) {
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

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let h = h.rem_euclid(1.0) * 6.0;
    let i = h.floor();
    let f = h - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match i as i32 % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

fn nearest_skin_tone(r: u8, g: u8, b: u8) -> (u64, &'static str) {
    SKIN_TONES
        .iter()
        .min_by_key(|(_, _, hex)| {
            let dr = r as i32 - ((hex >> 16) & 0xff) as i32;
            let dg = g as i32 - ((hex >> 8) & 0xff) as i32;
            let db = b as i32 - (hex & 0xff) as i32;
            dr * dr + dg * dg + db * db
        })
        .map(|(id, name, _)| (*id, *name))
        .unwrap_or((101, "Medium red"))
}

fn skin_color_for_part(colors: &BodyColors, part: SkinPart) -> u64 {
    match part {
        SkinPart::All | SkinPart::Head => colors.head,
        SkinPart::Torso => colors.torso,
        SkinPart::LeftArm => colors.left_arm,
        SkinPart::RightArm => colors.right_arm,
        SkinPart::LeftLeg => colors.left_leg,
        SkinPart::RightLeg => colors.right_leg,
    }
}
