use egui::{Color32, Pos2, Rect, TextureHandle, Vec2};

use crate::app::App;
use crate::canvas::Canvas;
use crate::screens;
use crate::theme;

impl App {
    pub fn show_boot(
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
            Rect::from_min_size(Pos2::new(0.0, 320.0), Vec2::new(theme::CANVAS_W, 280.0)),
            Color32::TRANSPARENT,
            Color32::from_black_alpha(scrim),
        );
        canvas.rect_gradient_v(
            Rect::from_min_size(Pos2::new(0.0, 600.0), Vec2::new(theme::CANVAS_W, 300.0)),
            Color32::from_black_alpha(scrim),
            Color32::TRANSPARENT,
        );

        if let Some(wordmark) = wordmark {
            let rect = Rect::from_center_size(Pos2::new(960.0, 488.0), Vec2::new(610.0, 163.0));
            canvas.image_contain(wordmark, rect, Color32::WHITE);
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

        let pulse = if self.config.reduced_motion {
            1.0
        } else {
            0.72 + 0.28 * (self.started.elapsed().as_secs_f32() * 1.4).sin().abs()
        };
        canvas.text_centered_with_shadow(
            Rect::from_min_size(Pos2::new(0.0, 820.0), Vec2::new(theme::CANVAS_W, 40.0)),
            "Press Any Button To Begin",
            34.0,
            theme::TEXT_PRIMARY.gamma_multiply(pulse),
            false,
        );
    }
}
