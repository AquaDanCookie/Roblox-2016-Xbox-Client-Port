pub mod add_friend;
pub mod boot;
pub mod browse;
pub mod chat;
pub mod details;
pub mod favorites;
pub mod games;
pub mod home;
pub mod launch;
pub mod placeholder;
pub mod profile;
pub mod search;
pub mod settings;
pub mod signin;

use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, TextureHandle, Vec2};

use crate::canvas::Canvas;
use crate::models::GameSummary;
use crate::theme;
use crate::util;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Boot,
    SignIn,
    Home,
    Games,
    Favorites,
    Search,
    Details,
    Settings,
    Launch,
    Browse,
    Profile,
    AddFriend,
    Chat,
    Placeholder,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavTab {
    Home,
    Avatar,
    Games,
    Friends,
    Chat,
    Robux,
    Settings,
}

impl NavTab {
    pub const ALL: [NavTab; 7] = [
        NavTab::Home,
        NavTab::Avatar,
        NavTab::Games,
        NavTab::Friends,
        NavTab::Chat,
        NavTab::Robux,
        NavTab::Settings,
    ];

    pub fn label(self) -> &'static str {
        match self {
            NavTab::Home => "HOME",
            NavTab::Avatar => "AVATAR",
            NavTab::Games => "GAMES",
            NavTab::Friends => "FRIENDS",
            NavTab::Chat => "CHAT",
            NavTab::Robux => "ROBUX",
            NavTab::Settings => "SETTINGS",
        }
    }

    pub fn screen(self) -> Screen {
        match self {
            NavTab::Home => Screen::Home,
            NavTab::Games => Screen::Games,
            NavTab::Chat => Screen::Chat,
            NavTab::Settings => Screen::Settings,
            NavTab::Avatar | NavTab::Friends | NavTab::Robux => Screen::Placeholder,
        }
    }
}

pub fn full() -> Rect {
    Rect::from_min_size(Pos2::ZERO, Vec2::new(theme::CANVAS_W, theme::CANVAS_H))
}

pub fn adaptive_dim(canvas: &Canvas, base: u8) -> u8 {
    if canvas.video_active {
        (canvas.video_luma * 12.0).clamp(0.0, 12.0) as u8
    } else {
        base
    }
}

pub fn chrome(canvas: &Canvas, background: &TextureHandle, dim: u8) {
    canvas.image_cover(background, full(), Color32::WHITE);
    if dim > 0 {
        canvas.rect_filled(full(), 0, Color32::from_black_alpha(dim));
    }
    canvas.rect_gradient_v(
        Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(theme::CANVAS_W, 280.0)),
        Color32::from_black_alpha(82),
        Color32::TRANSPARENT,
    );
    canvas.rect_gradient_v(
        Rect::from_min_size(
            Pos2::new(0.0, theme::CANVAS_H - 180.0),
            Vec2::new(theme::CANVAS_W, 180.0),
        ),
        Color32::TRANSPARENT,
        Color32::from_black_alpha(100),
    );
}

pub fn panel(canvas: &Canvas, rect: Rect) {
    canvas.rect_filled(rect, 0, theme::panel(205));
    canvas.rect_stroke(rect, 0, Stroke::new(1.0, Color32::from_white_alpha(18)));
}

pub fn section_label(canvas: &Canvas, x: f32, y: f32, text: &str) {
    let label = text.to_uppercase();
    canvas.text_tracked(
        Pos2::new(x + 1.5, y + 1.5),
        &label,
        24.0,
        Color32::from_black_alpha(180),
        1.5,
        false,
    );
    canvas.text_tracked(
        Pos2::new(x, y),
        &label,
        24.0,
        theme::TEXT_PRIMARY,
        1.5,
        false,
    );
}

