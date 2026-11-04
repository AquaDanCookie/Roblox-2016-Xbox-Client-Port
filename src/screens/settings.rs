use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, TextureHandle, Vec2};

use crate::app::{App, SettingsTab};
use crate::audio::Sfx;
use crate::canvas::Canvas;
use crate::config::open_path;
use crate::screens::{self, ButtonKind};
use crate::theme;

impl App {
    pub fn show_settings(
        &mut self,
        canvas: &mut Canvas,
        background: &TextureHandle,
        wordmark: Option<&TextureHandle>,
    ) {
        screens::chrome(canvas, background, 180);

        if let Some(tab) = screens::nav(canvas, wordmark, crate::screens::NavTab::Settings) {
            self.go_tab(tab);
            return;
        }

        screens::section_label(canvas, 150.0, 252.0, "Settings");

        let panel = Rect::from_min_size(Pos2::new(150.0, 282.0), Vec2::new(1620.0, 720.0));
        screens::panel(canvas, panel);

        for (index, tab) in SettingsTab::ALL.iter().enumerate() {
            let rect = Rect::from_min_size(
                Pos2::new(190.0, 320.0 + index as f32 * 64.0),
                Vec2::new(300.0, 52.0),
            );
            let response = canvas.focusable(tab.id(), rect, Sense::click());
            let selected = self.settings.tab == *tab;
            if selected {
                canvas.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(rect.min.x, rect.min.y + 6.0),
                        Vec2::new(3.0, 40.0),
                    ),
                    0,
                    theme::ACCENT,
                );
            }
            let color = if selected {
                theme::TEXT_PRIMARY
            } else {
                theme::TEXT_SECONDARY
            };
            canvas.text(
                Pos2::new(rect.min.x + 16.0, rect.center().y),
                Align2::LEFT_CENTER,
                tab.label(),
                24.0,
                color,
                false,
            );
            if response.focused {
                canvas.focus_ring(rect, 0);
            }
            if response.activated {
                self.settings.tab = *tab;
                self.config.settings_tab = tab.index();
                self.config.save(&self.paths);
                self.focus.set(tab.id());
            }
        }

        let content = Rect::from_min_size(Pos2::new(560.0, 320.0), Vec2::new(1160.0, 620.0));
        match self.settings.tab {
            SettingsTab::Account => self.show_account(canvas, content),
            SettingsTab::Audio => self.show_audio(canvas, content),
            SettingsTab::Video => self.show_video(canvas, content),
            SettingsTab::Controls => self.show_controls(canvas, content),
            SettingsTab::Diagnostics => self.show_diagnostics(canvas, content),
        }
    }

    fn show_account(&mut self, canvas: &mut Canvas, content: Rect) {
        heading(canvas, content, "Account");

        let account = self.session.as_ref().map(|session| {
            (
                session.user.display().to_owned(),
                session.user.name.clone(),
                session.user.id,
                session.storage,
                session.headshots.get(&session.user.id).cloned(),
            )
        });

        let Some((display, username, user_id, storage, headshot_url)) = account else {
            canvas.text(
                Pos2::new(content.min.x, content.min.y + 80.0),
                Align2::LEFT_TOP,
                "You are browsing as a guest.",
                24.0,
                theme::TEXT_PRIMARY,
                false,
            );
            let note = Rect::from_min_size(
                Pos2::new(content.min.x, content.min.y + 130.0),
                Vec2::new(900.0, 150.0),
            );
            screens::panel(canvas, note);
            canvas.text(
                Pos2::new(note.min.x + 24.0, note.min.y + 28.0),
                Align2::LEFT_TOP,
                "Sign in with the Roblox login page, or import the existing Sober session.",
                20.0,
                theme::TEXT_SECONDARY,
                false,
            );
            canvas.text(
                Pos2::new(note.min.x + 24.0, note.min.y + 66.0),
                Align2::LEFT_TOP,
                "The Sober import is read-only. The app stores its own session.",
                18.0,
                theme::TEXT_MUTED,
                false,
            );
            if let Some(error) = &self.auth_error {
                canvas.text(
                    Pos2::new(content.min.x, content.min.y + 300.0),
                    Align2::LEFT_TOP,
                    error,
                    16.0,
                    theme::NEGATIVE,
                    false,
                );
            }
            if self.auth_state == crate::auth::AuthState::SigningIn {
                canvas.text(
                    Pos2::new(content.min.x, content.min.y + 300.0),
                    Align2::LEFT_TOP,
                    "Waiting for the Roblox sign-in window...",
                    16.0,
                    theme::TEXT_SECONDARY,
                    false,
                );
            }

            if screens::button(
                canvas,
                "settings:account-signin",
                Rect::from_min_size(
                    Pos2::new(content.min.x, content.min.y + 360.0),
                    Vec2::new(220.0, 52.0),
                ),
                "Sign in",
                ButtonKind::Primary,
                true,
            ) {
                self.start_webview_login();
            }
            if self.paths.sober_cookie_file().is_file()
                && screens::button(
                    canvas,
                    "settings:account-sober",
                    Rect::from_min_size(
                        Pos2::new(content.min.x + 240.0, content.min.y + 360.0),
                        Vec2::new(260.0, 52.0),
                    ),
                    "Use Sober session",
                    ButtonKind::Ghost,
                    true,
                )
            {
                self.start_sober_import();
            }
            return;
        };

        if let Some(url) = &headshot_url {
            self.images.ensure(url);
        }
        let texture = headshot_url
            .as_ref()
            .and_then(|url| self.images.get(url))
            .cloned();
        let head = Rect::from_min_size(
            Pos2::new(content.min.x, content.min.y + 70.0),
            Vec2::splat(120.0),
        );
        match &texture {
            Some(texture) => canvas.image_cover(texture, head, egui::Color32::WHITE),
            None => screens::skeleton_card(canvas, head),
        }
        canvas.rect_stroke(
            head,
            0,
            egui::Stroke::new(1.0, egui::Color32::from_white_alpha(24)),
        );

        canvas.text(
            Pos2::new(content.min.x + 150.0, content.min.y + 90.0),
            Align2::LEFT_CENTER,
            display,
            34.0,
            theme::TEXT_PRIMARY,
            true,
        );
        canvas.text(
            Pos2::new(content.min.x + 150.0, content.min.y + 132.0),
            Align2::LEFT_CENTER,
            format!("@{username}"),
            22.0,
            theme::TEXT_SECONDARY,
            false,
        );
        canvas.text(
            Pos2::new(content.min.x + 150.0, content.min.y + 166.0),
            Align2::LEFT_CENTER,
            format!("User ID {user_id}"),
            16.0,
            theme::TEXT_MUTED,
            false,
        );
        canvas.text(
            Pos2::new(content.min.x + 150.0, content.min.y + 196.0),
            Align2::LEFT_CENTER,
            format!("Session stored in {}", storage.label()),
            16.0,
            theme::TEXT_MUTED,
            false,
        );

        if screens::button(
            canvas,
            "settings:account-signout",
            Rect::from_min_size(
                Pos2::new(content.min.x + 150.0, content.min.y + 250.0),
                Vec2::new(220.0, 48.0),
            ),
            "Sign out",
            ButtonKind::Secondary,
            true,
        ) {
            self.sign_out();
            return;
        }

        canvas.text(
            Pos2::new(content.min.x, content.min.y + 340.0),
            Align2::LEFT_TOP,
            "Signed in through the Roblox login window or a read-only Sober import.",
            16.0,
            theme::TEXT_MUTED,
            false,
        );
    }

    fn show_audio(&mut self, canvas: &mut Canvas, content: Rect) {
        heading(canvas, content, "Audio");

        if let Some(outcome) = slider(
            canvas,
            "settings:master",
            content.min.x,
            content.min.y + 100.0,
            "Master volume",
            self.config.master_volume,
            &mut self.settings.drag_slider,
        ) {
            self.config.master_volume = outcome.value;
            self.music.set_levels(&self.config);
            if outcome.finished {
                self.config.save(&self.paths);
            }
        }
        if let Some(outcome) = slider(
            canvas,
            "settings:music",
            content.min.x,
            content.min.y + 200.0,
            "Music volume",
            self.config.music_volume,
            &mut self.settings.drag_slider,
        ) {
            self.config.music_volume = outcome.value;
            self.music.set_levels(&self.config);
            if outcome.finished {
                self.config.save(&self.paths);
            }
        }

        if toggle(
            canvas,
            "settings:mute",
            content.min.x,
            content.min.y + 300.0,
            "Mute all menu audio",
            self.config.muted,
        ) {
            self.config.muted = !self.config.muted;
            self.config.save(&self.paths);
            self.music.set_levels(&self.config);
        }

        if toggle(
            canvas,
            "settings:focusmute",
            content.min.x,
            content.min.y + 370.0,
            "Mute when the window loses focus",
            self.config.mute_on_focus_loss,
        ) {
            self.config.mute_on_focus_loss = !self.config.mute_on_focus_loss;
            self.config.save(&self.paths);
        }

        if toggle(
            canvas,
            "settings:ui-sounds",
            content.min.x,
            content.min.y + 440.0,
            "UI sounds",
            self.config.ui_sounds,
        ) {
            self.config.ui_sounds = !self.config.ui_sounds;
            self.config.save(&self.paths);
            self.toast_silent(if self.config.ui_sounds {
                "UI sounds on"
            } else {
                "UI sounds off"
            });
        }

        let sfx_status = if !self.config.ui_sounds {
            "UI sounds: off".to_owned()
        } else if self.paths.sfx_file("select").is_some() {
            "UI sounds: Xbox menu set (local)".to_owned()
        } else {
            "UI sounds: generated fallback".to_owned()
        };
        canvas.text(
            Pos2::new(content.min.x, content.min.y + 500.0),
            Align2::LEFT_CENTER,
            sfx_status,
            15.0,
            theme::TEXT_MUTED,
            false,
        );

        canvas.text(
            Pos2::new(content.min.x, content.min.y + 540.0),
            Align2::LEFT_CENTER,
            format!("Menu music: {}", self.music.status_text()),
            18.0,
            theme::TEXT_SECONDARY,
            false,
        );
        if let Some(path) = self.music.path() {
            canvas.text(
                Pos2::new(content.min.x, content.min.y + 570.0),
                Align2::LEFT_CENTER,
                path.display().to_string(),
                15.0,
                theme::TEXT_MUTED,
                false,
            );
        } else {
            canvas.text(
                Pos2::new(content.min.x, content.min.y + 570.0),
                Align2::LEFT_CENTER,
                format!(
                    "Add menu.mp3, menu.wav, or menu.flac to {}",
                    self.paths.music_dir().display()
                ),
                15.0,
                theme::TEXT_MUTED,
                false,
            );
        }

        canvas.text(
            Pos2::new(content.min.x, content.min.y + 600.0),
            Align2::LEFT_CENTER,
            "Menu music fades out while a game is launching and fades back in on return.",
            15.0,
            theme::TEXT_MUTED,
            false,
        );

        let open_width = button_width(canvas, "Open music folder");
        let open = Rect::from_min_size(
            Pos2::new(content.min.x, content.min.y + 620.0),
            Vec2::new(open_width, 48.0),
        );
        if screens::button(
            canvas,
            "settings:open-music",
            open,
            "Open music folder",
            ButtonKind::Secondary,
            true,
        ) {
            let folder = self.paths.music_dir();
            if let Err(error) = open_path(&folder) {
                self.toast(error);
            }
        }

        let reload = Rect::from_min_size(
            Pos2::new(open.max.x + 24.0, content.min.y + 620.0),
            Vec2::new(button_width(canvas, "Reload music"), 48.0),
        );
        if screens::button(
            canvas,
            "settings:reload-music",
            reload,
            "Reload music",
            ButtonKind::Secondary,
            true,
        ) {
            self.music.reload(&self.paths, &self.config);
            self.toast_silent("Menu music reloaded.");
        }
    }

    fn show_video(&mut self, canvas: &mut Canvas, content: Rect) {
        heading(canvas, content, "Video");

        if toggle(
            canvas,
            "settings:reduced-motion",
            content.min.x,
            content.min.y + 100.0,
            "Reduced motion",
            self.config.reduced_motion,
        ) {
            self.config.reduced_motion = !self.config.reduced_motion;
            self.config.save(&self.paths);
            if self.config.reduced_motion {
                self.stop_video();
            } else {
                self.reload_video();
            }
            self.toast_silent(if self.config.reduced_motion {
                "Reduced motion enabled."
            } else {
                "Reduced motion disabled."
            });
        }

        let status = if self.config.reduced_motion {
            "Background video is paused by Reduced motion.".to_owned()
        } else if let Some(video) = &self.video {
            format!(
                "Background video {}: {}",
                if video.has_frame() {
                    "playing"
                } else {
                    "loading"
                },
                video.path().display()
            )
        } else if let Some(error) = &self.video_error {
            format!("Background video unavailable: {error}")
        } else if let Some(path) = self.paths.background_video_file() {
            format!("Background video ready: {}", path.display())
        } else {
            format!(
                "Background video not installed. Add menu.mp4 to {}.",
                self.paths.background_dir().display()
            )
        };

        let status_rect = Rect::from_min_size(
            Pos2::new(content.min.x, content.min.y + 205.0),
            Vec2::new(1100.0, 64.0),
        );
        let galley = canvas.wrapped(status_rect, &status, 18.0, theme::TEXT_SECONDARY, false);
        canvas.galley_clipped(status_rect.min, galley, theme::TEXT_SECONDARY, status_rect);

        let open = Rect::from_min_size(
            Pos2::new(content.min.x, content.min.y + 295.0),
            Vec2::new(button_width(canvas, "Open background folder"), 48.0),
        );
        if screens::button(
            canvas,
            "settings:open-background",
            open,
            "Open background folder",
            ButtonKind::Secondary,
            true,
        ) {
            let folder = self.paths.background_dir();
            if let Err(error) = open_path(&folder) {
                self.toast(error);
            }
        }

        let reload = Rect::from_min_size(
            Pos2::new(open.max.x + 24.0, content.min.y + 295.0),
            Vec2::new(button_width(canvas, "Reload video"), 48.0),
        );
        if screens::button(
            canvas,
            "settings:reload-background",
            reload,
            "Reload video",
            ButtonKind::Secondary,
            true,
        ) {
            self.reload_video();
            self.toast_silent("Background video reloaded.");
        }

        canvas.text(
            Pos2::new(content.min.x, content.min.y + 370.0),
            Align2::LEFT_TOP,
            "The app loops the first video in this folder. It accepts mp4, webm, mkv, or mov. A bundled menu video is used when present; ffmpeg must be installed to replace it.",
            15.0,
            theme::TEXT_MUTED,
            false,
        );

        canvas.text(
            Pos2::new(content.min.x, content.min.y + 430.0),
            Align2::LEFT_TOP,
            "Fullscreen is handled by the window manager in this build. The shell draws on a fixed 1920x1080 virtual canvas and scales to the window.",
            18.0,
            theme::TEXT_MUTED,
            false,
        );
    }

    fn show_controls(&mut self, canvas: &mut Canvas, content: Rect) {
        heading(canvas, content, "Controls");
        let left = content.min.x;
        let right = content.min.x + 560.0;

        canvas.text(
            Pos2::new(left, content.min.y + 90.0),
            Align2::LEFT_TOP,
            "Keyboard",
            24.0,
            theme::TEXT_PRIMARY,
            false,
        );
        let keyboard = [
            ("Arrows / WASD", "Move focus"),
            ("Enter / Space", "Select"),
            ("Esc / Backspace", "Back"),
            ("Tab / Shift+Tab", "Cycle focus"),
            ("Y", "Search"),
            ("X", "Settings"),
            ("M", "Mute"),
            ("F12", "Screenshot"),
        ];
        for (index, (key, action)) in keyboard.iter().enumerate() {
            let y = content.min.y + 140.0 + index as f32 * 34.0;
            canvas.text(
                Pos2::new(left, y),
                Align2::LEFT_CENTER,
                *key,
                18.0,
                theme::TEXT_SECONDARY,
                false,
            );
            canvas.text(
                Pos2::new(left + 260.0, y),
                Align2::LEFT_CENTER,
                *action,
                18.0,
                theme::TEXT_MUTED,
                false,
            );
        }

        canvas.text(
            Pos2::new(right, content.min.y + 90.0),
            Align2::LEFT_TOP,
            "Controller",
            24.0,
            theme::TEXT_PRIMARY,
            false,
        );
        let controller = [
            ("Stick / D-pad", "Move focus"),
            ("A", "Select"),
            ("B", "Back"),
            ("X", "Settings"),
            ("Y", "Search"),
            ("LB / RB", "Previous / next tab"),
            ("LT / RT", "Page a game row"),
            ("Menu", "Settings"),
        ];
        for (index, (key, action)) in controller.iter().enumerate() {
            let y = content.min.y + 140.0 + index as f32 * 34.0;
            canvas.text(
                Pos2::new(right, y),
                Align2::LEFT_CENTER,
                *key,
                18.0,
                theme::TEXT_SECONDARY,
                false,
            );
            canvas.text(
                Pos2::new(right + 220.0, y),
                Align2::LEFT_CENTER,
                *action,
                18.0,
                theme::TEXT_MUTED,
                false,
            );
        }

        let scroll_y = content.min.y + 460.0;
        canvas.text(
            Pos2::new(left, scroll_y),
            Align2::LEFT_CENTER,
            "Game scroll speed",
            20.0,
            theme::TEXT_SECONDARY,
            false,
        );
        let scroll_value = self.config.game_scroll_speed.clamp(0.25, 2.0);
        if screens::button(
            canvas,
            "settings:scroll-speed-down",
            Rect::from_min_size(
                Pos2::new(left + 500.0, scroll_y - 22.0),
                Vec2::new(44.0, 44.0),
            ),
            "−",
            ButtonKind::Secondary,
            true,
        ) {
            self.config.game_scroll_speed = (scroll_value - 0.25).max(0.25);
            self.config.save(&self.paths);
        }
        if screens::button(
            canvas,
            "settings:scroll-speed-up",
            Rect::from_min_size(
                Pos2::new(left + 554.0, scroll_y - 22.0),
                Vec2::new(44.0, 44.0),
            ),
            "+",
            ButtonKind::Primary,
            true,
        ) {
            self.config.game_scroll_speed = (scroll_value + 0.25).min(2.0);
            self.config.save(&self.paths);
        }
        canvas.text(
            Pos2::new(left + 622.0, scroll_y),
            Align2::LEFT_CENTER,
            format!("{scroll_value:.2}x"),
            19.0,
            theme::TEXT_PRIMARY,
            false,
        );
        canvas.text(
            Pos2::new(left, scroll_y + 56.0),
            Align2::LEFT_CENTER,
            "Wheel scrolling in Games and the category browser. Saved automatically.",
            15.0,
            theme::TEXT_MUTED,
            false,
        );

        canvas.text(
            Pos2::new(left, content.max.y - 40.0),
            Align2::LEFT_CENTER,
            if self.gamepad.connected() {
                "A controller is connected."
            } else {
                "No controller is connected. Keyboard and mouse are active."
            },
            16.0,
            theme::TEXT_MUTED,
            false,
        );
    }

    fn show_diagnostics(&mut self, canvas: &mut Canvas, content: Rect) {
        heading(canvas, content, "Diagnostics");
        let last_error = self
            .last_api_error
            .as_deref()
            .map(|text| text.chars().take(110).collect::<String>())
            .unwrap_or_else(|| "none".to_owned());
        let lines = [
            format!("Version: {}", env!("CARGO_PKG_VERSION")),
            format!("API: {}", self.api_status),
            format!("Last error: {last_error}"),
            if self.sober.installed {
                format!("Sober: {} ({})", self.sober.version, self.sober.detail)
            } else {
                format!("Sober: {}", self.sober.detail)
            },
            format!(
                "Cordial fallback: {}",
                if self.sober.cordial_installed {
                    "installed"
                } else {
                    "not installed"
                }
            ),
            format!("Config: {}", self.paths.config_file().display()),
            format!("Screenshots: {}", self.paths.screenshot_dir().display()),
            format!(
                "App log: {}",
                self.paths.state_dir.join("app.log").display()
            ),
        ];
        for (index, line) in lines.iter().enumerate() {
            canvas.text(
                Pos2::new(content.min.x, content.min.y + 90.0 + index as f32 * 32.0),
                Align2::LEFT_CENTER,
                line,
                17.0,
                theme::TEXT_SECONDARY,
                false,
            );
        }

        let open_config = Rect::from_min_size(
            Pos2::new(content.min.x, content.min.y + 380.0),
            Vec2::new(button_width(canvas, "Open config folder"), 48.0),
        );
        if screens::button(
            canvas,
            "settings:open-config",
            open_config,
            "Open config folder",
            ButtonKind::Secondary,
            true,
        ) {
            if let Err(error) = open_path(&self.paths.config_dir) {
                self.toast(error);
            }
        }

        let copy = Rect::from_min_size(
            Pos2::new(open_config.max.x + 24.0, content.min.y + 380.0),
            Vec2::new(button_width(canvas, "Copy diagnostics"), 48.0),
        );
        if screens::button(
            canvas,
            "settings:copy-diagnostics",
            copy,
            "Copy diagnostics",
            ButtonKind::Secondary,
            true,
        ) {
            let text = format!(
                "Roblox 2016 Xbox shell {}\nAPI: {}\nLast error: {}\nSober: {}\nConfig: {}\nApp log: {}\n",
                env!("CARGO_PKG_VERSION"),
                self.api_status,
                self.last_api_error.as_deref().unwrap_or("none"),
                self.sober.detail,
                self.paths.config_file().display(),
                self.paths.state_dir.join("app.log").display()
            );
            canvas.ui.ctx().copy_text(text);
            self.toast_silent("Diagnostics copied.");
        }

        let open_logs = Rect::from_min_size(
            Pos2::new(copy.max.x + 24.0, content.min.y + 380.0),
            Vec2::new(button_width(canvas, "Open log folder"), 48.0),
        );
        if screens::button(
            canvas,
            "settings:open-logs",
            open_logs,
            "Open log folder",
            ButtonKind::Secondary,
            true,
        ) {
            if let Err(error) = open_path(&self.paths.state_dir) {
                self.toast(error);
            }
        }

        canvas.text(
            Pos2::new(content.min.x, content.min.y + 470.0),
            Align2::LEFT_TOP,
            "Logs are written to the app log and stderr. Screenshots are saved from F12.",
            16.0,
            theme::TEXT_MUTED,
            false,
        );
    }
}

