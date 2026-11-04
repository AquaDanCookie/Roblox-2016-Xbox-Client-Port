use egui::{Align2, Color32, Pos2, Rect, Stroke, TextureHandle, Vec2};

use crate::app::App;
use crate::audio::Sfx;
use crate::canvas::Canvas;
use crate::models::Load;
use crate::screens::{self, ButtonKind, Screen};
use crate::theme;
use crate::util;

impl App {
    pub fn show_details(&mut self, canvas: &mut Canvas, background: &TextureHandle) {
        let dim = screens::adaptive_dim(canvas, 28);
        screens::chrome(canvas, background, dim);

        if self.details.focus_play && self.details.details.ready().is_some() {
            self.details.focus_play = false;
            canvas.focus.set("details:play");
        }

        let back_label = match self.details.from {
            Screen::Home => "Home",
            Screen::Games => "Games",
            Screen::Favorites => "Favorites",
            Screen::Search => "Search",
            Screen::Browse => "Browse",
            _ => "Back",
        };
        if screens::back_button(canvas, "details:back", back_label) {
            if matches!(canvas.sfx.last(), Some(Sfx::Select)) {
                canvas.sfx.pop();
            }
            self.music
                .play_sfx(Sfx::SlideOut, &self.paths, &self.config);
            self.goto(self.details.from);
            return;
        }

        let bundle = match &self.details.details {
            Load::Ready(bundle) => bundle.clone(),
            Load::Loading | Load::Idle => {
                self.draw_loading(canvas);
                return;
            }
            Load::Failed(message) => {
                let rect = Rect::from_min_size(Pos2::new(150.0, 240.0), Vec2::new(1620.0, 520.0));
                if screens::error_panel(canvas, rect, message, None, "details:retry") {
                    self.retry_details();
                }
                return;
            }
        };

        let mut urls = Vec::new();
        if let Some(url) = &bundle.thumbnail_url {
            urls.push(url.clone());
        }
        for item in &bundle.media {
            if let Some(url) = &item.url {
                urls.push(url.clone());
            }
        }
        for url in &urls {
            self.images.ensure(url);
        }

        let thumbnail = bundle
            .thumbnail_url
            .as_ref()
            .and_then(|url| self.images.get(url))
            .cloned();
        let media: Vec<Option<TextureHandle>> = bundle
            .media
            .iter()
            .take(3)
            .map(|item| {
                item.url
                    .as_ref()
                    .and_then(|url| self.images.get(url))
                    .cloned()
            })
            .collect();

        let title = if bundle.details.name.is_empty() {
            "Experience".to_owned()
        } else {
            bundle.details.name.clone()
        };
        let creator = bundle.details.creator.name.clone();
        let title =
            util::truncate_to_width(&title, 950.0, |text| canvas.text_width(text, 50.0, true));
        canvas.text_with_shadow(
            Pos2::new(156.0, 176.0),
            Align2::LEFT_CENTER,
            &title,
            50.0,
            theme::TEXT_PRIMARY,
            true,
        );
        if !creator.is_empty() {
            let genre = bundle.details.genre_l1.trim();
            let subtitle = if genre.is_empty() {
                format!("By {creator}")
            } else {
                format!("By {creator} · {genre}")
            };
            let subtitle = util::truncate_to_width(&subtitle, 1000.0, |text| {
                canvas.text_width(text, 22.0, false)
            });
            canvas.text_with_shadow(
                Pos2::new(156.0, 226.0),
                Align2::LEFT_CENTER,
                subtitle,
                22.0,
                theme::TEXT_MUTED,
                false,
            );
        }

        let media_start = usize::from(bundle.thumbnail_url.is_some());
        let mut lightbox_index = None;
        let thumb_rect = Rect::from_min_size(Pos2::new(156.0, 278.0), Vec2::new(540.0, 540.0));
        if draw_image(
            canvas,
            "details:image:thumbnail",
            thumb_rect,
            thumbnail.as_ref(),
            "Experience thumbnail",
            bundle.thumbnail_url.is_some(),
        ) {
            lightbox_index = Some(0);
        }

        screens::section_label(canvas, 753.0, 252.0, "Rating and Description");
        let votes = bundle.votes;
        if let Some(votes) = votes {
            screens::rating_summary(
                canvas,
                753.0,
                332.0,
                430.0,
                votes.up_votes,
                votes.down_votes,
            );
        } else {
            canvas.text(
                Pos2::new(753.0, 332.0),
                Align2::LEFT_CENTER,
                "Rating unavailable",
                18.0,
                theme::TEXT_MUTED,
                false,
            );
        }

        let description = util::strip_html(&bundle.details.description);
        let desc_rect = Rect::from_min_size(Pos2::new(753.0, 400.0), Vec2::new(430.0, 380.0));
        if description.trim().is_empty() {
            canvas.text(
                Pos2::new(753.0, 410.0),
                Align2::LEFT_TOP,
                "No description was provided.",
                24.0,
                theme::TEXT_MUTED,
                false,
            );
        } else {
            let galley = canvas.wrapped(
                desc_rect,
                description.trim(),
                24.0,
                theme::TEXT_SECONDARY,
                false,
            );
            let content_height = galley.size().y / canvas.scale;
            let max_scroll = (content_height - desc_rect.height()).max(0.0);
            let wheel = canvas.ui.input(|input| input.smooth_scroll_delta.y);
            if wheel != 0.0 {
                self.details.scroll = (self.details.scroll - wheel).clamp(0.0, max_scroll);
            }
            self.details.scroll = self.details.scroll.clamp(0.0, max_scroll);
            let text_pos = Pos2::new(desc_rect.min.x, desc_rect.min.y - self.details.scroll);
            // The galley carries its own text colour; drawing a second,
            // offset copy with a different fallback colour produced a ghosted
            // double image instead of a shadow.
            canvas.galley_clipped(text_pos, galley, theme::TEXT_SECONDARY, desc_rect);

            let track = Rect::from_min_size(
                Pos2::new(desc_rect.max.x + 8.0, desc_rect.min.y),
                Vec2::new(5.0, desc_rect.height()),
            );
            screens::drag_scrollbar(
                canvas,
                "details:description-scrollbar",
                track,
                content_height,
                desc_rect.height(),
                &mut self.details.scroll,
                &mut self.scroll_drag,
            );
        }

        let info_y = 802.0;
        canvas.text_with_shadow(
            Pos2::new(753.0, info_y),
            Align2::LEFT_CENTER,
            format!(
                "{} visits   {} playing   {} favorites",
                util::format_count(bundle.details.visits),
                util::format_count(bundle.details.playing),
                util::format_count(bundle.details.favorited_count)
            ),
            17.0,
            theme::TEXT_MUTED,
            false,
        );
        canvas.text_with_shadow(
            Pos2::new(753.0, info_y + 24.0),
            Align2::LEFT_CENTER,
            format!(
                "Up to {} players   Updated {}",
                bundle.details.max_players,
                util::format_date(&bundle.details.updated)
            ),
            17.0,
            theme::TEXT_MUTED,
            false,
        );
        if let Some(warning) = bundle.warnings.first() {
            canvas.text(
                Pos2::new(753.0, info_y + 52.0),
                Align2::LEFT_CENTER,
                warning,
                14.0,
                theme::TEXT_MUTED,
                false,
            );
        }

        screens::section_label(canvas, 1233.0, 252.0, "Game Images");
        let label_for = |index: usize| {
            bundle
                .media
                .get(index)
                .map(|item| item.alt_text.trim())
                .filter(|text| !text.is_empty())
                .unwrap_or("Game image")
        };
        let large = Rect::from_min_size(Pos2::new(1233.0, 282.0), Vec2::new(432.0, 243.0));
        if draw_image(
            canvas,
            "details:image:large",
            large,
            media.first().and_then(|texture| texture.as_ref()),
            label_for(0),
            urls.len() > media_start,
        ) {
            lightbox_index = Some(media_start);
        }
        let small_a = Rect::from_min_size(Pos2::new(1233.0, 540.0), Vec2::new(210.0, 118.0));
        if draw_image(
            canvas,
            "details:image:small-a",
            small_a,
            media.get(1).and_then(|texture| texture.as_ref()),
            label_for(1),
            urls.len() > media_start + 1,
        ) {
            lightbox_index = Some(media_start + 1);
        }
        let small_b = Rect::from_min_size(Pos2::new(1455.0, 540.0), Vec2::new(210.0, 118.0));
        if draw_image(
            canvas,
            "details:image:small-b",
            small_b,
            media.get(2).and_then(|texture| texture.as_ref()),
            label_for(2),
            urls.len() > media_start + 2,
        ) {
            lightbox_index = Some(media_start + 2);
        }
        if let Some(index) = lightbox_index {
            self.open_lightbox(urls.clone(), index);
        }

        let play = Rect::from_min_size(Pos2::new(156.0, 952.0), Vec2::new(232.0, 80.0));
        let favorite = Rect::from_min_size(Pos2::new(400.0, 952.0), Vec2::new(220.0, 80.0));

        let play_clicked =
            screens::button(canvas, "details:play", play, "Play", ButtonKind::Play, true);
        let favorite_state = self.favorite_states.get(&bundle.details.id).copied();
        let favorite_busy = self.favorite_pending.contains(&bundle.details.id);
        if screens::button(
            canvas,
            "details:favorite",
            favorite,
            if favorite_state == Some(true) {
                "Favorited"
            } else {
                "Favorite"
            },
            ButtonKind::Secondary,
            !favorite_busy,
        ) {
            self.toggle_favorite(bundle.details.id);
        }

        if play_clicked {
            // The launch swoosh is the confirm sound for PLAY. Avoid stacking
            // the generic select click underneath it.
            if matches!(canvas.sfx.last(), Some(Sfx::Select)) {
                canvas.sfx.pop();
            }
            self.open_launch(title, bundle.details.root_place_id, Screen::Details);
        }
    }