pub fn nav(
    canvas: &mut Canvas,
    wordmark: Option<&TextureHandle>,
    current: NavTab,
) -> Option<NavTab> {
    if let Some(wordmark) = wordmark {
        let rect = Rect::from_min_size(Pos2::new(99.0, 42.0), Vec2::new(220.0, 55.0));
        canvas.image_contain(wordmark, rect, Color32::WHITE);
    }

    let mut x = 104.0;
    let y = 196.0;
    let mut clicked = None;
    for tab in NavTab::ALL {
        let label = tab.label();
        let width = canvas.text_width(label, 47.0, true);
        let rect = Rect::from_min_size(Pos2::new(x - 8.0, y - 30.0), Vec2::new(width + 16.0, 60.0));
        let response = canvas.focusable(format!("nav:{}", label), rect, Sense::click());
        let color = if tab == current {
            theme::TEXT_PRIMARY
        } else if response.focused {
            theme::FOCUS
        } else {
            theme::ACCENT_BRIGHT
        };
        canvas.text_with_shadow(
            Pos2::new(x, y),
            Align2::LEFT_CENTER,
            label,
            47.0,
            color,
            true,
        );
        // 2016 places the selection rule below the glyphs, not through the
        // descenders. The old +14 offset cut across the text.
        if tab == current {
            canvas.line(
                Pos2::new(x, y + 31.0),
                Pos2::new(x + width, y + 31.0),
                Stroke::new(3.0, theme::ACCENT_BRIGHT),
            );
        } else if response.focused {
            canvas.line(
                Pos2::new(x, y + 31.0),
                Pos2::new(x + width, y + 31.0),
                Stroke::new(3.0, theme::FOCUS),
            );
        }
        if response.activated {
            clicked = Some(tab);
        }
        x += width + 34.0;
    }
    clicked
}

#[allow(dead_code)]
pub fn prompt_row(canvas: &mut Canvas, _controller: bool, items: &[(&str, &str)]) {
    let mut x = theme::CANVAS_W - theme::SAFE_X;
    for (key, label) in items.iter().rev() {
        let label_width = canvas.text_width(label, 26.0, false);
        let total = 56.0 + label_width;
        x -= total;
        let center = Pos2::new(x + 24.0, theme::PROMPT_Y);
        canvas.circle_stroke(center, 24.0, Stroke::new(1.5, theme::ACCENT_BRIGHT));
        canvas.text_centered(
            Rect::from_center_size(center, Vec2::splat(48.0)),
            *key,
            22.0,
            theme::TEXT_PRIMARY,
            false,
        );
        canvas.text(
            Pos2::new(x + 54.0, theme::PROMPT_Y),
            Align2::LEFT_CENTER,
            *label,
            26.0,
            theme::TEXT_SECONDARY,
            false,
        );
        x -= 18.0;
    }
}

/// The 2016 overflow control is a wide grey bar with three white square dots.
pub fn ellipsis_button(canvas: &mut Canvas, id: &str, rect: Rect) -> bool {
    let response = canvas.focusable(id, rect, Sense::click());
    canvas.rect_filled(rect, 0, theme::SECONDARY_BUTTON);
    let dot = 8.0;
    let gap = 10.0;
    let total = dot * 3.0 + gap * 2.0;
    let start = rect.center().x - total * 0.5;
    for index in 0..3 {
        let x = start + index as f32 * (dot + gap);
        canvas.rect_filled(
            Rect::from_min_size(Pos2::new(x, rect.center().y - dot * 0.5), Vec2::splat(dot)),
            1,
            theme::TEXT_PRIMARY,
        );
    }
    if response.focused || response.hovered {
        canvas.focus_ring(rect, 0);
    }
    response.activated
}

