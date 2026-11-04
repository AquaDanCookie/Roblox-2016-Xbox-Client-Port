use egui::{Pos2, Rect, Sense, TextureHandle, Vec2};

use crate::app::App;
use crate::audio::Sfx;
use crate::canvas::Canvas;
use crate::models::{GameSummary, Load, SortInfo};
use crate::screens::{self, NavTab, Screen};
use crate::theme;

const COL_START: f32 = 96.0;
const CARD_W: f32 = 272.0;
const CARD_H: f32 = 272.0;
const CARD_GAP: f32 = 10.0;
const COL_W: f32 = CARD_W * 2.0 + CARD_GAP;
const COL_GAP: f32 = 30.0;
const LABEL_Y: f32 = 252.0;
const GRID_TOP: f32 = 288.0;
const ROW_STEP: f32 = CARD_H + CARD_GAP;
const OVERFLOW_W: f32 = 132.0;
const OVERFLOW_H: f32 = 44.0;
pub const COLUMN_STEP: f32 = COL_W + COL_GAP;
pub const COLUMN_WIDTH: f32 = COL_W;
pub const VIEWPORT_WIDTH: f32 = theme::CANVAS_W - COL_START * 2.0;

enum RowState {
    Ready(Vec<GameSummary>),
    Loading,
    Failed(String),
}

