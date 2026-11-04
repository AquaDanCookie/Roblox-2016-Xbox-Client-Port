use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, TextureHandle, Vec2};

use crate::app::App;
use crate::audio::Sfx;
use crate::canvas::Canvas;
use crate::models::{GameSummary, Load};
use crate::screens::{self, ButtonKind, Screen};
use crate::theme;
use crate::util;

const SELECTED_SIZE: f32 = 440.0;
const CARD_SIZE: f32 = 300.0;
const CARD_GAP: f32 = 16.0;
const ROW_X: f32 = 100.0;
const ROW_Y: f32 = 220.0;

impl App {
    pub fn show_browse(
        &mut self,
        canvas: &mut Canvas,
        background: &TextureHandle,
        _wordmark: Option<&TextureHandle>,
    ) {
        let dim = screens::adaptive_dim(canvas, 36);
        screens::chrome(canvas, background, dim);

        // Mouse hover must not drive selection here. Moving the cursor would
        // otherwise re-layout the sliding row under the pointer and race to
        // one end of the list.
        canvas.hover_enabled = false;

        let mut page_slide = {
            let open_t = (self.browse.opened_at.elapsed().as_secs_f32() / 0.22).clamp(0.0, 1.0);
            let open_ease = 1.0 - (1.0 - open_t).powi(3);
            (1.0 - open_ease) * 180.0
        };
        if let Some(start) = self.browse.closing_at {
            let close_t = (start.elapsed().as_secs_f32() / 0.22).clamp(0.0, 1.0);
            let close_ease = close_t * close_t * (3.0 - 2.0 * close_t);
            page_slide = close_ease * 180.0;
            canvas.input_enabled = false;
            if close_t >= 1.0 {
                self.goto(self.browse.from);
                return;
            }
        }
        canvas.translate.x = page_slide;

        let back_label = match self.browse.from {
            Screen::Home => "Home",
            _ => "Games",
        };
        if screens::back_button(canvas, "browse:back", back_label) {
            if matches!(canvas.sfx.last(), Some(Sfx::Select)) {
                canvas.sfx.pop();
            }
            self.start_browse_close();
            return;
        }

        let title = if self.browse.title.is_empty() {
            "Featured".to_owned()
        } else {
            self.browse.title.to_uppercase()
        };
        canvas.text_with_shadow(
            Pos2::new(120.0, 175.0),
            Align2::LEFT_CENTER,
            &title,
            36.0,
            theme::TEXT_PRIMARY,
            true,
        );
        let title_width = canvas.text_width(&title, 36.0, true);
        let chevron_center = Pos2::new(120.0 + title_width + 42.0, 175.0);
        let chevron_rect = Rect::from_center_size(chevron_center, Vec2::splat(48.0));
        let chevron = canvas.focusable("browse:category", chevron_rect, Sense::click());
        canvas.circle_stroke(
            chevron_center,
            14.0,
            Stroke::new(1.5, theme::TEXT_SECONDARY),
        );
        let chevron_color = if chevron.focused {
            theme::FOCUS
        } else {
            theme::TEXT_SECONDARY
        };
        canvas.line(
            Pos2::new(chevron_center.x - 5.0, chevron_center.y - 2.0),
            Pos2::new(chevron_center.x, chevron_center.y + 3.0),
            Stroke::new(2.0, chevron_color),
        );
        canvas.line(
            Pos2::new(chevron_center.x, chevron_center.y + 3.0),
            Pos2::new(chevron_center.x + 5.0, chevron_center.y - 2.0),
            Stroke::new(2.0, chevron_color),
        );
        if chevron.activated {
            self.cycle_browse_category();
            return;
        }

        let games = match self.catalog.sort_games.get(&self.browse.sort_id) {
            Some(Load::Ready(games)) => games.clone(),
            Some(Load::Failed(message)) => {
                let rect = Rect::from_min_size(Pos2::new(100.0, ROW_Y), Vec2::new(1720.0, 400.0));
                if screens::error_panel(canvas, rect, message, None, "browse:retry") {
                    let sort_id = self.browse.sort_id.clone();
                    self.retry_sort_content(&sort_id);
                }
                return;
            }
            _ => {
                self.draw_skeletons(canvas);
                return;
            }
        };

        if games.is_empty() {
            let rect = Rect::from_min_size(Pos2::new(100.0, ROW_Y), Vec2::new(1720.0, 220.0));
            screens::panel(canvas, rect);
            canvas.text_centered(
                rect,
                "No experiences in this category right now.",
                24.0,
                theme::TEXT_SECONDARY,
                false,
            );
            return;
        }

        let dt = canvas.ui.input(|input| input.stable_dt).clamp(0.0, 0.05);
        self.browse.wheel_cooldown = (self.browse.wheel_cooldown - dt).max(0.0);
        let wheel = canvas.ui.input(|input| input.smooth_scroll_delta.y);
        self.browse.wheel_accum = (self.browse.wheel_accum
            + wheel * self.config.game_scroll_speed.clamp(0.25, 2.0))
        .clamp(-160.0, 160.0);
        const WHEEL_STEP: f32 = 28.0;
        if self.browse.wheel_cooldown == 0.0 && self.browse.wheel_accum.abs() >= WHEEL_STEP {
            let length = games.len();
            let delta = if self.browse.wheel_accum > 0.0 { -1 } else { 1 };
            self.browse.selected =
                (self.browse.selected as isize + delta).clamp(0, length as isize - 1) as usize;
            self.browse.scroll_pos = self.browse.selected as f32;
            self.browse.wheel_accum = 0.0;
            self.browse.wheel_cooldown = 0.10;
            canvas
                .focus
                .set(format!("browse:card:{}", self.browse.selected));
            canvas.sfx.push(Sfx::Navigate);
        }
        let selected = self.browse.selected.min(games.len() - 1);
        self.browse.selected = selected;

        let follow = (1.0 - (-18.0 * dt).exp()).clamp(0.0, 1.0);
        if self.browse.card_rects.len() != games.len() {
            self.browse.card_rects = (0..games.len())
                .map(|index| card_rect(index, selected))
                .collect();
        }
        for (index, current) in self.browse.card_rects.iter_mut().enumerate() {
            *current = lerp_rect(*current, card_rect(index, selected), follow);
        }
        let rects = self.browse.card_rects.clone();

        self.ensure_thumbnails_for(&games);

        let mut activate = None;
        for (index, game) in games.iter().enumerate() {
            let rect = rects[index];
            if rect.max.x < 0.0 || rect.min.x > theme::CANVAS_W {
                continue;
            }
            let id = format!("browse:card:{index}");
            let response = canvas.focusable(&id, rect, Sense::click());
            draw_card_image(canvas, rect, game, self.thumbnail(game.universe_id));
            if index == selected || response.focused {
                canvas.focus_ring(rect, 0);
            }
            if response.activated {
                if index == selected {
                    activate = Some((game.universe_id, game.root_place_id));
                } else {
                    self.browse.selected = index;
                    self.browse.scroll_pos = index as f32;
                }
            }
        }

        if let Some(id) = canvas.focus.current.as_deref()
            && let Some(index) = id
                .strip_prefix("browse:card:")
                .and_then(|value| value.parse::<usize>().ok())
            && index < games.len()
        {
            self.browse.selected = index;
        }

        if games.len() > 1 {
            let mut scroll = self.browse.scroll_pos;
            screens::drag_horizontal_scrollbar(
                canvas,
                "browse:scrollbar",
                Rect::from_min_size(Pos2::new(100.0, 1016.0), Vec2::new(1720.0, 5.0)),
                games.len() as f32,
                1.0,
                &mut scroll,
                &mut self.scroll_drag,
            );
            self.browse.scroll_pos = scroll;
            let new_selected = scroll
                .round()
                .clamp(0.0, games.len().saturating_sub(1) as f32)
                as usize;
            if new_selected != self.browse.selected {
                self.browse.selected = new_selected;
                canvas.focus.set(format!("browse:card:{}", new_selected));
                canvas.sfx.push(Sfx::Navigate);
            }
        }

        let game = &games[selected];
        let favorite_universe = game.universe_id;
        self.ensure_favorite_status(favorite_universe);
        let favorite_state = self.favorite_states.get(&favorite_universe).copied();
        let favorite_busy = self.favorite_pending.contains(&favorite_universe);
        let name = util::truncate_to_width(&game.name, 900.0, |text| {
            canvas.text_width(text, 50.0, true)
        });
        canvas.text_with_shadow(
            Pos2::new(120.0, 700.0),
            Align2::LEFT_CENTER,
            name,
            50.0,
            theme::TEXT_PRIMARY,
            true,
        );
        screens::rating_summary(canvas, 120.0, 760.0, 520.0, game.up_votes, game.down_votes);
        let creator = if game.creator_name.is_empty() {
            String::new()
        } else {
            format!("By {}", game.creator_name)
        };
        if !creator.is_empty() {
            canvas.text_with_shadow(
                Pos2::new(120.0, 826.0),
                Align2::LEFT_CENTER,
                util::truncate_to_width(&creator, 520.0, |text| {
                    canvas.text_width(text, 22.0, false)
                }),
                22.0,
                theme::TEXT_MUTED,
                false,
            );
        }

        let play = Rect::from_min_size(Pos2::new(100.0, 900.0), Vec2::new(232.0, 80.0));
        let favorite = Rect::from_min_size(Pos2::new(344.0, 900.0), Vec2::new(220.0, 80.0));
        if screens::button(canvas, "browse:play", play, "Play", ButtonKind::Play, true) {
            self.open_launch(game.name.clone(), game.root_place_id, Screen::Browse);
            return;
        }
        if screens::button(
            canvas,
            "browse:favorite",
            favorite,
            if favorite_state == Some(true) {
                "Favorited"
            } else {
                "Favorite"
            },
            ButtonKind::Secondary,
            !favorite_busy,
        ) {
            self.toggle_favorite(favorite_universe);
        }

        if let Some((universe_id, root_place_id)) = activate {
            self.open_details(universe_id, root_place_id, Screen::Browse);
        }
    }