/// 2016-style rating row: an outlined green thumb in a green ring on the
/// left, an outlined red thumb in a red ring on the right, a segmented vote
/// bar between them, and bare vote counts under the rings.
pub fn rating_summary(canvas: &Canvas, x: f32, y: f32, width: f32, up_votes: u64, down_votes: u64) {
    let icon_radius = 23.0;
    let up_center = Pos2::new(x + icon_radius, y);
    let down_center = Pos2::new(x + width - icon_radius, y);
    thumb_icon(canvas, up_center, true, theme::SUCCESS);
    thumb_icon(canvas, down_center, false, theme::NEGATIVE);

    let bar_x = x + icon_radius * 2.0 + 20.0;
    let bar_width = (width - (icon_radius * 2.0 + 20.0) * 2.0).max(40.0);
    let bar = Rect::from_min_size(Pos2::new(bar_x, y - 6.0), Vec2::new(bar_width, 12.0));
    let total = up_votes + down_votes;
    let positive = if total == 0 {
        0.0
    } else {
        up_votes as f32 / total as f32
    };
    const SEGMENTS: usize = 10;
    let segment_w = bar.width() / SEGMENTS as f32;
    let positive_segments = (positive * SEGMENTS as f32).round() as usize;
    for index in 0..SEGMENTS {
        let segment = Rect::from_min_size(
            Pos2::new(bar.min.x + index as f32 * segment_w, bar.min.y),
            Vec2::new((segment_w - 2.0).max(1.0), bar.height()),
        );
        let color = if index < positive_segments {
            theme::SUCCESS
        } else {
            theme::NEGATIVE
        };
        canvas.rect_filled(segment, 0, color);
    }

    canvas.text(
        Pos2::new(up_center.x, y + 44.0),
        Align2::CENTER_CENTER,
        util::format_number(up_votes),
        20.0,
        theme::SUCCESS,
        false,
    );
    canvas.text(
        Pos2::new(down_center.x, y + 44.0),
        Align2::CENTER_CENTER,
        util::format_number(down_votes),
        20.0,
        theme::NEGATIVE,
        false,
    );
}

fn thumb_icon(canvas: &Canvas, center: Pos2, up: bool, color: Color32) {
    let direction = if up { 1.0 } else { -1.0 };
    // A mitten-shaped thumbs-up (thumb on the left, fingers/palm on the
    // right). The previous path read as a little flag because the thumb and
    // finger edge were almost collinear.
    // Feather-style thumbs-up: the hand and a separate wrist cuff. The cuff
    // is what makes the silhouette read as a hand at small sizes; the old
    // single closed blob looked like a flag.
    let path = [
        (14.0, 9.0),
        (14.0, 5.0),
        (13.0, 4.0),
        (11.0, 2.0),
        (7.0, 11.0),
        (7.0, 22.0),
        (18.0, 22.0),
        (19.5, 21.0),
        (20.0, 19.5),
        (21.5, 11.5),
        (20.5, 10.0),
        (19.5, 9.0),
        (14.0, 9.0),
    ];
    let scale = 1.35;
    let points = path
        .iter()
        .map(|(x, y)| {
            canvas.pos(
                center.x + (x - 12.0) * scale,
                center.y + (y - 12.0) * scale * direction,
            )
        })
        .collect();
    canvas.circle_stroke(center, 23.0, Stroke::new(2.0, color));
    canvas
        .painter
        .add(egui::Shape::closed_line(points, Stroke::new(2.2, color)));

    let cuff_y1 = center.y + (11.0 - 12.0) * scale * direction;
    let cuff_y2 = center.y + (22.0 - 12.0) * scale * direction;
    let cuff = Rect::from_min_max(
        Pos2::new(center.x + (2.0 - 12.0) * scale, cuff_y1.min(cuff_y2)),
        Pos2::new(center.x + (7.0 - 12.0) * scale, cuff_y1.max(cuff_y2)),
    );
    canvas.rect_stroke(cuff, 2, Stroke::new(2.0, color));
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    Primary,
    Secondary,
    Play,
    Ghost,
    #[allow(dead_code)]
    Danger,
}

pub fn button(
    canvas: &mut Canvas,
    id: &str,
    rect: Rect,
    label: &str,
    kind: ButtonKind,
    enabled: bool,
) -> bool {
    let response = canvas.focusable_enabled(id, rect, Sense::click(), enabled);
    let (fill, text_color) = match kind {
        ButtonKind::Primary => (theme::ACCENT, theme::TEXT_PRIMARY),
        ButtonKind::Secondary => (theme::SECONDARY_BUTTON, theme::TEXT_PRIMARY),
        ButtonKind::Play => {
            if response.focused {
                (theme::PLAY_HOVER, theme::TEXT_PRIMARY)
            } else {
                (theme::PLAY, theme::TEXT_PRIMARY)
            }
        }
        ButtonKind::Ghost => (Color32::TRANSPARENT, theme::TEXT_SECONDARY),
        ButtonKind::Danger => (theme::DANGER, theme::TEXT_PRIMARY),
    };

    if kind == ButtonKind::Ghost {
        canvas.rect_filled(
            rect,
            0,
            Color32::from_black_alpha(if response.focused || response.hovered {
                60
            } else {
                40
            }),
        );
        if response.focused || response.hovered {
            canvas.rect_filled(rect, 0, Color32::from_white_alpha(12));
        }
        canvas.text_centered(rect, label, 22.0, text_color, false);
    } else {
        let fill = if enabled {
            fill
        } else {
            fill.gamma_multiply(0.45)
        };
        canvas.rect_filled(rect, 0, fill);
        if response.focused {
            canvas.focus_ring(rect, 0);
        } else if canvas.hover_enabled && response.hovered && enabled {
            canvas.rect_stroke(rect, 0, Stroke::new(1.0, Color32::from_white_alpha(80)));
        }
        canvas.text_centered(
            rect,
            label.to_uppercase(),
            32.0,
            if enabled {
                text_color
            } else {
                theme::TEXT_MUTED
            },
            false,
        );
    }

    response.activated
}

