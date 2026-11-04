use std::sync::Arc;

use egui::{
    Align2, Color32, CornerRadius, FontFamily, FontId, Galley, Id, Painter, Pos2, Rect, Sense,
    Stroke, StrokeKind, TextureHandle, Ui, Vec2,
};

use crate::audio::Sfx;
use crate::focus::{FocusItem, FocusState};
use crate::theme;

pub struct Canvas<'a> {
    pub ui: &'a mut Ui,
    pub painter: Painter,
    pub origin: Pos2,
    pub scale: f32,
    pub translate: Vec2,
    pub focus: FocusState,
    pub registry: Vec<FocusItem>,
    pub input_enabled: bool,
    pub hover_enabled: bool,
    pub confirm: bool,
    pub focus_visible: bool,
    pub video_active: bool,
    pub video_luma: f32,
    pub sfx: Vec<Sfx>,
    light_family: FontFamily,
}

pub struct FocusResponse {
    pub activated: bool,
    pub focused: bool,
    pub hovered: bool,
}

impl<'a> Canvas<'a> {
    pub fn new(ui: &'a mut Ui, focus: FocusState, confirm: bool, hover_enabled: bool) -> Self {
        Self::build(ui, focus, confirm, hover_enabled, true)
    }

    pub fn overlay(ui: &'a mut Ui, focus: FocusState, confirm: bool, hover_enabled: bool) -> Self {
        Self::build(ui, focus, confirm, hover_enabled, false)
    }