    fn draw_loading(&self, canvas: &mut Canvas) {
        let thumb = Rect::from_min_size(Pos2::new(156.0, 278.0), Vec2::new(540.0, 540.0));
        screens::skeleton_card(canvas, thumb);
        for (index, width) in [430.0, 300.0, 360.0].iter().enumerate() {
            let rect = Rect::from_min_size(
                Pos2::new(753.0, 252.0 + index as f32 * 42.0),
                Vec2::new(*width, 20.0),
            );
            screens::skeleton_card(canvas, rect);
        }
        let large = Rect::from_min_size(Pos2::new(1233.0, 282.0), Vec2::new(432.0, 243.0));
        screens::skeleton_card(canvas, large);
        canvas.text_centered(
            Rect::from_min_size(Pos2::new(156.0, 850.0), Vec2::new(540.0, 40.0)),
            "Loading experience...",
            22.0,
            theme::TEXT_SECONDARY,
            false,
        );
    }
}

fn draw_image(
    canvas: &mut Canvas,
    id: &str,
    rect: Rect,
    texture: Option<&TextureHandle>,
    label: &str,
    clickable: bool,
) -> bool {
    let response = if clickable {
        Some(canvas.focusable(id, rect, egui::Sense::click()))
    } else {
        None
    };
    match texture {
        Some(texture) => canvas.image_cover(texture, rect, Color32::WHITE),
        None => {
            canvas.rect_filled(rect, 0, theme::PANEL_SOLID);
            canvas.text_centered(rect, label, 16.0, theme::TEXT_MUTED, false);
        }
    }
    canvas.rect_stroke(
        rect,
        0,
        if response.as_ref().is_some_and(|response| response.focused) {
            Stroke::new(2.0, theme::TEXT_PRIMARY)
        } else {
            Stroke::new(1.0, Color32::from_white_alpha(24))
        },
    );
    response.is_some_and(|response| response.activated)
}