pub fn text_link(canvas: &mut Canvas, id: &str, rect: Rect, label: &str, enabled: bool) -> bool {
    let response = canvas.focusable_enabled(id, rect, Sense::click(), enabled);
    let color = if !enabled {
        theme::TEXT_MUTED.gamma_multiply(0.6)
    } else if response.focused {
        theme::FOCUS
    } else {
        theme::TEXT_SECONDARY
    };
    canvas.text_centered(rect, label, 18.0, color, false);
    if response.focused {
        canvas.line(
            Pos2::new(rect.center().x - rect.width() * 0.3, rect.center().y + 14.0),
            Pos2::new(rect.center().x + rect.width() * 0.3, rect.center().y + 14.0),
            Stroke::new(1.5, theme::FOCUS),
        );
    }
    response.activated
}

pub fn back_button(canvas: &mut Canvas, id: &str, label: &str) -> bool {
    let rect = Rect::from_min_size(Pos2::new(132.0, 40.0), Vec2::new(180.0, 48.0));
    let response = canvas.focusable(id, rect, Sense::click());
    let center = Pos2::new(rect.min.x + 24.0, rect.center().y);
    canvas.circle_stroke(center, 18.0, Stroke::new(2.0, theme::TEXT_PRIMARY));
    // Draw the arrow with lines instead of the "←" glyph. The glyph's font
    // bearings left it visibly high and off-centre inside the circle.
    let arrow = Stroke::new(2.0, theme::TEXT_PRIMARY);
    canvas.line(
        Pos2::new(center.x - 7.0, center.y),
        Pos2::new(center.x + 7.0, center.y),
        arrow,
    );
    canvas.line(
        Pos2::new(center.x - 7.0, center.y),
        Pos2::new(center.x - 1.0, center.y - 6.0),
        arrow,
    );
    canvas.line(
        Pos2::new(center.x - 7.0, center.y),
        Pos2::new(center.x - 1.0, center.y + 6.0),
        arrow,
    );
    canvas.text(
        Pos2::new(rect.min.x + 52.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        24.0,
        theme::TEXT_PRIMARY,
        true,
    );
    if response.focused || response.hovered {
        canvas.focus_ring(rect, 0);
    }
    response.activated
}

pub fn skeleton_card(canvas: &Canvas, rect: Rect) {
    canvas.rect_filled(rect, 0, theme::PANEL_SOLID);
    canvas.rect_stroke(rect, 0, Stroke::new(1.0, Color32::from_white_alpha(10)));
}

pub fn game_card(
    canvas: &mut Canvas,
    id: &str,
    rect: Rect,
    game: &GameSummary,
    texture: Option<&TextureHandle>,
) -> bool {
    let response = canvas.focusable(id, rect, Sense::click());

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

    if response.focused || (canvas.hover_enabled && response.hovered) {
        let name = util::truncate_to_width(&game.name, rect.width() - 42.0, |text| {
            canvas.text_width(text, 28.0, false)
        });
        let detail = if game.creator_name.is_empty() {
            format!("{} playing", util::format_count(game.player_count))
        } else {
            format!(
                "{} playing · by {}",
                util::format_count(game.player_count),
                game.creator_name
            )
        };
        let detail = util::truncate_to_width(&detail, rect.width() - 42.0, |text| {
            canvas.text_width(text, 16.0, false)
        });
        // The 2016 client put a solid dark caption block behind the focused
        // card's name and player count so it stayed readable on any thumbnail.
        let name_width = canvas.text_width(&name, 28.0, false);
        let detail_width = canvas.text_width(&detail, 16.0, false);
        let box_width = (name_width.max(detail_width) + 26.0)
            .min(rect.width() - 16.0)
            .max(80.0);
        let caption = Rect::from_min_size(
            Pos2::new(rect.min.x + 8.0, rect.max.y - 68.0),
            Vec2::new(box_width, 60.0),
        );
        canvas.rect_filled(caption, 0, Color32::from_rgba_unmultiplied(10, 16, 20, 220));
        canvas.text(
            Pos2::new(caption.min.x + 13.0, caption.min.y + 21.0),
            Align2::LEFT_CENTER,
            name,
            28.0,
            theme::TEXT_PRIMARY,
            false,
        );
        canvas.text(
            Pos2::new(caption.min.x + 13.0, caption.min.y + 43.0),
            Align2::LEFT_CENTER,
            detail,
            16.0,
            theme::TEXT_SECONDARY,
            false,
        );
    }

    if game.is_sponsored {
        let badge = Rect::from_min_size(
            Pos2::new(rect.min.x + 10.0, rect.min.y + 10.0),
            Vec2::new(92.0, 24.0),
        );
        canvas.rect_filled(badge, 0, Color32::from_black_alpha(170));
        canvas.text_centered(badge, "Sponsored", 13.0, theme::TEXT_SECONDARY, false);
    }

    if response.focused {
        canvas.focus_ring(rect, 0);
    }

    response.activated
}

pub fn error_panel(
    canvas: &mut Canvas,
    rect: Rect,
    message: &str,
    detail: Option<&str>,
    retry_id: &str,
) -> bool {
    panel(canvas, rect);
    canvas.text_centered(
        Rect::from_min_size(
            Pos2::new(rect.min.x, rect.min.y + 28.0),
            Vec2::new(rect.width(), 30.0),
        ),
        message,
        24.0,
        theme::TEXT_PRIMARY,
        true,
    );
    if let Some(detail) = detail {
        canvas.text_centered(
            Rect::from_min_size(
                Pos2::new(rect.min.x, rect.min.y + 66.0),
                Vec2::new(rect.width(), 24.0),
            ),
            detail,
            16.0,
            theme::TEXT_MUTED,
            false,
        );
    }
    let button_rect = Rect::from_center_size(
        Pos2::new(rect.center().x, rect.max.y - 48.0),
        Vec2::new(160.0, 48.0),
    );
    button(
        canvas,
        retry_id,
        button_rect,
        "Retry",
        ButtonKind::Secondary,
        true,
    )
}

#[derive(Clone, Debug)]
pub struct ScrollDrag {
    id: String,
    start_pointer: f32,
    start_thumb: f32,
}

/// Draws a 2016-style scrollbar and lets the user drag the thumb. The same
/// helper is used by every scrolling surface so track clicks, wheel jumps and
/// thumb dragging behave consistently.
pub fn drag_scrollbar(
    canvas: &mut Canvas,
    id: &str,
    track: Rect,
    content_height: f32,
    viewport_height: f32,
    scroll: &mut f32,
    drag: &mut Option<ScrollDrag>,
) {
    let max_scroll = (content_height - viewport_height).max(0.0);
    if max_scroll <= 0.0 {
        if drag.as_ref().is_some_and(|state| state.id == id) {
            *drag = None;
        }
        return;
    }
    *scroll = scroll.clamp(0.0, max_scroll);
    let primary_down = canvas.ui.input(|input| input.pointer.primary_down());
    if !primary_down && drag.is_some() {
        *drag = None;
    }
    let thumb_height =
        (track.height() * viewport_height / content_height).clamp(36.0, track.height());
    let travel = (track.height() - thumb_height).max(1.0);
    let thumb_y = track.min.y + (*scroll / max_scroll) * travel;
    let thumb = Rect::from_min_size(
        Pos2::new(track.min.x, thumb_y),
        Vec2::new(track.width(), thumb_height),
    );

    let hit = canvas.screen_rect(track.expand(7.0));
    let (pointer, down, pressed) = canvas.ui.input(|input| {
        (
            input.pointer.interact_pos(),
            input.pointer.primary_down(),
            input.pointer.primary_pressed(),
        )
    });
    if pressed
        && drag.is_none()
        && let Some(pointer) = pointer
        && hit.contains(pointer)
    {
        let thumb_hit = canvas.screen_rect(thumb).expand(6.0).contains(pointer);
        if !thumb_hit {
            let relative = ((pointer.y - hit.min.y) / hit.height().max(1.0)).clamp(0.0, 1.0);
            *scroll = relative * max_scroll;
        }
        *drag = Some(ScrollDrag {
            id: id.to_owned(),
            start_pointer: pointer.y,
            start_thumb: track.min.y + (*scroll / max_scroll) * travel,
        });
    }
    if let Some(state) = drag.as_mut()
        && state.id == id
    {
        if !down {
            *drag = None;
        } else if let Some(pointer) = pointer {
            let desired = state.start_thumb + (pointer.y - state.start_pointer) / canvas.scale;
            *scroll = ((desired - track.min.y) / travel).clamp(0.0, 1.0) * max_scroll;
        }
    }

    canvas.rect_filled(track, 2, Color32::from_white_alpha(24));
    let thumb_y = track.min.y + (*scroll / max_scroll) * travel;
    canvas.rect_filled(
        Rect::from_min_size(
            Pos2::new(track.min.x, thumb_y),
            Vec2::new(track.width(), thumb_height),
        ),
        2,
        theme::TEXT_MUTED,
    );
}

pub fn drag_horizontal_scrollbar(
    canvas: &mut Canvas,
    id: &str,
    track: Rect,
    content_width: f32,
    viewport_width: f32,
    scroll: &mut f32,
    drag: &mut Option<ScrollDrag>,
) {
    let max_scroll = (content_width - viewport_width).max(0.0);
    if max_scroll <= 0.0 {
        if drag.as_ref().is_some_and(|state| state.id == id) {
            *drag = None;
        }
        return;
    }
    *scroll = scroll.clamp(0.0, max_scroll);
    let primary_down = canvas.ui.input(|input| input.pointer.primary_down());
    if !primary_down && drag.is_some() {
        *drag = None;
    }
    let thumb_width = (track.width() * viewport_width / content_width).clamp(52.0, track.width());
    let travel = (track.width() - thumb_width).max(1.0);
    let thumb_x = track.min.x + (*scroll / max_scroll) * travel;
    let thumb = Rect::from_min_size(
        Pos2::new(thumb_x, track.min.y),
        Vec2::new(thumb_width, track.height()),
    );
    let hit = canvas.screen_rect(track.expand(7.0));
    let (pointer, down, pressed) = canvas.ui.input(|input| {
        (
            input.pointer.interact_pos(),
            input.pointer.primary_down(),
            input.pointer.primary_pressed(),
        )
    });
    if pressed
        && drag.is_none()
        && let Some(pointer) = pointer
        && hit.contains(pointer)
    {
        let thumb_hit = canvas.screen_rect(thumb).expand(6.0).contains(pointer);
        if !thumb_hit {
            let relative = ((pointer.x - hit.min.x) / hit.width().max(1.0)).clamp(0.0, 1.0);
            *scroll = relative * max_scroll;
        }
        *drag = Some(ScrollDrag {
            id: id.to_owned(),
            start_pointer: pointer.x,
            start_thumb: track.min.x + (*scroll / max_scroll) * travel,
        });
    }
    if let Some(state) = drag.as_mut()
        && state.id == id
    {
        if !down {
            *drag = None;
        } else if let Some(pointer) = pointer {
            let desired = state.start_thumb + (pointer.x - state.start_pointer) / canvas.scale;
            *scroll = ((desired - track.min.x) / travel).clamp(0.0, 1.0) * max_scroll;
        }
    }
    canvas.rect_filled(track, 2, Color32::from_white_alpha(24));
    let thumb_x = track.min.x + (*scroll / max_scroll) * travel;
    canvas.rect_filled(
        Rect::from_min_size(
            Pos2::new(thumb_x, track.min.y),
            Vec2::new(thumb_width, track.height()),
        ),
        2,
        theme::TEXT_MUTED,
    );
}