fn button_width(canvas: &Canvas, label: &str) -> f32 {
    canvas.text_width(&label.to_uppercase(), 32.0, false) + 48.0
}

fn heading(canvas: &Canvas, content: Rect, text: &str) {
    canvas.text(
        Pos2::new(content.min.x, content.min.y),
        Align2::LEFT_CENTER,
        text,
        30.0,
        theme::TEXT_PRIMARY,
        true,
    );
}

struct SliderOutcome {
    value: f32,
    finished: bool,
}

fn slider(
    canvas: &mut Canvas,
    id: &str,
    x: f32,
    y: f32,
    label: &str,
    value: f32,
    drag: &mut Option<String>,
) -> Option<SliderOutcome> {
    let track = Rect::from_min_size(Pos2::new(x, y + 34.0), Vec2::new(520.0, 10.0));
    let hit = Rect::from_min_size(Pos2::new(x, y + 12.0), Vec2::new(600.0, 66.0));
    let (pointer, down, pressed) = canvas.ui.input(|input| {
        (
            input.pointer.interact_pos(),
            input.pointer.primary_down(),
            input.pointer.primary_pressed(),
        )
    });
    let hit_screen = canvas.screen_rect(hit);
    let track_screen = canvas.screen_rect(track);
    if pressed
        && drag.is_none()
        && let Some(pointer) = pointer
        && hit_screen.contains(pointer)
    {
        *drag = Some(id.to_owned());
    }
    let mut result = None;
    let mut shown = value.clamp(0.0, 1.0);
    if let Some(active) = drag.as_deref()
        && active == id
    {
        if let Some(pointer) = pointer {
            shown = ((pointer.x - track_screen.min.x) / track_screen.width()).clamp(0.0, 1.0);
            result = Some(SliderOutcome {
                value: shown,
                finished: false,
            });
        }
        if !down {
            *drag = None;
            result = Some(SliderOutcome {
                value: shown,
                finished: true,
            });
        }
    }
    canvas.text(
        Pos2::new(x, y),
        Align2::LEFT_CENTER,
        label,
        20.0,
        theme::TEXT_SECONDARY,
        false,
    );
    canvas.rect_filled(track, 3, Color32::from_white_alpha(30));
    let fill = Rect::from_min_size(track.min, Vec2::new(track.width() * shown, track.height()));
    canvas.rect_filled(fill, 3, theme::ACCENT);
    let knob = Pos2::new(track.min.x + track.width() * shown, track.center().y);
    canvas.circle_filled(knob, 10.0, theme::TEXT_PRIMARY);
    canvas.text(
        Pos2::new(track.max.x + 28.0, track.center().y),
        Align2::LEFT_CENTER,
        format!("{:.0}%", shown * 100.0),
        18.0,
        theme::TEXT_MUTED,
        false,
    );
    let response = canvas.focusable(id, hit, Sense::click());
    if response.focused {
        canvas.focus_ring(
            Rect::from_min_size(Pos2::new(x - 8.0, y - 8.0), Vec2::new(556.0, 72.0)),
            0,
        );
    }
    result
}

