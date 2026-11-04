use std::path::PathBuf;
use std::sync::Arc;

use egui::{Color32, FontData, FontDefinitions, FontFamily};

pub const BG_DEEP: Color32 = Color32::from_rgb(16, 25, 30);
pub const BG_MID: Color32 = Color32::from_rgb(32, 43, 48);
pub const BG_LIGHT: Color32 = Color32::from_rgb(44, 59, 66);
pub const PANEL_SOLID: Color32 = Color32::from_rgb(27, 42, 49);
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(244, 248, 250);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(195, 210, 217);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(138, 166, 178);
pub const ACCENT: Color32 = Color32::from_rgb(74, 162, 210);
pub const ACCENT_BRIGHT: Color32 = Color32::from_rgb(121, 201, 232);
pub const FOCUS: Color32 = Color32::from_rgb(232, 250, 255);
pub const PLAY: Color32 = Color32::from_rgb(42, 164, 87);
pub const PLAY_HOVER: Color32 = Color32::from_rgb(52, 196, 106);
pub const SECONDARY_BUTTON: Color32 = Color32::from_rgb(88, 95, 105);
pub const DANGER: Color32 = Color32::from_rgb(205, 45, 53);
pub const SUCCESS: Color32 = Color32::from_rgb(82, 164, 100);
pub const NEGATIVE: Color32 = Color32::from_rgb(201, 106, 114);

pub const CANVAS_W: f32 = 1920.0;
pub const CANVAS_H: f32 = 1080.0;
pub const SAFE_X: f32 = 64.0;
pub const PROMPT_Y: f32 = 1002.0;

pub fn panel(alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(18, 30, 36, alpha)
}

pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_unmultiplied(
        mix(a.r(), b.r()),
        mix(a.g(), b.g()),
        mix(a.b(), b.b()),
        mix(a.a(), b.a()),
    )
}

pub struct FontInstall {
    pub primary: &'static str,
}

pub fn install_fonts(ctx: &egui::Context) -> FontInstall {
    let mut fonts = FontDefinitions::default();
    let mut proportional = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let mut monospace = fonts
        .families
        .get(&FontFamily::Monospace)
        .cloned()
        .unwrap_or_default();

    load_embedded_font(
        "source_sans",
        include_bytes!("../assets/fonts/SourceSansPro-Regular.ttf"),
        &mut fonts,
    );
    load_embedded_font(
        "source_sans_light",
        include_bytes!("../assets/fonts/SourceSansPro-Light.ttf"),
        &mut fonts,
    );
    load_embedded_font(
        "source_sans_bold",
        include_bytes!("../assets/fonts/SourceSansPro-Bold.ttf"),
        &mut fonts,
    );

    let noto = load_font(
        "noto",
        &[PathBuf::from("/usr/share/fonts/noto/NotoSans-Regular.ttf")],
        &mut fonts,
    );
    let emoji = load_font(
        "noto_emoji",
        &[PathBuf::from("/usr/share/fonts/noto/NotoColorEmoji.ttf")],
        &mut fonts,
    );

    proportional.insert(0, "source_sans".to_owned());
    proportional.insert(1, "source_sans_bold".to_owned());
    if noto {
        proportional.push("noto".to_owned());
    }
    if emoji {
        proportional.push("noto_emoji".to_owned());
        monospace.push("noto_emoji".to_owned());
    }
    fonts
        .families
        .insert(FontFamily::Proportional, proportional.clone());
    fonts.families.insert(FontFamily::Monospace, monospace);

    let mut light = vec!["source_sans_light".to_owned(), "source_sans".to_owned()];
    if noto {
        light.push("noto".to_owned());
    }
    if emoji {
        light.push("noto_emoji".to_owned());
    }
    fonts
        .families
        .insert(FontFamily::Name("light".into()), light);

    ctx.set_fonts(fonts);
    FontInstall {
        primary: "Source Sans Pro",
    }
}

fn load_embedded_font(name: &str, bytes: &'static [u8], fonts: &mut FontDefinitions) {
    fonts
        .font_data
        .insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
}

fn load_font(name: &str, paths: &[PathBuf], fonts: &mut FontDefinitions) -> bool {
    for path in paths {
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        fonts
            .font_data
            .insert(name.to_owned(), Arc::new(FontData::from_owned(bytes)));
        return true;
    }
    false
}

pub fn configure_egui(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG_DEEP;
    visuals.window_fill = PANEL_SOLID;
    visuals.extreme_bg_color = BG_DEEP;
    visuals.faint_bg_color = BG_MID;
    visuals.selection.bg_fill = ACCENT.gamma_multiply(0.55);
    visuals.selection.stroke = egui::Stroke::new(1.0, FOCUS);
    visuals.widgets.inactive.bg_fill = PANEL_SOLID;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT_SECONDARY);
    visuals.widgets.hovered.bg_fill = BG_LIGHT;
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.active.bg_fill = ACCENT.gamma_multiply(0.65);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
    ctx.set_visuals_of(egui::Theme::Dark, visuals.clone());
    ctx.set_visuals_of(egui::Theme::Light, visuals);
    ctx.set_theme(egui::Theme::Dark);

    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    style.animation_time = 0.12;
    style.text_styles = [
        (
            egui::TextStyle::Heading,
            egui::FontId::new(38.0, FontFamily::Name("light".into())),
        ),
        (
            egui::TextStyle::Body,
            egui::FontId::new(20.0, FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Button,
            egui::FontId::new(22.0, FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Small,
            egui::FontId::new(16.0, FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Monospace,
            egui::FontId::new(16.0, FontFamily::Monospace),
        ),
    ]
    .into();
    ctx.set_style_of(egui::Theme::Dark, style.clone());
    ctx.set_style_of(egui::Theme::Light, style);
}
