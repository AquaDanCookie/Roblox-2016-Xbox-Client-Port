use egui::{Color32, Pos2, Rect, TextureHandle, Vec2};

use crate::app::{App, open_url};
use crate::auth::AuthState;
use crate::canvas::Canvas;
use crate::screens::{self, ButtonKind, Screen};
use crate::theme;

impl App {
    pub fn show_signin(
        &mut self,
        canvas: &mut Canvas,
        background: &TextureHandle,
        wordmark: Option<&TextureHandle>,
    ) {
        let dim = if canvas.video_active {
            (6.0 + canvas.video_luma * 36.0).min(42.0) as u8
        } else {
            24
        };
        screens::chrome(canvas, background, dim);
        let scrim = if canvas.video_active {
            (12.0 + canvas.video_luma * 85.0).min(96.0) as u8
        } else {
            40
        };

        canvas.rect_gradient_v(
            Rect::from_min_size(Pos2::new(0.0, 430.0), Vec2::new(theme::CANVAS_W, 250.0)),
            Color32::TRANSPARENT,
            Color32::from_black_alpha(scrim),
        );
        canvas.rect_gradient_v(
            Rect::from_min_size(Pos2::new(0.0, 680.0), Vec2::new(theme::CANVAS_W, 300.0)),
            Color32::from_black_alpha(scrim),
            Color32::TRANSPARENT,
        );

        if let Some(wordmark) = wordmark {
            let rect = Rect::from_center_size(Pos2::new(960.0, 488.0), Vec2::new(610.0, 163.0));
            canvas.image_contain(wordmark, rect, egui::Color32::WHITE);
        }

        canvas.text_centered_with_shadow(
            Rect::from_min_size(Pos2::new(0.0, 592.0), Vec2::new(theme::CANVAS_W, 36.0)),
            "Community Created Gaming.",
            50.0,
            theme::TEXT_SECONDARY,
            true,
        );
        canvas.text_centered_with_shadow(
            Rect::from_min_size(Pos2::new(0.0, 655.0), Vec2::new(theme::CANVAS_W, 36.0)),
            "Limitless Possibilities.",
            50.0,
            theme::TEXT_SECONDARY,
            true,
        );

        let busy = matches!(self.auth_state, AuthState::SigningIn | AuthState::Checking);
        let create = Rect::from_center_size(Pos2::new(960.0, 756.0), Vec2::new(352.0, 52.0));
        let sign_in = Rect::from_center_size(Pos2::new(960.0, 862.0), Vec2::new(220.0, 52.0));
        let sober = Rect::from_center_size(Pos2::new(960.0, 936.0), Vec2::new(260.0, 36.0));
        let guest = Rect::from_center_size(Pos2::new(960.0, 976.0), Vec2::new(300.0, 36.0));

        if screens::button(
            canvas,
            "signin:create",
            create,
            "Create Account",
            ButtonKind::Secondary,
            true,
        ) && let Err(error) = open_url("https://www.roblox.com/signup")
        {
            self.toast(error);
        }

        if screens::button(
            canvas,
            "signin:signin",
            sign_in,
            "Sign In",
            ButtonKind::Primary,
            !busy,
        ) {
            self.start_webview_login();
        }

        if self.paths.sober_cookie_file().is_file()
            && screens::text_link(canvas, "signin:sober", sober, "Use Sober session", !busy)
        {
            self.start_sober_import();
        }

        if screens::text_link(canvas, "signin:guest", guest, "Browse as guest", true) {
            self.goto(Screen::Home);
        }

        let (message, color) = if let Some(error) = &self.auth_error {
            (error.clone(), theme::NEGATIVE)
        } else {
            match self.auth_state {
                AuthState::Checking => (
                    "Checking the saved session...".to_owned(),
                    theme::TEXT_SECONDARY,
                ),
                AuthState::SigningIn => (
                    "A Roblox sign-in window is open. Complete sign-in there.".to_owned(),
                    theme::TEXT_SECONDARY,
                ),
                _ => (
                    "The public catalog is available without an account.".to_owned(),
                    theme::TEXT_MUTED,
                ),
            }
        };
        canvas.text_centered_with_shadow(
            Rect::from_min_size(Pos2::new(0.0, 1024.0), Vec2::new(theme::CANVAS_W, 20.0)),
            message,
            14.0,
            color,
            false,
        );
    }
}