    fn build(
        ui: &'a mut Ui,
        focus: FocusState,
        confirm: bool,
        hover_enabled: bool,
        clear: bool,
    ) -> Self {
        let available = ui.max_rect();
        let available = if available.width() > 1.0 && available.height() > 1.0 {
            available
        } else {
            Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 720.0))
        };

        if clear {
            ui.painter().rect_filled(available, 0, Color32::BLACK);
        }

        let scale = (available.width() / theme::CANVAS_W)
            .min(available.height() / theme::CANVAS_H)
            .max(0.01);
        let size = Vec2::new(theme::CANVAS_W * scale, theme::CANVAS_H * scale);
        let origin = available.center() - size * 0.5;
        let content = Rect::from_min_size(origin, size);
        ui.set_clip_rect(content);
        let painter = ui.painter().with_clip_rect(content);

        Self {
            ui,
            painter,
            origin,
            scale,
            translate: Vec2::ZERO,
            focus,
            registry: Vec::new(),
            input_enabled: true,
            hover_enabled,
            confirm,
            focus_visible: true,
            video_active: false,
            video_luma: 0.0,
            sfx: Vec::new(),
            light_family: FontFamily::Name(Arc::from("light")),
        }
    }

    pub fn finish(mut self) -> (FocusState, Vec<FocusItem>) {
        // A mouse focus ring should not linger after the pointer leaves the
        // control. Keyboard navigation still keeps focus until the next mouse
        // movement, which is the expected console/desktop hybrid behaviour.
        let pointer = self.ui.input(|input| {
            (
                input.pointer.delta() != Vec2::ZERO,
                input.pointer.hover_pos(),
            )
        });
        if self.hover_enabled && pointer.0 {
            if let Some(pointer) = pointer.1 {
                let over_control = self
                    .registry
                    .iter()
                    .any(|item| item.enabled && self.screen_rect(item.rect).contains(pointer));
                if !over_control {
                    self.focus.clear();
                }
            } else {
                self.focus.clear();
            }
        }
        (self.focus, self.registry)
    }

    pub fn pos(&self, x: f32, y: f32) -> Pos2 {
        self.origin + Vec2::new(x * self.scale, y * self.scale) + self.translate
    }

    pub fn screen_rect(&self, rect: Rect) -> Rect {
        Rect::from_min_max(
            self.pos(rect.min.x, rect.min.y),
            self.pos(rect.max.x, rect.max.y),
        )
    }

    pub fn font(&self, size: f32, light: bool) -> FontId {
        let family = if light {
            self.light_family.clone()
        } else {
            FontFamily::Proportional
        };
        FontId::new((size * self.scale).max(1.0), family)
    }

    pub fn text(
        &self,
        pos: Pos2,
        anchor: Align2,
        text: impl ToString,
        size: f32,
        color: Color32,
        light: bool,
    ) -> Rect {
        self.painter.text(
            self.pos(pos.x, pos.y),
            anchor,
            text,
            self.font(size, light),
            color,
        )
    }

    /// Text drawn over the live/video background needs a small dark edge to
    /// stay legible on bright frames, like the 2016 client's text shadows.
    pub fn text_with_shadow(
        &self,
        pos: Pos2,
        anchor: Align2,
        text: impl ToString,
        size: f32,
        color: Color32,
        light: bool,
    ) -> Rect {
        let text = text.to_string();
        self.text(
            Pos2::new(pos.x + 1.5, pos.y + 1.5),
            anchor,
            &text,
            size,
            Color32::from_black_alpha(165),
            light,
        );
        self.text(pos, anchor, text, size, color, light)
    }

    pub fn text_centered(
        &self,
        rect: Rect,
        text: impl ToString,
        size: f32,
        color: Color32,
        light: bool,
    ) -> Rect {
        self.text(
            rect.center(),
            Align2::CENTER_CENTER,
            text,
            size,
            color,
            light,
        )
    }

    pub fn text_centered_with_shadow(
        &self,
        rect: Rect,
        text: impl ToString,
        size: f32,
        color: Color32,
        light: bool,
    ) -> Rect {
        self.text_with_shadow(
            rect.center(),
            Align2::CENTER_CENTER,
            text,
            size,
            color,
            light,
        )
    }

    pub fn text_width(&self, text: &str, size: f32, light: bool) -> f32 {
        let galley =
            self.painter
                .layout_no_wrap(text.to_owned(), self.font(size, light), Color32::WHITE);
        galley.size().x / self.scale
    }

    pub fn text_tracked(
        &self,
        pos: Pos2,
        text: &str,
        size: f32,
        color: Color32,
        tracking: f32,
        light: bool,
    ) -> Rect {
        let mut job = egui::text::LayoutJob::simple(
            text.to_owned(),
            self.font(size, light),
            color,
            f32::INFINITY,
        );
        job.sections[0].format.extra_letter_spacing = tracking * self.scale;
        let galley = self.painter.layout_job(job);
        let rect = Rect::from_min_size(self.pos(pos.x, pos.y), galley.size());
        self.painter.galley(self.pos(pos.x, pos.y), galley, color);
        rect
    }

    pub fn wrapped(
        &self,
        rect: Rect,
        text: &str,
        size: f32,
        color: Color32,
        light: bool,
    ) -> Arc<Galley> {
        self.painter.layout(
            text.to_owned(),
            self.font(size, light),
            color,
            rect.width() * self.scale,
        )
    }

    pub fn galley_clipped(&self, pos: Pos2, galley: Arc<Galley>, color: Color32, clip: Rect) {
        let painter = self.painter.with_clip_rect(self.screen_rect(clip));
        painter.galley(self.pos(pos.x, pos.y), galley, color);
    }

    pub fn rect_filled(&self, rect: Rect, radius: u8, color: Color32) {
        self.painter
            .rect_filled(self.screen_rect(rect), self.corner(radius), color);
    }

    pub fn rect_stroke(&self, rect: Rect, radius: u8, stroke: Stroke) {
        self.painter.rect_stroke(
            self.screen_rect(rect),
            self.corner(radius),
            stroke,
            StrokeKind::Inside,
        );
    }

    pub fn rect_gradient_v(&self, rect: Rect, top: Color32, bottom: Color32) {
        let screen = self.screen_rect(rect);
        let mut mesh = egui::Mesh::default();
        let base = mesh.vertices.len() as u32;
        mesh.colored_vertex(screen.left_top(), top);
        mesh.colored_vertex(screen.right_top(), top);
        mesh.colored_vertex(screen.right_bottom(), bottom);
        mesh.colored_vertex(screen.left_bottom(), bottom);
        mesh.add_triangle(base, base + 1, base + 2);
        mesh.add_triangle(base, base + 2, base + 3);
        self.painter.add(egui::Shape::mesh(mesh));
    }

    pub fn line(&self, a: Pos2, b: Pos2, stroke: Stroke) {
        self.painter
            .line_segment([self.pos(a.x, a.y), self.pos(b.x, b.y)], stroke);
    }

    pub fn circle_filled(&self, center: Pos2, radius: f32, color: Color32) {
        self.painter
            .circle_filled(self.pos(center.x, center.y), radius * self.scale, color);
    }

    pub fn circle_stroke(&self, center: Pos2, radius: f32, stroke: Stroke) {
        self.painter
            .circle_stroke(self.pos(center.x, center.y), radius * self.scale, stroke);
    }

    pub fn focus_ring(&self, rect: Rect, radius: u8) {
        let screen = self.screen_rect(rect);
        // The 2016 selection treatment is a thick white rule with a soft
        // cyan halo just outside it.
        for (expand, width, alpha) in [(1.2, 2.0, 70u8), (2.8, 2.5, 38), (4.6, 3.0, 18)] {
            self.painter.rect_stroke(
                screen.expand(expand * self.scale),
                self.corner(radius),
                Stroke::new(
                    width * self.scale,
                    Color32::from_rgba_unmultiplied(122, 207, 241, alpha),
                ),
                StrokeKind::Middle,
            );
        }
        self.painter.rect_stroke(
            screen,
            self.corner(radius),
            Stroke::new(4.0 * self.scale, Color32::WHITE),
            StrokeKind::Inside,
        );
    }

    pub fn focusable(&mut self, id: impl Into<String>, rect: Rect, sense: Sense) -> FocusResponse {
        self.focusable_enabled(id, rect, sense, true)
    }

    pub fn focusable_enabled(
        &mut self,
        id: impl Into<String>,
        rect: Rect,
        sense: Sense,
        enabled: bool,
    ) -> FocusResponse {
        let id = id.into();
        let interaction_sense = if self.input_enabled {
            sense
        } else {
            Sense::hover()
        };
        #[allow(deprecated)]
        let response = self.ui.interact(
            self.screen_rect(rect),
            Id::new(("focus", id.as_str())),
            interaction_sense,
        );
        let mouse_moved = self.ui.input(|input| input.pointer.delta() != Vec2::ZERO);
        if self.input_enabled
            && self.hover_enabled
            && response.hovered()
            && (mouse_moved || response.clicked() || response.dragged())
        {
            self.focus.set(id.clone());
        }
        let focused = self.focus.current.as_deref() == Some(id.as_str());
        let activated =
            self.input_enabled && enabled && (response.clicked() || (self.confirm && focused));
        if activated {
            log::debug!(
                "focus activate {id} (click={}, confirm={})",
                response.clicked(),
                self.confirm && focused
            );
            self.sfx.push(Sfx::Select);
        }
        let hovered = response.hovered();
        self.registry.push(FocusItem { id, rect, enabled });
        FocusResponse {
            activated,
            // Programmatic focus (for example, the initial focus set when a
            // screen opens) must not draw a ring until the user actually
            // hovers it or navigates with the keyboard/gamepad.
            focused: focused && (self.focus_visible || hovered),
            hovered,
        }
    }

    pub fn register(&mut self, id: impl Into<String>, rect: Rect, enabled: bool) {
        self.registry.push(FocusItem {
            id: id.into(),
            rect,
            enabled,
        });
    }

    pub fn is_focused(&self, id: &str) -> bool {
        self.focus.current.as_deref() == Some(id)
    }

    pub fn image_cover(&self, texture: &TextureHandle, rect: Rect, tint: Color32) {
        self.image_cover_bias(texture, rect, tint, 0.5);
    }

    /// Draws a cover-fitted image with a vertical crop bias. 0.0 shows the top
    /// edge of the texture, 0.5 centres it, and 1.0 shows the bottom edge.
    pub fn image_cover_bias(
        &self,
        texture: &TextureHandle,
        rect: Rect,
        tint: Color32,
        bias_y: f32,
    ) {
        let texture_size = texture.size_vec2();
        if texture_size.x <= 0.0 || texture_size.y <= 0.0 {
            return;
        }
        let target_aspect = rect.width() / rect.height().max(0.001);
        let texture_aspect = texture_size.x / texture_size.y;
        let uv = if texture_aspect > target_aspect {
            let width = target_aspect / texture_aspect;
            Rect::from_min_max(
                Pos2::new((1.0 - width) * 0.5, 0.0),
                Pos2::new((1.0 + width) * 0.5, 1.0),
            )
        } else {
            let height = texture_aspect / target_aspect;
            let min_y = (1.0 - height) * bias_y.clamp(0.0, 1.0);
            Rect::from_min_max(Pos2::new(0.0, min_y), Pos2::new(1.0, min_y + height))
        };
        self.painter
            .image(texture.id(), self.screen_rect(rect), uv, tint);
    }

    pub fn image_contain(&self, texture: &TextureHandle, rect: Rect, tint: Color32) {
        let texture_size = texture.size_vec2();
        if texture_size.x <= 0.0 || texture_size.y <= 0.0 {
            return;
        }
        let scale = (rect.width() / texture_size.x).min(rect.height() / texture_size.y);
        let target = Rect::from_center_size(rect.center(), texture_size * scale);
        self.painter.image(
            texture.id(),
            self.screen_rect(target),
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            tint,
        );
    }

    pub fn gradient_v(&self, rect: Rect, top: Color32, bottom: Color32) {
        const SLICES: usize = 24;
        for index in 0..SLICES {
            let t0 = index as f32 / SLICES as f32;
            let t1 = (index + 1) as f32 / SLICES as f32;
            let slice = Rect::from_min_max(
                Pos2::new(rect.min.x, rect.min.y + rect.height() * t0),
                Pos2::new(rect.max.x, rect.min.y + rect.height() * t1),
            );
            self.rect_filled(slice, 0, theme::mix(top, bottom, t0));
        }
    }

    pub fn corner(&self, radius: u8) -> CornerRadius {
        CornerRadius::same((radius as f32 * self.scale).round() as u8)
    }
}