    fn cycle_browse_category(&mut self) {
        let Some(sorts) = self.catalog.sorts.ready().cloned() else {
            return;
        };
        if sorts.is_empty() {
            return;
        }
        let index = sorts
            .iter()
            .position(|sort| sort.sort_id == self.browse.sort_id)
            .unwrap_or(0);
        let next = &sorts[(index + 1) % sorts.len()];
        self.browse.sort_id = next.sort_id.clone();
        self.browse.title = if next.display_name.is_empty() {
            next.sort_id.clone()
        } else {
            next.display_name.clone()
        };
        self.browse.selected = 0;
        self.browse.scroll_pos = 0.0;
        self.ensure_sort_content(&next.sort_id);
    }

    fn draw_skeletons(&self, canvas: &mut Canvas) {
        for index in 0..5 {
            let rect = Rect::from_min_size(
                Pos2::new(ROW_X + index as f32 * (CARD_SIZE + CARD_GAP), ROW_Y),
                Vec2::splat(CARD_SIZE),
            );
            screens::skeleton_card(canvas, rect);
        }
        canvas.text(
            Pos2::new(ROW_X + 8.0, ROW_Y + 18.0),
            Align2::LEFT_TOP,
            "Loading experiences...",
            16.0,
            Color32::from_white_alpha(170),
            false,
        );
    }
}

