use std::time::{Duration, Instant};

use egui::{Key, Modifiers};
use gilrs::{Axis, Button, EventType, Gilrs};

use crate::focus::Dir;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Navigate(Dir),
    Confirm,
    Back,
    Settings,
    Search,
    TabPrev,
    TabNext,
    PagePrev,
    PageNext,
    FocusNext,
    FocusPrev,
    ToggleMute,
    Screenshot,
}

pub struct Gamepad {
    gilrs: Option<Gilrs>,
    connected: bool,
    held: Option<Dir>,
    pressed_at: Instant,
    last_repeat: Instant,
}

impl Gamepad {
    pub fn new() -> Self {
        let gilrs = Gilrs::new().ok();
        if let Some(gilrs) = &gilrs {
            for (id, pad) in gilrs.gamepads() {
                log::info!(
                    "gilrs device {id:?}: name={:?} connected={}",
                    pad.name(),
                    pad.is_connected()
                );
            }
        }
        let connected = gilrs
            .as_ref()
            .is_some_and(|gilrs| gilrs.gamepads().any(|(_, pad)| pad.is_connected()));
        Self {
            gilrs,
            connected,
            held: None,
            pressed_at: Instant::now(),
            last_repeat: Instant::now(),
        }
    }

    pub fn connected(&self) -> bool {
        self.connected
    }

    pub fn poll(&mut self) -> Vec<Action> {
        let mut actions = Vec::new();
        let Some(gilrs) = self.gilrs.as_mut() else {
            self.connected = false;
            return actions;
        };

        while let Some(event) = gilrs.next_event() {
            log::debug!("gilrs event {:?} from {:?}", event.event, event.id);
            match event.event {
                EventType::Connected => self.connected = true,
                EventType::Disconnected => {
                    self.connected = gilrs.gamepads().any(|(_, pad)| pad.is_connected());
                }
                EventType::ButtonPressed(button, _) => {
                    if let Some(action) = button_action(button) {
                        actions.push(action);
                    }
                }
                EventType::AxisChanged(axis, value, _) => {
                    if let Some(dir) = axis_direction(axis, value) {
                        actions.push(Action::Navigate(dir));
                    }
                }
                _ => {}
            }
        }

        if !self.connected {
            return actions;
        }

        let Some(pad) = gilrs
            .gamepads()
            .map(|(_, pad)| pad)
            .find(|pad| pad.is_connected())
        else {
            self.connected = false;
            return actions;
        };

        let horizontal = pad.value(Axis::LeftStickX);
        let vertical = pad.value(Axis::LeftStickY);
        let direction = if horizontal.abs() > vertical.abs() {
            if horizontal > 0.55 {
                Some(Dir::Right)
            } else if horizontal < -0.55 {
                Some(Dir::Left)
            } else {
                None
            }
        } else if vertical > 0.55 {
            Some(Dir::Down)
        } else if vertical < -0.55 {
            Some(Dir::Up)
        } else {
            None
        };

        let now = Instant::now();
        match direction {
            Some(direction) if self.held == Some(direction) => {
                let delay = if now.duration_since(self.pressed_at) < Duration::from_millis(350) {
                    Duration::from_millis(350)
                } else {
                    Duration::from_millis(90)
                };
                if now.duration_since(self.last_repeat) >= delay {
                    self.last_repeat = now;
                    actions.push(Action::Navigate(direction));
                }
            }
            Some(direction) => {
                self.held = Some(direction);
                self.pressed_at = now;
                self.last_repeat = now;
                actions.push(Action::Navigate(direction));
            }
            None => {
                self.held = None;
            }
        }

        actions
    }
}

fn button_action(button: Button) -> Option<Action> {
    match button {
        Button::DPadLeft => Some(Action::Navigate(Dir::Left)),
        Button::DPadRight => Some(Action::Navigate(Dir::Right)),
        Button::DPadUp => Some(Action::Navigate(Dir::Up)),
        Button::DPadDown => Some(Action::Navigate(Dir::Down)),
        Button::South => Some(Action::Confirm),
        Button::East => Some(Action::Back),
        Button::West => Some(Action::Settings),
        Button::North => Some(Action::Search),
        Button::LeftTrigger => Some(Action::TabPrev),
        Button::RightTrigger => Some(Action::TabNext),
        Button::LeftTrigger2 => Some(Action::PagePrev),
        Button::RightTrigger2 => Some(Action::PageNext),
        Button::Start | Button::Mode => Some(Action::Settings),
        Button::Select => Some(Action::Screenshot),
        _ => None,
    }
}

fn axis_direction(axis: Axis, value: f32) -> Option<Dir> {
    if value.abs() < 0.55 {
        return None;
    }
    match axis {
        Axis::DPadX if value > 0.0 => Some(Dir::Right),
        Axis::DPadX => Some(Dir::Left),
        Axis::DPadY if value > 0.0 => Some(Dir::Down),
        Axis::DPadY => Some(Dir::Up),
        _ => None,
    }
}

pub fn keyboard_actions(ctx: &egui::Context) -> Vec<Action> {
    let mut actions = Vec::new();
    ctx.input_mut(|input| {
        if input.consume_key(Modifiers::SHIFT, Key::Tab) {
            actions.push(Action::FocusPrev);
        } else if input.consume_key(Modifiers::NONE, Key::Tab) {
            actions.push(Action::FocusNext);
        }

        for (key, direction) in [
            (Key::ArrowLeft, Dir::Left),
            (Key::A, Dir::Left),
            (Key::ArrowRight, Dir::Right),
            (Key::D, Dir::Right),
            (Key::ArrowUp, Dir::Up),
            (Key::W, Dir::Up),
            (Key::ArrowDown, Dir::Down),
            (Key::S, Dir::Down),
        ] {
            if input.consume_key(Modifiers::NONE, key) {
                actions.push(Action::Navigate(direction));
                break;
            }
        }

        if input.consume_key(Modifiers::NONE, Key::Enter)
            || input.consume_key(Modifiers::NONE, Key::Space)
        {
            actions.push(Action::Confirm);
        }
        if input.consume_key(Modifiers::NONE, Key::Escape)
            || input.consume_key(Modifiers::NONE, Key::Backspace)
        {
            actions.push(Action::Back);
        }
        if input.consume_key(Modifiers::NONE, Key::Y) {
            actions.push(Action::Search);
        }
        if input.consume_key(Modifiers::NONE, Key::X) {
            actions.push(Action::Settings);
        }
        if input.consume_key(Modifiers::NONE, Key::M) {
            actions.push(Action::ToggleMute);
        }
        if input.consume_key(Modifiers::NONE, Key::F12) {
            actions.push(Action::Screenshot);
        }
    });
    actions
}