fn toggle(canvas: &mut Canvas, id: &str, x: f32, y: f32, label: &str, value: bool) -> bool {
    let row = Rect::from_min_size(Pos2::new(x, y - 12.0), Vec2::new(760.0, 48.0));
    let response = canvas.focusable(id, row, Sense::click());
    let box_rect = Rect::from_min_size(Pos2::new(x, y - 6.0), Vec2::splat(26.0));
    canvas.rect_stroke(
        box_rect,
        1,
        Stroke::new(1.5, Color32::from_white_alpha(110)),
    );
    if value {
        canvas.rect_filled(box_rect.shrink(6.0), 0, theme::ACCENT);
    }
    canvas.text(
        Pos2::new(x + 44.0, y + 8.0),
        Align2::LEFT_CENTER,
        label,
        20.0,
        theme::TEXT_SECONDARY,
        false,
    );
    if response.focused {
        let label_width = canvas.text_width(label, 20.0, false) + 76.0;
        canvas.focus_ring(
            Rect::from_min_size(Pos2::new(x - 16.0, y - 12.0), Vec2::new(label_width, 48.0)),
            0,
        );
    }
    if response.activated {
        // Toggles have their own switch sound instead of the generic select
        // click that focusable() queues for every activation.
        if matches!(canvas.sfx.last(), Some(Sfx::Select)) {
            canvas.sfx.pop();
        }
        canvas.sfx.push(Sfx::Toggle);
    }
    response.activated
}
