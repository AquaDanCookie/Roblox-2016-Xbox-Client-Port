use egui::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Debug)]
pub struct FocusItem {
    pub id: String,
    pub rect: Rect,
    pub enabled: bool,
}

#[derive(Default)]
pub struct FocusState {
    pub current: Option<String>,
}

impl FocusState {
    pub fn set(&mut self, id: impl Into<String>) {
        self.current = Some(id.into());
    }

    pub fn clear(&mut self) {
        self.current = None;
    }

    pub fn ensure(&mut self, items: &[FocusItem]) {
        if self
            .current
            .as_ref()
            .is_some_and(|id| items.iter().any(|item| &item.id == id))
        {
            return;
        }
        if self.current.is_none() {
            self.current = items
                .iter()
                .find(|item| item.enabled)
                .map(|item| item.id.clone());
        } else {
            // The focused control disappeared (usually because it scrolled
            // out of view). Falling back to the first item on the page
            // silently selects the Home tab; leave focus empty until the
            // user actually moves or clicks instead.
            self.current = None;
        }
    }

    pub fn navigate(&mut self, dir: Dir, items: &[FocusItem]) -> bool {
        let Some(current_id) = self.current.as_deref() else {
            self.ensure(items);
            return self.current.is_some();
        };
        let Some(current) = items.iter().find(|item| item.id == current_id) else {
            self.ensure(items);
            return false;
        };

        let mut best: Option<(f32, &FocusItem)> = None;
        for item in items {
            if item.id == current_id || !item.enabled {
                continue;
            }
            let delta = item.rect.center() - current.rect.center();
            let (primary, orthogonal) = match dir {
                Dir::Left => (-delta.x, delta.y.abs()),
                Dir::Right => (delta.x, delta.y.abs()),
                Dir::Up => (-delta.y, delta.x.abs()),
                Dir::Down => (delta.y, delta.x.abs()),
            };
            if primary <= 1.0 {
                continue;
            }
            let (current_range, item_range) = match dir {
                Dir::Left | Dir::Right => (current.rect.y_range(), item.rect.y_range()),
                Dir::Up | Dir::Down => (current.rect.x_range(), item.rect.x_range()),
            };
            let overlap = (current_range.max.min(item_range.max)
                - current_range.min.max(item_range.min))
            .max(0.0);
            let score = primary + orthogonal * 2.0 - overlap * 1.5;
            if best.is_none_or(|(best_score, _)| score < best_score) {
                best = Some((score, item));
            }
        }

        if let Some((_, item)) = best {
            self.current = Some(item.id.clone());
            true
        } else {
            false
        }
    }

    pub fn cycle(&mut self, items: &[FocusItem], forward: bool) -> bool {
        let enabled: Vec<&FocusItem> = items.iter().filter(|item| item.enabled).collect();
        if enabled.is_empty() {
            self.current = None;
            return false;
        }
        let index = self
            .current
            .as_ref()
            .and_then(|id| enabled.iter().position(|item| &item.id == id));
        let next = match (index, forward) {
            (Some(index), true) => (index + 1) % enabled.len(),
            (Some(index), false) => (index + enabled.len() - 1) % enabled.len(),
            (None, true) => 0,
            (None, false) => enabled.len() - 1,
        };
        self.current = Some(enabled[next].id.clone());
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Rect, pos2};

    fn item(id: &str, x: f32, y: f32) -> FocusItem {
        FocusItem {
            id: id.to_owned(),
            rect: Rect::from_min_size(pos2(x, y), egui::vec2(100.0, 50.0)),
            enabled: true,
        }
    }

    #[test]
    fn right_stays_in_row() {
        let items = vec![
            item("a", 0.0, 0.0),
            item("b", 120.0, 0.0),
            item("c", 0.0, 100.0),
        ];
        let mut focus = FocusState {
            current: Some("a".into()),
        };
        assert!(focus.navigate(Dir::Right, &items));
        assert_eq!(focus.current.as_deref(), Some("b"));
    }

    #[test]
    fn down_prefers_overlapping_column() {
        let items = vec![
            item("a", 0.0, 0.0),
            item("b", 0.0, 100.0),
            item("c", 120.0, 100.0),
        ];
        let mut focus = FocusState {
            current: Some("a".into()),
        };
        assert!(focus.navigate(Dir::Down, &items));
        assert_eq!(focus.current.as_deref(), Some("b"));
    }

    #[test]
    fn missing_focus_does_not_jump_to_the_first_item() {
        let mut focus = FocusState {
            current: Some("scrolled-row".into()),
        };
        focus.ensure(&[item("nav:home", 0.0, 0.0)]);
        assert!(focus.current.is_none());
    }

    #[test]
    fn skips_disabled() {
        let mut disabled = item("b", 120.0, 0.0);
        disabled.enabled = false;
        let items = vec![item("a", 0.0, 0.0), disabled, item("c", 240.0, 0.0)];
        let mut focus = FocusState {
            current: Some("a".into()),
        };
        assert!(focus.navigate(Dir::Right, &items));
        assert_eq!(focus.current.as_deref(), Some("c"));
    }
}
