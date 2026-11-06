use egui::{Color32, Pos2, Rect, Stroke, TextureHandle, Vec2};

use crate::app::{App, LaunchPhase};
use crate::canvas::Canvas;
use crate::screens::{self, ButtonKind};
use crate::theme;
use crate::util;

impl App {
    pub fn show_launch(&mut self, canvas: &mut Canvas, background: &TextureHandle) {
        screens::chrome(canvas, background, 150);

        let panel = Rect::from_center_size(Pos2::new(960.0, 520.0), Vec2::new(920.0, 460.0));
        screens::panel(canvas, panel);

        let name = if self.launch.name.is_empty() {
            "the experience".to_owned()
        } else {
            self.launch.name.clone()
        };
        let phase = self.launch.phase;
        let show_spinner = matches!(
            phase,
            LaunchPhase::Checking | LaunchPhase::Launching | LaunchPhase::Waiting
        );
        if show_spinner {
            let rotation = if self.config.reduced_motion {
                0.0
            } else {
                self.launch.started.elapsed().as_secs_f32() * 2.4
            };
            draw_spinner(canvas, Pos2::new(960.0, 405.0), 34.0, rotation);
        }

        let title = match phase {
            LaunchPhase::Checking => "Checking Sober session...".to_owned(),
            LaunchPhase::Launching => "Handing off to Sober...".to_owned(),
            LaunchPhase::Waiting => format!("Joining {name}..."),
            LaunchPhase::Joined => format!("Joined {name}"),
            LaunchPhase::SessionUnverified => "Sober session could not be verified".to_owned(),
            LaunchPhase::Mismatch => "Different account".to_owned(),
            LaunchPhase::Failed => "Launch failed".to_owned(),
        };
        canvas.text_centered(
            Rect::from_min_size(Pos2::new(0.0, 470.0), Vec2::new(theme::CANVAS_W, 40.0)),
            title,
            30.0,
            theme::TEXT_PRIMARY,
            true,
        );

        let subtitle = match phase {
            LaunchPhase::Checking => {
                "The shell will use Sober's signed-in account for the game.".to_owned()
            }
            LaunchPhase::Launching => "Starting the Roblox runtime.".to_owned(),
            LaunchPhase::Waiting => {
                "Waiting for Sober to confirm the join. This can take up to 90 seconds.".to_owned()
            }
            LaunchPhase::Joined => {
                "Sober has the game. Switch to Sober to play, or return to the shell.".to_owned()
            }
            LaunchPhase::SessionUnverified => {
                self.launch.session_error.clone().unwrap_or_else(|| {
                    "Sober is not signed in. Open Sober, sign in, then retry.".to_owned()
                })
            }
            LaunchPhase::Mismatch => {
                let app = self
                    .session
                    .as_ref()
                    .map(|session| session.user.display().to_owned())
                    .unwrap_or_else(|| "Guest".to_owned());
                let sober = self
                    .launch
                    .sober_account
                    .as_ref()
                    .map(|account| account.display().to_owned())
                    .unwrap_or_else(|| "unknown".to_owned());
                format!(
                    "The shell is signed in as {app}. Sober is signed in as {sober}. Launching will join as {sober}."
                )
            }
            LaunchPhase::Failed => self
                .launch
                .message
                .clone()
                .unwrap_or_else(|| "The launch could not be completed.".to_owned()),
        };

        let subtitle_rect = Rect::from_min_size(
            Pos2::new(panel.min.x + 60.0, 510.0),
            Vec2::new(panel.width() - 120.0, 60.0),
        );
        let galley = canvas.wrapped(subtitle_rect, &subtitle, 18.0, theme::TEXT_SECONDARY, false);
        canvas.galley_centered_clipped(subtitle_rect, galley, theme::TEXT_SECONDARY);

        if let Some(path) = &self.launch.log_path
            && matches!(phase, LaunchPhase::Failed | LaunchPhase::Waiting)
        {
            let text = format!("Sober log: {}", path.display());
            let text = util::truncate_to_width(&text, 800.0, |value| {
                canvas.text_width(value, 14.0, false)
            });
            canvas.text_centered(
                Rect::from_min_size(Pos2::new(0.0, 585.0), Vec2::new(theme::CANVAS_W, 24.0)),
                text,
                14.0,
                theme::TEXT_MUTED,
                false,
            );
        }

        let primary = Rect::from_min_size(Pos2::new(540.0, 650.0), Vec2::new(220.0, 52.0));
        let joined_primary = Rect::from_min_size(Pos2::new(540.0, 650.0), Vec2::new(320.0, 52.0));
        let secondary = Rect::from_min_size(Pos2::new(790.0, 650.0), Vec2::new(220.0, 52.0));
        let back = Rect::from_min_size(Pos2::new(1040.0, 650.0), Vec2::new(160.0, 52.0));

        match phase {
            LaunchPhase::Checking | LaunchPhase::Launching | LaunchPhase::Waiting => {
                if screens::button(
                    canvas,
                    "launch:cancel",
                    primary,
                    "Cancel",
                    ButtonKind::Secondary,
                    true,
                ) {
                    self.toast_silent("Stopped waiting. Sober may still launch.");
                    self.leave_launch();
                    return;
                }
            }
            LaunchPhase::Joined => {
                if screens::button(
                    canvas,
                    "launch:return",
                    joined_primary,
                    "Return to shell",
                    ButtonKind::Secondary,
                    true,
                ) {
                    self.leave_launch();
                    return;
                }
            }
            LaunchPhase::Failed => {
                if screens::button(
                    canvas,
                    "launch:retry",
                    primary,
                    "Retry",
                    ButtonKind::Primary,
                    true,
                ) {
                    self.retry_launch();
                }
                if screens::button(
                    canvas,
                    "launch:failed-open",
                    secondary,
                    "Open Sober",
                    ButtonKind::Secondary,
                    true,
                ) {
                    self.open_sober();
                }
                if screens::button(
                    canvas,
                    "launch:failed-back",
                    back,
                    "Back",
                    ButtonKind::Ghost,
                    true,
                ) {
                    self.leave_launch();
                    return;
                }
            }
            LaunchPhase::SessionUnverified => {
                if screens::button(
                    canvas,
                    "launch:anyway",
                    primary,
                    "Launch anyway",
                    ButtonKind::Primary,
                    true,
                ) {
                    self.launch_anyway();
                }
                if screens::button(
                    canvas,
                    "launch:open",
                    secondary,
                    "Open Sober",
                    ButtonKind::Secondary,
                    true,
                ) {
                    self.open_sober();
                }
                if screens::button(
                    canvas,
                    "launch:unverified-back",
                    back,
                    "Back",
                    ButtonKind::Ghost,
                    true,
                ) {
                    self.leave_launch();
                    return;
                }
            }
            LaunchPhase::Mismatch => {
                if screens::button(
                    canvas,
                    "launch:mismatch-launch",
                    primary,
                    "Launch as Sober",
                    ButtonKind::Primary,
                    true,
                ) {
                    self.launch_anyway();
                }
                if screens::button(
                    canvas,
                    "launch:mismatch-cancel",
                    secondary,
                    "Cancel",
                    ButtonKind::Secondary,
                    true,
                ) {
                    self.leave_launch();
                    return;
                }
            }
        }
    }
}

fn draw_spinner(canvas: &Canvas, center: Pos2, radius: f32, rotation: f32) {
    const SEGMENTS: usize = 28;
    for index in 0..SEGMENTS {
        let angle = rotation + index as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
        let alpha = (index as f32 / SEGMENTS as f32 * 220.0) as u8;
        let inner = Pos2::new(
            center.x + angle.cos() * (radius - 10.0),
            center.y + angle.sin() * (radius - 10.0),
        );
        let outer = Pos2::new(
            center.x + angle.cos() * radius,
            center.y + angle.sin() * radius,
        );
        canvas.line(
            inner,
            outer,
            Stroke::new(3.0, Color32::from_rgba_unmultiplied(143, 211, 255, alpha)),
        );
    }
}