fn card_rect(index: usize, selected: usize) -> Rect {
    let (x, size) = if index == selected {
        (ROW_X, SELECTED_SIZE)
    } else if index < selected {
        (
            ROW_X - (selected - index) as f32 * (CARD_SIZE + CARD_GAP),
            CARD_SIZE,
        )
    } else {
        (
            ROW_X
                + SELECTED_SIZE
                + CARD_GAP
                + (index - selected - 1) as f32 * (CARD_SIZE + CARD_GAP),
            CARD_SIZE,
        )
    };
    Rect::from_min_size(Pos2::new(x, ROW_Y), Vec2::splat(size))
}

fn lerp_rect(from: Rect, to: Rect, t: f32) -> Rect {
    let lerp = |a: f32, b: f32| a + (b - a) * t;
    Rect::from_min_max(
        Pos2::new(lerp(from.min.x, to.min.x), lerp(from.min.y, to.min.y)),
        Pos2::new(lerp(from.max.x, to.max.x), lerp(from.max.y, to.max.y)),
    )
}

fn draw_card_image(
    canvas: &Canvas,
    rect: Rect,
    game: &GameSummary,
    texture: Option<&TextureHandle>,
) {
    match texture {
        Some(texture) => canvas.image_cover(texture, rect, Color32::WHITE),
        None => {
            canvas.rect_filled(rect, 0, theme::PANEL_SOLID);
            canvas.text_centered(
                rect,
                util::initials(&game.name),
                42.0,
                theme::TEXT_MUTED,
                true,
            );
        }
    }
}