impl App {
    pub fn show_games(
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

        let favorites_rect = Rect::from_min_size(Pos2::new(1330.0, 170.0), Vec2::new(240.0, 44.0));
        let favorites = canvas.focusable("games:favorites", favorites_rect, Sense::click());
        let favorites_color = if favorites.focused {
            theme::FOCUS
        } else {
            theme::TEXT_SECONDARY
        };
        let star_center = Pos2::new(favorites_rect.min.x + 24.0, favorites_rect.center().y);
        let mut star = Vec::new();
        for index in 0..10 {
            let radius = if index % 2 == 0 { 11.0 } else { 5.0 };
            let angle = -std::f32::consts::FRAC_PI_2 + index as f32 * std::f32::consts::PI / 5.0;
            star.push(canvas.pos(
                star_center.x + angle.cos() * radius,
                star_center.y + angle.sin() * radius,
            ));
        }
        canvas.painter.add(egui::Shape::closed_line(
            star,
            egui::Stroke::new(1.6, favorites_color),
        ));
        canvas.text(
            Pos2::new(favorites_rect.min.x + 48.0, favorites_rect.center().y),
            egui::Align2::LEFT_CENTER,
            "FAVORITES",
            20.0,
            favorites_color,
            false,
        );
        if favorites.focused {
            canvas.line(
                Pos2::new(favorites_rect.min.x + 2.0, favorites_rect.max.y),
                Pos2::new(favorites_rect.max.x, favorites_rect.max.y),
                egui::Stroke::new(2.0, theme::FOCUS),
            );
        }
        if favorites.activated {
            self.open_favorites();
            return;
        }

        let search_rect = Rect::from_min_size(Pos2::new(1600.0, 170.0), Vec2::new(220.0, 44.0));
        let search = canvas.focusable("games:search", search_rect, Sense::click());
        let search_color = if search.focused {
            theme::FOCUS
        } else {
            theme::TEXT_SECONDARY
        };
        let lens = Pos2::new(search_rect.min.x + 22.0, search_rect.center().y);
        canvas.circle_stroke(lens, 9.0, egui::Stroke::new(2.0, search_color));
        canvas.line(
            Pos2::new(lens.x + 7.0, lens.y + 7.0),
            Pos2::new(lens.x + 15.0, lens.y + 15.0),
            egui::Stroke::new(2.0, search_color),
        );
        canvas.text(
            Pos2::new(search_rect.min.x + 54.0, search_rect.center().y),
            egui::Align2::LEFT_CENTER,
            "SEARCH",
            22.0,
            search_color,
            false,
        );
        if search.focused {
            canvas.line(
                Pos2::new(search_rect.min.x + 2.0, search_rect.max.y),
                Pos2::new(search_rect.max.x, search_rect.max.y),
                egui::Stroke::new(2.0, theme::FOCUS),
            );
        }
        if search.activated {
            self.open_search();
            return;
        }

        let sorts = match &self.catalog.sorts {
            Load::Ready(sorts) => sorts.clone(),
            Load::Loading | Load::Idle => {
                self.draw_games_skeletons(canvas, 3);
                return;
            }
            Load::Failed(message) => {
                let rect =
                    Rect::from_min_size(Pos2::new(COL_START, GRID_TOP), Vec2::new(1728.0, 400.0));
                if screens::error_panel(canvas, rect, message, None, "games:retry-sorts") {
                    self.retry_sorts();
                }
                return;
            }
        };

        let rows: Vec<SortInfo> = sorts;
        if rows.is_empty() {
            let rect =
                Rect::from_min_size(Pos2::new(COL_START, GRID_TOP), Vec2::new(1728.0, 300.0));
            screens::panel(canvas, rect);
            canvas.text_centered(
                rect,
                "No game rows are available right now.",
                22.0,
                theme::TEXT_SECONDARY,
                false,
            );
            return;
        }

        let row_states: Vec<RowState> = rows
            .iter()
            .map(|sort| match self.catalog.sort_games.get(&sort.sort_id) {
                Some(Load::Ready(games)) => RowState::Ready(games.clone()),
                Some(Load::Failed(message)) => RowState::Failed(message.clone()),
                _ => RowState::Loading,
            })
            .collect();

        let viewport_w = theme::CANVAS_W - COL_START * 2.0;
        let total_w = rows.len() as f32 * COLUMN_STEP - COL_GAP;
        let max_scroll = (total_w - viewport_w).max(0.0);
        let (scroll_x, scroll_y) = canvas
            .ui
            .input(|input| (input.smooth_scroll_delta.x, input.smooth_scroll_delta.y));
        let wheel = if scroll_x.abs() > scroll_y.abs() {
            scroll_x
        } else {
            scroll_y
        };
        self.games.scroll_target = self.games.scroll_target.clamp(0.0, max_scroll);
        let dt = canvas.ui.input(|input| input.stable_dt).clamp(0.0, 0.05);
        self.games.wheel_cooldown = (self.games.wheel_cooldown - dt).max(0.0);
        if wheel != 0.0 {
            self.games.scroll_target = (self.games.scroll_target
                - wheel * 0.8 * self.config.game_scroll_speed.clamp(0.25, 2.0))
            .clamp(0.0, max_scroll);
        }
        let follow = (1.0 - (-20.0 * dt).exp()).clamp(0.0, 1.0);
        self.games.scroll += (self.games.scroll_target - self.games.scroll) * follow;
        if (self.games.scroll_target - self.games.scroll).abs() < 0.05 {
            self.games.scroll = self.games.scroll_target;
        }
        let pointer_moved = canvas.ui.input(|input| input.pointer.delta() != Vec2::ZERO);

        let mut visible_games: Vec<GameSummary> = Vec::new();
        for state in &row_states {
            if let RowState::Ready(games) = state {
                visible_games.extend(games.iter().take(4).cloned());
            }
        }
        self.ensure_thumbnails_for(&visible_games);

        let mut activate = None;
        let mut browse_target = None;
        for (column, sort) in rows.iter().enumerate() {
            let x = COL_START + column as f32 * COLUMN_STEP - self.games.scroll;
            let label = if sort.display_name.is_empty() {
                "More Games".to_owned()
            } else {
                sort.display_name.clone()
            };
            screens::section_label(canvas, x, LABEL_Y, &label);
            let column_rect = Rect::from_min_size(
                Pos2::new(x, GRID_TOP),
                Vec2::new(COL_W, CARD_H * 2.0 + CARD_GAP),
            );

            match &row_states[column] {
                RowState::Ready(games) => {
                    if games.is_empty() {
                        screens::panel(canvas, column_rect);
                        canvas.text_centered(
                            column_rect,
                            "No experiences in this row right now.",
                            20.0,
                            theme::TEXT_SECONDARY,
                            false,
                        );
                        continue;
                    }

                    for index in 0..games.len().min(4) {
                        let game = &games[index];
                        let card_column = index % 2;
                        let card_row = index / 2;
                        let rect = Rect::from_min_size(
                            Pos2::new(
                                x + card_column as f32 * (CARD_W + CARD_GAP),
                                GRID_TOP + card_row as f32 * ROW_STEP,
                            ),
                            Vec2::new(CARD_W, CARD_H),
                        );
                        let id = format!("games:{column}:{index}");
                        if screens::game_card(
                            canvas,
                            &id,
                            rect,
                            game,
                            self.thumbnail(game.universe_id),
                        ) {
                            activate = Some((game.universe_id, game.root_place_id));
                        }
                    }

                    let overflow_rect = Rect::from_min_size(
                        Pos2::new(
                            x + COL_W - OVERFLOW_W,
                            GRID_TOP + CARD_H * 2.0 + CARD_GAP + 10.0,
                        ),
                        Vec2::new(OVERFLOW_W, OVERFLOW_H),
                    );
                    if screens::ellipsis_button(
                        canvas,
                        &format!("games:more:{column}"),
                        overflow_rect,
                    ) {
                        if matches!(canvas.sfx.last(), Some(Sfx::Select)) {
                            canvas.sfx.pop();
                        }
                        browse_target = Some((sort.sort_id.clone(), label.clone()));
                    }
                }
                RowState::Loading => {
                    for index in 0..4 {
                        let card_column = index % 2;
                        let card_row = index / 2;
                        let rect = Rect::from_min_size(
                            Pos2::new(
                                x + card_column as f32 * (CARD_W + CARD_GAP),
                                GRID_TOP + card_row as f32 * ROW_STEP,
                            ),
                            Vec2::new(CARD_W, CARD_H),
                        );
                        screens::skeleton_card(canvas, rect);
                    }
                }
                RowState::Failed(message) => {
                    if screens::error_panel(
                        canvas,
                        column_rect,
                        message,
                        None,
                        &format!("games:retry:{column}"),
                    ) {
                        self.retry_sort_content(&sort.sort_id);
                    }
                }
            }
        }

        if max_scroll > 0.0 {
            let track = Rect::from_min_size(Pos2::new(96.0, 1028.0), Vec2::new(1728.0, 5.0));
            screens::drag_horizontal_scrollbar(
                canvas,
                "games:scrollbar",
                track,
                total_w,
                viewport_w,
                &mut self.games.scroll_target,
                &mut self.scroll_drag,
            );
        }

        let focus_changed = self.games.last_focus.as_deref() != canvas.focus.current.as_deref();
        if focus_changed {
            if self.scroll_drag.is_none()
                && !pointer_moved
                && let Some(id) = canvas.focus.current.as_deref()
                && let Some(rest) = id.strip_prefix("games:")
                && let Some(column) = rest
                    .split(':')
                    .next()
                    .and_then(|value| value.parse::<usize>().ok())
            {
                let column_x = COL_START + column as f32 * COLUMN_STEP;
                let screen_x = column_x - self.games.scroll;
                let mut target = self.games.scroll_target;
                if screen_x < COL_START {
                    target = column_x - COL_START;
                } else if screen_x + COL_W > COL_START + viewport_w {
                    target = column_x + COL_W - COL_START - viewport_w;
                }
                self.games.scroll_target = target.clamp(0.0, max_scroll);
            }
            self.games.last_focus = canvas.focus.current.clone();
        }

        if let Some((sort_id, title)) = browse_target {
            self.open_browse(sort_id, title, Screen::Games);
            return;
        }
        if let Some((universe_id, root_place_id)) = activate {
            self.open_details(universe_id, root_place_id, Screen::Games);
        }
    }

    fn draw_games_skeletons(&self, canvas: &mut Canvas, columns: usize) {
        screens::section_label(canvas, COL_START, LABEL_Y, "Loading catalog");
        for column in 0..columns {
            let x = COL_START + column as f32 * (COL_W + COL_GAP);
            for index in 0..4 {
                let card_column = index % 2;
                let card_row = index / 2;
                let rect = Rect::from_min_size(
                    Pos2::new(
                        x + card_column as f32 * (CARD_W + CARD_GAP),
                        GRID_TOP + card_row as f32 * ROW_STEP,
                    ),
                    Vec2::new(CARD_W, CARD_H),
                );
                screens::skeleton_card(canvas, rect);
            }
        }
    }
}
