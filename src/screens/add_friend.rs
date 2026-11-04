use egui::{Align2, Color32, FontId, Pos2, Rect, Stroke, TextureHandle, Vec2};

use secrecy::ExposeSecret;

use crate::app::App;
use crate::canvas::Canvas;
use crate::screens::{self, ButtonKind};
use crate::theme;

impl App {
    pub fn show_add_friend(&mut self, canvas: &mut Canvas, background: &TextureHandle) {
        screens::chrome(canvas, background, 120);

        if screens::back_button(canvas, "addfriend:back", "Back") {
            self.goto(self.add_friend.from);
            return;
        }

        let panel = Rect::from_min_size(Pos2::new(300.0, 240.0), Vec2::new(1320.0, 430.0));
        screens::panel(canvas, panel);

        canvas.text(
            Pos2::new(360.0, 300.0),
            Align2::LEFT_CENTER,
            "Add Friend",
            40.0,
            theme::TEXT_PRIMARY,
            true,
        );

        if let Some((_, name)) = &self.add_friend.target {
            canvas.text(
                Pos2::new(360.0, 360.0),
                Align2::LEFT_CENTER,
                format!("Send a friend request to {name}?"),
                22.0,
                theme::TEXT_SECONDARY,
                false,
            );
        } else {
            canvas.text(
                Pos2::new(360.0, 360.0),
                Align2::LEFT_CENTER,
                "Enter a Roblox username.",
                20.0,
                theme::TEXT_SECONDARY,
                false,
            );
            let field_rect = Rect::from_min_size(Pos2::new(360.0, 390.0), Vec2::new(720.0, 60.0));
            let response = canvas.focusable("addfriend:field", field_rect, egui::Sense::click());
            if response.activated {
                self.add_friend.editing = true;
                self.add_friend.focus_requested = true;
            }
            if self.add_friend.focus_requested {
                self.add_friend.focus_requested = false;
                self.add_friend.editing = true;
            }
            canvas.rect_filled(
                field_rect,
                0,
                if self.add_friend.editing || response.focused {
                    theme::panel(235)
                } else {
                    theme::panel(200)
                },
            );
            canvas.rect_stroke(
                field_rect,
                0,
                if self.add_friend.editing {
                    Stroke::new(2.0, theme::ACCENT)
                } else {
                    Stroke::new(1.0, Color32::from_white_alpha(30))
                },
            );
            let text_rect = canvas.screen_rect(field_rect);
            let edit = canvas.ui.put(
                text_rect,
                egui::TextEdit::singleline(&mut self.add_friend.username)
                    .frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(18, 16)))
                    .hint_text("Username")
                    .font(FontId::new(
                        24.0 * canvas.scale,
                        egui::FontFamily::Proportional,
                    ))
                    .text_color(theme::TEXT_PRIMARY)
                    .align(egui::Align2::LEFT_CENTER)
                    .min_size(text_rect.size()),
            );
            if self.add_friend.editing && !edit.has_focus() {
                edit.request_focus();
            }
            if !self.add_friend.editing && edit.has_focus() {
                canvas
                    .ui
                    .memory_mut(|memory| memory.surrender_focus(edit.id));
            }
            if edit.changed() {
                self.add_friend.editing = true;
            }
        }

        let send = Rect::from_min_size(Pos2::new(360.0, 500.0), Vec2::new(260.0, 56.0));
        let send_label = if self.add_friend.target.is_some() {
            "Send Request"
        } else {
            "Find & Add"
        };
        if screens::button(
            canvas,
            "addfriend:send",
            send,
            send_label,
            ButtonKind::Primary,
            true,
        ) {
            self.submit_add_friend();
        }

        if let Some((message, error)) = &self.add_friend.status {
            canvas.text(
                Pos2::new(360.0, 590.0),
                Align2::LEFT_CENTER,
                message,
                18.0,
                if *error {
                    theme::NEGATIVE
                } else {
                    theme::SUCCESS
                },
                false,
            );
        }
    }

    fn submit_add_friend(&mut self) {
        let Some(token) = self
            .session
            .as_ref()
            .map(|session| session.token.expose_secret().to_owned())
        else {
            self.add_friend.status = Some(("Sign in to add friends.".to_owned(), true));
            return;
        };
        if let Some((user_id, _)) = &self.add_friend.target {
            self.add_friend.status = Some(("Sending friend request...".to_owned(), false));
            self.api
                .send_friend_request(token, *user_id, self.api_tx.clone());
            return;
        }
        let username = self.add_friend.username.trim().to_owned();
        if username.is_empty() {
            self.add_friend.status = Some(("Enter a username first.".to_owned(), true));
            return;
        }
        self.add_friend.status = Some(("Looking up the user...".to_owned(), false));
        self.api.resolve_username(username, self.api_tx.clone());
    }
}
