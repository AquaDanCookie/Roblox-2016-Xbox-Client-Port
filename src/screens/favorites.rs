use egui::{Align2, Pos2, Rect, TextureHandle, Vec2};

use crate::app::App;
use crate::canvas::Canvas;
use crate::models::Load;
use crate::screens::{self, Screen};
use crate::theme;

const GRID_X: f32 = 230.0;
const GRID_Y: f32 = 230.0;
const CARD: f32 = 230.0;
const GAP: f32 = 16.0;
const COLUMNS: usize = 6;
const MAX_CARDS: usize = 18;

impl App {
    pub fn show_favorites(&mut self, canvas: &mut Canvas, background: &TextureHandle) {
        let dim = screens::adaptive_dim(canvas, 30);
        screens::chrome(canvas, background, dim);

        if screens::back_button(canvas, "favorites:back", "Games") {
            self.goto(Screen::Games);
            return;
        }

        screens::section_label(canvas, GRID_X, 190.0, "My Favorites");

        if self.session.is_none() {
            canvas.text(
                Pos2::new(GRID_X, GRID_Y + 20.0),
                Align2::LEFT_TOP,
                "Sign in to see your favorite experiences.",
                24.0,
                theme::TEXT_SECONDARY,
                false,
            );
            return;
        }

        let games = match &self.favorites.games {
            Load::Ready(games) => games.clone(),
            Load::Loading | Load::Idle => {
                for index in 0..10 {
                    let column = index % COLUMNS;
                    let row = index / COLUMNS;
                    let rect = Rect::from_min_size(
                        Pos2::new(
                            GRID_X + column as f32 * (CARD + GAP),
                            GRID_Y + row as f32 * (CARD + GAP),
                        ),
                        Vec2::splat(CARD),
                    );
                    screens::skeleton_card(canvas, rect);
                }
                canvas.text(
                    Pos2::new(GRID_X + 8.0, GRID_Y + 16.0),
                    Align2::LEFT_TOP,
                    "Loading favorites...",
                    16.0,
                    theme::TEXT_MUTED,
                    false,
                );
                return;
            }
            Load::Failed(error) => {
                let rect = Rect::from_min_size(Pos2::new(GRID_X, GRID_Y), Vec2::new(1460.0, 300.0));
                if screens::error_panel(canvas, rect, error, None, "favorites:retry") {
                    self.favorites.requested = false;
                    self.favorites.games = Load::Idle;
                }
                return;
            }
        };

        if games.is_empty() {
            canvas.text(
                Pos2::new(GRID_X, GRID_Y + 20.0),
                Align2::LEFT_TOP,
                "You have not favorited any experiences yet.",
                24.0,
                theme::TEXT_SECONDARY,
                false,
            );
            return;
        }

        self.ensure_thumbnails_for(&games);
        let mut activate = None;
        for (index, game) in games.iter().take(MAX_CARDS).enumerate() {
            let column = index % COLUMNS;
            let row = index / COLUMNS;
            let rect = Rect::from_min_size(
                Pos2::new(
                    GRID_X + column as f32 * (CARD + GAP),
                    GRID_Y + row as f32 * (CARD + GAP),
                ),
                Vec2::splat(CARD),
            );
            if screens::game_card(
                canvas,
                &format!("favorites:card:{index}"),
                rect,
                game,
                self.thumbnail(game.universe_id),
            ) {
                activate = Some((game.universe_id, game.root_place_id));
            }
        }

        canvas.text(
            Pos2::new(GRID_X, 990.0),
            Align2::LEFT_CENTER,
            if games.len() > MAX_CARDS {
                format!("Showing {} of {} favorites", MAX_CARDS, games.len())
            } else {
                format!("{} favorites", games.len())
            },
            16.0,
            theme::TEXT_MUTED,
            false,
        );

        if let Some((universe_id, root_place_id)) = activate {
            self.open_details(universe_id, root_place_id, Screen::Favorites);
        }
    }
}