impl App {
    pub fn draw_lightbox(&mut self, ui: &mut egui::Ui) {
        let Some(lightbox) = self.details.lightbox.as_ref() else {
            return;
        };
        let urls = lightbox.urls.clone();
        let index = lightbox.index.min(urls.len().saturating_sub(1));
        let previous_index = lightbox.previous_index;
        let slide_started = lightbox.slide_started;
        let slide_forward = lightbox.slide_forward;
        let url = urls.get(index).cloned();
        let previous_url = previous_index.and_then(|index| urls.get(index)).cloned();
        if let Some(url) = &url {
            self.images.ensure(url);
        }
        if let Some(url) = &previous_url {
            self.images.ensure(url);
        }
        let texture = url.as_ref().and_then(|url| self.images.get(url)).cloned();
        let previous_texture = previous_url
            .as_ref()
            .and_then(|url| self.images.get(url))
            .cloned();

        let mut canvas = Canvas::overlay(ui, crate::focus::FocusState::default(), false, true);
        canvas.rect_filled(screens::full(), 0, Color32::from_black_alpha(240));

        let image_rect = Rect::from_center_size(
            Pos2::new(theme::CANVAS_W * 0.5, theme::CANVAS_H * 0.5 - 20.0),
            Vec2::new(1500.0, 800.0),
        );
        canvas.rect_filled(image_rect, 0, theme::PANEL_SOLID);
        let progress = slide_started
            .map(|start| (start.elapsed().as_secs_f32() / 0.24).clamp(0.0, 1.0))
            .unwrap_or(1.0);
        let ease = 1.0 - (1.0 - progress).powi(3);
        let direction = if slide_forward { 1.0 } else { -1.0 };
        let current_offset = direction * 1500.0 * (1.0 - ease);
        let previous_offset = -direction * 1500.0 * ease;
        let clip = canvas.screen_rect(image_rect);
        let clipped = canvas.painter.with_clip_rect(clip);
        let original_painter = std::mem::replace(&mut canvas.painter, clipped);
        if let Some(previous_texture) = &previous_texture {
            let shifted = image_rect.translate(Vec2::new(previous_offset, 0.0));
            canvas.image_contain(previous_texture, shifted, Color32::WHITE);
        }
        let current_rect = image_rect.translate(Vec2::new(current_offset, 0.0));
        match &texture {
            Some(texture) => canvas.image_contain(texture, current_rect, Color32::WHITE),
            None => {
                canvas.text_centered(
                    current_rect,
                    "Loading image...",
                    26.0,
                    theme::TEXT_MUTED,
                    false,
                );
            }
        }
        canvas.painter = original_painter;
        canvas.rect_stroke(
            image_rect,
            0,
            Stroke::new(1.0, Color32::from_white_alpha(45)),
        );

        let close_rect = Rect::from_center_size(
            Pos2::new(image_rect.max.x - 38.0, image_rect.min.y + 38.0),
            Vec2::splat(58.0),
        );
        let close = canvas.focusable("lightbox:close", close_rect, egui::Sense::click());
        canvas.circle_filled(close_rect.center(), 27.0, Color32::from_black_alpha(150));
        canvas.circle_stroke(
            close_rect.center(),
            27.0,
            Stroke::new(
                1.5,
                if close.focused {
                    theme::FOCUS
                } else {
                    theme::TEXT_PRIMARY
                },
            ),
        );
        let x_color = if close.focused {
            theme::FOCUS
        } else {
            theme::TEXT_PRIMARY
        };
        canvas.line(
            Pos2::new(close_rect.center().x - 9.0, close_rect.center().y - 9.0),
            Pos2::new(close_rect.center().x + 9.0, close_rect.center().y + 9.0),
            Stroke::new(2.0, x_color),
        );
        canvas.line(
            Pos2::new(close_rect.center().x + 9.0, close_rect.center().y - 9.0),
            Pos2::new(close_rect.center().x - 9.0, close_rect.center().y + 9.0),
            Stroke::new(2.0, x_color),
        );

        let prev_rect = Rect::from_center_size(
            Pos2::new(image_rect.min.x + 54.0, image_rect.center().y),
            Vec2::splat(84.0),
        );
        let next_rect = Rect::from_center_size(
            Pos2::new(image_rect.max.x - 54.0, image_rect.center().y),
            Vec2::splat(84.0),
        );
        let prev = canvas.focusable("lightbox:prev", prev_rect, egui::Sense::click());
        let next = canvas.focusable("lightbox:next", next_rect, egui::Sense::click());
        for (rect, response, forward) in [(prev_rect, &prev, false), (next_rect, &next, true)] {
            if urls.len() <= 1 {
                continue;
            }
            canvas.circle_filled(rect.center(), 38.0, Color32::from_black_alpha(160));
            canvas.circle_stroke(
                rect.center(),
                38.0,
                Stroke::new(
                    1.5,
                    if response.focused {
                        theme::FOCUS
                    } else {
                        theme::TEXT_PRIMARY
                    },
                ),
            );
            let color = if response.focused {
                theme::FOCUS
            } else {
                theme::TEXT_PRIMARY
            };
            // In this coordinate space a positive x step draws a '<'
            // chevron; the right-hand control needs the negative direction.
            let direction = if forward { -1.0 } else { 1.0 };
            canvas.line(
                Pos2::new(rect.center().x + 9.0 * direction, rect.center().y - 13.0),
                Pos2::new(rect.center().x - 5.0 * direction, rect.center().y),
                Stroke::new(2.5, color),
            );
            canvas.line(
                Pos2::new(rect.center().x - 5.0 * direction, rect.center().y),
                Pos2::new(rect.center().x + 9.0 * direction, rect.center().y + 13.0),
                Stroke::new(2.5, color),
            );
        }

        canvas.text_centered(
            Rect::from_min_size(Pos2::new(0.0, 992.0), Vec2::new(theme::CANVAS_W, 36.0)),
            if urls.is_empty() {
                String::new()
            } else {
                format!("{} / {}", index + 1, urls.len())
            },
            20.0,
            theme::TEXT_SECONDARY,
            false,
        );

        if close.activated {
            self.music.play_sfx(Sfx::Select, &self.paths, &self.config);
            self.close_lightbox();
            let _ = canvas.finish();
            return;
        }
        if prev.activated {
            self.music
                .play_sfx(Sfx::Navigate, &self.paths, &self.config);
            self.lightbox_step(false);
            let _ = canvas.finish();
            return;
        }
        if next.activated {
            self.music
                .play_sfx(Sfx::Navigate, &self.paths, &self.config);
            self.lightbox_step(true);
            let _ = canvas.finish();
            return;
        }

        if canvas.ui.input(|input| input.pointer.any_click())
            && let Some(pos) = canvas.ui.input(|input| input.pointer.interact_pos())
        {
            let image = canvas.screen_rect(image_rect);
            let close = canvas.screen_rect(close_rect);
            let prev = canvas.screen_rect(prev_rect);
            let next = canvas.screen_rect(next_rect);
            if !image.contains(pos)
                && !close.contains(pos)
                && !prev.contains(pos)
                && !next.contains(pos)
            {
                self.close_lightbox();
            }
        }
        let _ = canvas.finish();
    }
}
