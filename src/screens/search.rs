use std::time::Instant;

use egui::{Color32, Pos2, Rect, Stroke, TextureHandle, Vec2};

use crate::app::App;
use crate::canvas::Canvas;
use crate::models::Load;
use crate::screens::{self, ButtonKind, NavTab, Screen};
use crate::theme;

const RESULT_W: f32 = 250.0;
const RESULT_H: f32 = 250.0;
const RESULT_GAP: f32 = 14.0;
const RESULT_X: f32 = 104.0;
const RESULT_Y: f32 = 430.0;

impl App {
    pub fn show_search(
        &mut self,
        canvas: &mut Canvas,
        background: &TextureHandle,
        wordmark: Option<&TextureHandle>,
    ) {
        let dim = screens::adaptive_dim(canvas, 24);
        screens::chrome(canvas, background, dim);

        if let Some(tab) = screens::nav(canvas, wordmark, NavTab::Games) {
            self.go_tab(tab);
            return;
        }

        canvas.text(
            Pos2::new(104.0, 252.0),
            egui::Align2::LEFT_CENTER,
            "Search experiences",
            36.0,
            theme::TEXT_PRIMARY,
            true,
        );

        let field_rect = Rect::from_min_size(Pos2::new(104.0, 306.0), Vec2::new(900.0, 62.0));
        canvas.register("search:field", field_rect, true);
        let focused = self.search.editing || canvas.is_focused("search:field");
        canvas.rect_filled(field_rect, 0, theme::panel(230));
        canvas.rect_stroke(
            field_rect,
            0,
            if focused {
                Stroke::new(2.0, theme::ACCENT)
            } else {
                Stroke::new(1.0, Color32::from_white_alpha(28))
            },
        );

        let text_rect = canvas.screen_rect(field_rect);
        let response = canvas.ui.put(
            text_rect,
            egui::TextEdit::singleline(&mut self.search.query)
                .frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(18, 15)))
                .hint_text("Search experiences")
                .font(egui::FontId::new(
                    24.0 * canvas.scale,
                    egui::FontFamily::Proportional,
                ))
                .text_color(theme::TEXT_PRIMARY)
                .align(egui::Align2::LEFT_CENTER)
                .min_size(text_rect.size()),
        );

        if response.clicked() {
            self.search.editing = true;
            self.search.focus_requested = true;
            canvas.focus.set("search:field");
        }
        if self.search.focus_requested {
            response.request_focus();
            self.search.focus_requested = false;
            self.search.editing = true;
        }
        if !self.search.editing && response.has_focus() {
            canvas
                .ui
                .memory_mut(|memory| memory.surrender_focus(response.id));
        }
        if response.changed() {
            self.search.last_edit = Instant::now();
            self.search.last_query = self.search.query.clone();
        }

        canvas.text(
            Pos2::new(104.0, 392.0),
            egui::Align2::LEFT_CENTER,
            if self.search.editing {
                "Type to search. Enter to search. Esc to stop editing."
            } else {
                "Press A or click the field to type. Y returns to the field."
            },
            16.0,
            theme::TEXT_MUTED,
            false,
        );

        self.show_results(canvas);
    }

    fn show_results(&mut self, canvas: &mut Canvas) {
        let grid = Rect::from_min_size(Pos2::new(RESULT_X, RESULT_Y), Vec2::new(1728.0, 470.0));

        let games = match &self.search.results {
            Load::Ready(games) => games.iter().take(8).cloned().collect::<Vec<_>>(),
            Load::Loading => {
                self.draw_result_skeletons(canvas);
                return;
            }
            Load::Idle => {
                canvas.text(
                    Pos2::new(RESULT_X, RESULT_Y + 24.0),
                    egui::Align2::LEFT_TOP,
                    "Search Roblox experiences by name.",
                    20.0,
                    theme::TEXT_MUTED,
                    false,
                );
                return;
            }
            Load::Failed(message) => {
                if screens::error_panel(canvas, grid, message, None, "search:retry") {
                    self.submit_search();
                }
                return;
            }
        };

        if games.is_empty() {
            let rect = Rect::from_min_size(Pos2::new(RESULT_X, RESULT_Y), Vec2::new(1728.0, 220.0));
            screens::panel(canvas, rect);
            canvas.text_centered(
                Rect::from_min_size(
                    Pos2::new(RESULT_X, RESULT_Y + 48.0),
                    Vec2::new(1696.0, 40.0),
                ),
                format!("No experiences found for \"{}\".", self.search.query.trim()),
                26.0,
                theme::TEXT_PRIMARY,
                false,
            );
            let clear = Rect::from_center_size(
                Pos2::new(rect.center().x, rect.min.y + 140.0),
                Vec2::new(180.0, 48.0),
            );
            if screens::button(
                canvas,
                "search:clear-empty",
                clear,
                "Clear search",
                ButtonKind::Secondary,
                true,
            ) {
                self.search.query.clear();
                self.search.last_query.clear();
                self.search.last_submitted.clear();
                self.search.results = Load::Idle;
                self.search.editing = true;
                self.search.focus_requested = true;
            }
            return;
        }

        self.ensure_thumbnails_for(&games);

        let mut activate = None;
        for (index, game) in games.iter().enumerate() {
            let column = index % 5;
            let row = index / 5;
            let rect = Rect::from_min_size(
                Pos2::new(
                    RESULT_X + column as f32 * (RESULT_W + RESULT_GAP),
                    RESULT_Y + row as f32 * (RESULT_H + RESULT_GAP),
                ),
                Vec2::new(RESULT_W, RESULT_H),
            );
            let id = format!("search:result:{index}");
            if screens::game_card(canvas, &id, rect, game, self.thumbnail(game.universe_id)) {
                activate = Some((game.universe_id, game.root_place_id));
            }
        }

        let total = match &self.search.results {
            Load::Ready(games) => games.len(),
            _ => games.len(),
        };
        canvas.text(
            Pos2::new(RESULT_X, 966.0),
            egui::Align2::LEFT_CENTER,
            if total > games.len() {
                format!("Showing {} of {} results", games.len(), total)
            } else {
                format!("{} results", games.len())
            },
            16.0,
            theme::TEXT_MUTED,
            false,
        );

        if let Some((universe_id, root_place_id)) = activate {
            self.open_details(universe_id, root_place_id, Screen::Search);
        }
    }

    fn draw_result_skeletons(&self, canvas: &mut Canvas) {
        for index in 0..8 {
            let column = index % 5;
            let row = index / 5;
            let rect = Rect::from_min_size(
                Pos2::new(
                    RESULT_X + column as f32 * (RESULT_W + RESULT_GAP),
                    RESULT_Y + row as f32 * (RESULT_H + RESULT_GAP),
                ),
                Vec2::new(RESULT_W, RESULT_H),
            );
            screens::skeleton_card(canvas, rect);
        }
        canvas.text(
            Pos2::new(RESULT_X, RESULT_Y + 16.0),
            egui::Align2::LEFT_TOP,
            "Searching...",
            16.0,
            theme::TEXT_MUTED,
            false,
        );
    }
}
