use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, TextureHandle, Vec2};

use secrecy::ExposeSecret;

use crate::app::App;
use crate::canvas::Canvas;
use crate::models::{ChatConversation, Load};
use crate::screens::{self, ButtonKind, NavTab};
use crate::theme;
use crate::util;

const LIST_X: f32 = 96.0;
const LIST_Y: f32 = 292.0;
const LIST_W: f32 = 520.0;
const LIST_H: f32 = 708.0;
const THREAD_X: f32 = 640.0;
const THREAD_W: f32 = 1184.0;

struct FriendChatRow {
    user_id: u64,
    name: String,
    username: String,
    presence: String,
    online: bool,
    headshot_url: Option<String>,
}

impl App {
    pub fn show_chat(
        &mut self,
        canvas: &mut Canvas,
        background: &TextureHandle,
        wordmark: Option<&TextureHandle>,
    ) {
        let dim = screens::adaptive_dim(canvas, 30);
        screens::chrome(canvas, background, dim);

        if let Some(tab) = screens::nav(canvas, wordmark, NavTab::Chat) {
            self.go_tab(tab);
            return;
        }

        let Some(user_id) = self.session.as_ref().map(|session| session.user.id) else {
            let panel = Rect::from_min_size(Pos2::new(300.0, 340.0), Vec2::new(1320.0, 220.0));
            screens::panel(canvas, panel);
            canvas.text_centered(
                panel,
                "Sign in to view and send chats.",
                24.0,
                theme::TEXT_SECONDARY,
                false,
            );
            if screens::button(
                canvas,
                "chat:signin",
                Rect::from_center_size(panel.center(), Vec2::new(220.0, 56.0)),
                "Sign in",
                ButtonKind::Primary,
                true,
            ) {
                self.start_webview_login();
            }
            return;
        };

        let list = Rect::from_min_size(Pos2::new(LIST_X, LIST_Y), Vec2::new(LIST_W, LIST_H));
        let thread = Rect::from_min_size(Pos2::new(THREAD_X, LIST_Y), Vec2::new(THREAD_W, LIST_H));
        screens::panel(canvas, list);
        screens::panel(canvas, thread);

        let filter_rect = Rect::from_min_size(
            Pos2::new(LIST_X + 16.0, LIST_Y + 16.0),
            Vec2::new(LIST_W - 32.0, 48.0),
        );
        canvas.rect_filled(filter_rect, 0, theme::panel(215));
        let filter_response = canvas.ui.put(
            canvas.screen_rect(filter_rect),
            egui::TextEdit::singleline(&mut self.chat.filter)
                .frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(14, 13)))
                .hint_text(if self.chat.show_friends {
                    "Search friends"
                } else {
                    "Search chats"
                })
                .font(FontId::new(
                    18.0 * canvas.scale,
                    egui::FontFamily::Proportional,
                ))
                .text_color(theme::TEXT_PRIMARY)
                .align(egui::Align2::LEFT_CENTER)
                .min_size(canvas.screen_rect(filter_rect).size()),
        );
        if filter_response.clicked() {
            canvas.focus.set("chat:filter");
        }
        canvas.rect_stroke(
            filter_rect,
            0,
            if filter_response.has_focus() || canvas.is_focused("chat:filter") {
                Stroke::new(2.0, theme::ACCENT)
            } else {
                Stroke::new(1.0, Color32::from_white_alpha(24))
            },
        );
        canvas.register("chat:filter", filter_rect, true);

        let mut switch_friends = None;
        for (index, (label, friends)) in [("CHATS", false), ("FRIENDS", true)].iter().enumerate() {
            let rect = Rect::from_min_size(
                Pos2::new(LIST_X + 16.0 + index as f32 * 252.0, LIST_Y + 72.0),
                Vec2::new(244.0, 34.0),
            );
            let response = canvas.focusable(format!("chat:list-tab:{label}"), rect, Sense::click());
            let active = self.chat.show_friends == *friends;
            if active {
                canvas.rect_filled(rect, 0, Color32::from_white_alpha(18));
                canvas.rect_filled(
                    Rect::from_min_size(
                        rect.left_bottom() - Vec2::new(0.0, 3.0),
                        Vec2::new(rect.width(), 3.0),
                    ),
                    0,
                    theme::ACCENT_BRIGHT,
                );
            } else if response.focused {
                canvas.rect_filled(rect, 0, Color32::from_white_alpha(8));
            }
            canvas.text_centered(
                rect,
                *label,
                17.0,
                if active {
                    theme::TEXT_PRIMARY
                } else if response.focused {
                    theme::FOCUS
                } else {
                    theme::TEXT_SECONDARY
                },
                false,
            );
            if response.activated {
                switch_friends = Some(*friends);
            }
        }
        if let Some(friends) = switch_friends {
            self.chat.show_friends = friends;
            self.chat.list_scroll = 0.0;
            return;
        }
        let list_top = LIST_Y + 118.0;
        let list_bottom = LIST_Y + LIST_H - 12.0;

        let conversations = match &self.chat.conversations {
            Load::Ready(conversations) => conversations.clone(),
            Load::Loading | Load::Idle => {
                canvas.text(
                    Pos2::new(LIST_X + 16.0, LIST_Y + 84.0),
                    Align2::LEFT_TOP,
                    "Loading chats...",
                    18.0,
                    theme::TEXT_MUTED,
                    false,
                );
                Vec::new()
            }
            Load::Failed(error) => {
                canvas.text(
                    Pos2::new(LIST_X + 16.0, LIST_Y + 84.0),
                    Align2::LEFT_TOP,
                    error,
                    16.0,
                    theme::NEGATIVE,
                    false,
                );
                if screens::button(
                    canvas,
                    "chat:retry",
                    Rect::from_min_size(
                        Pos2::new(LIST_X + 16.0, LIST_Y + 132.0),
                        Vec2::new(160.0, 44.0),
                    ),
                    "Retry",
                    ButtonKind::Secondary,
                    true,
                ) {
                    self.chat.requested = false;
                    self.chat.conversations = Load::Idle;
                }
                Vec::new()
            }
        };

        if self.chat.selected.is_none()
            && self.chat.start_with.is_none()
            && !self.chat.show_friends
            && let Some(first) = conversations
                .iter()
                .find(|conversation| !conversation.id.is_empty())
        {
            self.chat.selected = Some(first.id.clone());
        }

        let filter = self.chat.filter.trim().to_ascii_lowercase();
        let visible: Vec<&ChatConversation> = conversations
            .iter()
            .filter(|conversation| {
                if conversation.id.is_empty() || self.chat.closed_chats.contains(&conversation.id) {
                    return false;
                }
                if filter.is_empty() {
                    return true;
                }
                let title = conversation_title(conversation, user_id).to_ascii_lowercase();
                let latest = conversation
                    .messages
                    .iter()
                    .max_by(|a, b| a.created_at.cmp(&b.created_at))
                    .map(|message| message.content.to_ascii_lowercase())
                    .unwrap_or_default();
                title.contains(&filter) || latest.contains(&filter)
            })
            .collect();

        let mut friend_rows: Vec<FriendChatRow> = Vec::new();
        if self.chat.show_friends
            && let Some(session) = &self.session
            && let Some(friends) = session.friends.ready()
        {
            friend_rows = friends
                .iter()
                .map(|friend| {
                    let presence = session
                        .presence
                        .get(&friend.id)
                        .cloned()
                        .unwrap_or_default();
                    FriendChatRow {
                        user_id: friend.id,
                        name: if friend.display().is_empty() {
                            format!("User {}", friend.id)
                        } else {
                            friend.display().to_owned()
                        },
                        username: friend.name.clone(),
                        presence: presence.label().to_owned(),
                        online: presence.is_online(),
                        headshot_url: session.headshots.get(&friend.id).cloned(),
                    }
                })
                .filter(|friend| {
                    filter.is_empty()
                        || friend.name.to_ascii_lowercase().contains(&filter)
                        || friend.username.to_ascii_lowercase().contains(&filter)
                })
                .collect();
            friend_rows.sort_by(|a, b| {
                b.online
                    .cmp(&a.online)
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            });
            for row in &friend_rows {
                if let Some(url) = &row.headshot_url {
                    self.images.ensure(url);
                }
            }
        }
        let friend_textures: Vec<Option<TextureHandle>> = friend_rows
            .iter()
            .map(|row| {
                row.headshot_url
                    .as_ref()
                    .and_then(|url| self.images.get(url))
                    .cloned()
            })
            .collect();

        let row_count = if self.chat.show_friends {
            friend_rows.len()
        } else {
            visible.len()
        };
        let content_height = row_count as f32 * 76.0;
        let max_scroll = (content_height - (list_bottom - list_top)).max(0.0);
        let wheel = canvas.ui.input(|input| input.smooth_scroll_delta.y);
        let list_viewport = Rect::from_min_max(
            Pos2::new(LIST_X, list_top),
            Pos2::new(LIST_X + LIST_W, list_bottom),
        );
        if wheel != 0.0
            && canvas
                .ui
                .rect_contains_pointer(canvas.screen_rect(list_viewport))
        {
            self.chat.list_scroll = (self.chat.list_scroll - wheel).clamp(0.0, max_scroll);
        }
        self.chat.list_scroll = self.chat.list_scroll.clamp(0.0, max_scroll);

        let clip = canvas.screen_rect(Rect::from_min_max(
            Pos2::new(LIST_X, list_top),
            Pos2::new(LIST_X + LIST_W, list_bottom),
        ));
        let clipped = canvas.painter.with_clip_rect(clip);
        let original_painter = std::mem::replace(&mut canvas.painter, clipped);

        if self.chat.show_friends {
            if friend_rows.is_empty() {
                canvas.text(
                    Pos2::new(LIST_X + 16.0, list_top + 12.0),
                    Align2::LEFT_TOP,
                    if filter.is_empty() {
                        "No friends to chat with right now."
                    } else {
                        "No friends match that search."
                    },
                    16.0,
                    theme::TEXT_MUTED,
                    false,
                );
            }
            for (index, row) in friend_rows.iter().enumerate() {
                let y = list_top + index as f32 * 76.0 - self.chat.list_scroll;
                if y + 68.0 < list_top || y > list_bottom {
                    continue;
                }
                let rect =
                    Rect::from_min_size(Pos2::new(LIST_X + 8.0, y), Vec2::new(LIST_W - 16.0, 68.0));
                let response =
                    canvas.focusable(format!("chat:friend:{}", row.user_id), rect, Sense::click());
                if response.focused {
                    canvas.rect_filled(rect, 0, Color32::from_white_alpha(10));
                }
                let head = Rect::from_min_size(
                    Pos2::new(rect.min.x + 10.0, rect.min.y + 8.0),
                    Vec2::splat(52.0),
                );
                match friend_textures
                    .get(index)
                    .and_then(|texture| texture.as_ref())
                {
                    Some(texture) => canvas.image_cover(texture, head, Color32::WHITE),
                    None => canvas.rect_filled(head, 0, theme::PANEL_SOLID),
                }
                canvas.text(
                    Pos2::new(rect.min.x + 76.0, rect.min.y + 20.0),
                    Align2::LEFT_CENTER,
                    util::truncate_to_width(&row.name, rect.width() - 96.0, |text| {
                        canvas.text_width(text, 20.0, false)
                    }),
                    20.0,
                    theme::TEXT_PRIMARY,
                    false,
                );
                canvas.text(
                    Pos2::new(rect.min.x + 76.0, rect.min.y + 46.0),
                    Align2::LEFT_CENTER,
                    util::truncate_to_width(&row.presence, rect.width() - 96.0, |text| {
                        canvas.text_width(text, 15.0, false)
                    }),
                    15.0,
                    if row.online {
                        theme::SUCCESS
                    } else {
                        theme::TEXT_MUTED
                    },
                    false,
                );
                if response.activated {
                    canvas.painter = original_painter;
                    self.chat.start_with = Some((row.user_id, row.name.clone()));
                    self.chat.selected_name = Some(row.name.clone());
                    self.chat.start_requested = false;
                    self.chat.selected = None;
                    self.chat.messages = Load::Idle;
                    self.chat.messages_for = None;
                    self.chat.show_friends = false;
                    self.chat.list_scroll = 0.0;
                    self.chat.status =
                        Some((format!("Starting a chat with {}...", row.name), false));
                    return;
                }
            }
        } else {
            for (index, conversation) in visible.iter().enumerate() {
                let y = list_top + index as f32 * 76.0 - self.chat.list_scroll;
                if y + 68.0 < list_top || y > list_bottom {
                    continue;
                }
                let rect =
                    Rect::from_min_size(Pos2::new(LIST_X + 8.0, y), Vec2::new(LIST_W - 16.0, 68.0));
                let response = canvas.focusable(
                    format!("chat:conversation:{}", conversation.id),
                    rect,
                    Sense::click(),
                );
                if self.chat.selected.as_deref() == Some(conversation.id.as_str()) {
                    canvas.rect_filled(rect, 0, Color32::from_white_alpha(18));
                    canvas.rect_filled(
                        Rect::from_min_size(
                            Pos2::new(rect.min.x, rect.min.y + 8.0),
                            Vec2::new(3.0, 52.0),
                        ),
                        0,
                        theme::ACCENT_BRIGHT,
                    );
                } else if response.focused {
                    canvas.rect_filled(rect, 0, Color32::from_white_alpha(10));
                }
                let title = conversation_title(conversation, user_id);
                canvas.text(
                    Pos2::new(rect.min.x + 18.0, rect.min.y + 20.0),
                    Align2::LEFT_CENTER,
                    util::truncate_to_width(&title, rect.width() - 72.0, |text| {
                        canvas.text_width(text, 20.0, false)
                    }),
                    20.0,
                    theme::TEXT_PRIMARY,
                    false,
                );
                let latest = conversation
                    .messages
                    .iter()
                    .max_by(|a, b| a.created_at.cmp(&b.created_at))
                    .map(|message| message.content.clone())
                    .unwrap_or_else(|| "No messages yet".to_owned());
                canvas.text(
                    Pos2::new(rect.min.x + 18.0, rect.min.y + 46.0),
                    Align2::LEFT_CENTER,
                    util::truncate_to_width(&latest, rect.width() - 72.0, |text| {
                        canvas.text_width(text, 15.0, false)
                    }),
                    15.0,
                    theme::TEXT_MUTED,
                    false,
                );
                let close_rect = Rect::from_center_size(
                    Pos2::new(rect.max.x - 30.0, rect.center().y),
                    Vec2::splat(32.0),
                );
                let close = canvas.focusable(
                    format!("chat:close:{}", conversation.id),
                    close_rect,
                    Sense::click(),
                );
                let close_color = if close.focused {
                    theme::FOCUS
                } else {
                    theme::TEXT_MUTED
                };
                canvas.line(
                    Pos2::new(close_rect.center().x - 7.0, close_rect.center().y - 7.0),
                    Pos2::new(close_rect.center().x + 7.0, close_rect.center().y + 7.0),
                    Stroke::new(1.8, close_color),
                );
                canvas.line(
                    Pos2::new(close_rect.center().x + 7.0, close_rect.center().y - 7.0),
                    Pos2::new(close_rect.center().x - 7.0, close_rect.center().y + 7.0),
                    Stroke::new(1.8, close_color),
                );
                if close.focused {
                    canvas.circle_stroke(close_rect.center(), 19.0, Stroke::new(2.0, theme::FOCUS));
                }
                if close.activated {
                    self.chat.closed_chats.insert(conversation.id.clone());
                    if self.chat.selected.as_deref() == Some(conversation.id.as_str()) {
                        self.chat.selected = None;
                        self.chat.selected_name = None;
                        self.chat.messages = Load::Idle;
                        self.chat.messages_for = None;
                    }
                    self.chat.status = Some(("Chat closed.".to_owned(), false));
                    return;
                }
                if response.activated {
                    self.chat.selected_name = Some(title);
                    self.chat.selected = Some(conversation.id.clone());
                    self.chat.messages_for = None;
                    self.chat.status = None;
                }
            }

            if visible.is_empty() && !matches!(self.chat.conversations, Load::Loading | Load::Idle)
            {
                canvas.text(
                    Pos2::new(LIST_X + 16.0, list_top + 12.0),
                    Align2::LEFT_TOP,
                    if filter.is_empty() {
                        "No chats yet. Open the Friends tab to start one."
                    } else {
                        "No chats match that search."
                    },
                    16.0,
                    theme::TEXT_MUTED,
                    false,
                );
            }
        }
        canvas.painter = original_painter;

        let track = Rect::from_min_size(
            Pos2::new(LIST_X + LIST_W - 10.0, list_top),
            Vec2::new(5.0, list_bottom - list_top),
        );
        screens::drag_scrollbar(
            canvas,
            "chat:list-scrollbar",
            track,
            content_height,
            list_bottom - list_top,
            &mut self.chat.list_scroll,
            &mut self.scroll_drag,
        );

        let selected = self
            .chat
            .selected
            .as_deref()
            .and_then(|id| {
                conversations
                    .iter()
                    .find(|conversation| conversation.id == id)
            })
            .cloned();
        let title = if let Some(conversation) = selected {
            conversation_title(&conversation, user_id)
        } else if self.chat.selected.is_some() {
            if let Some(name) = &self.chat.selected_name {
                name.clone()
            } else {
                let message = self
                    .chat
                    .status
                    .as_ref()
                    .map(|(message, _)| message.as_str())
                    .unwrap_or("Select a conversation");
                canvas.text_centered(thread, message, 24.0, theme::TEXT_MUTED, false);
                return;
            }
        } else {
            let message = self
                .chat
                .status
                .as_ref()
                .map(|(message, _)| message.as_str())
                .unwrap_or("Select a conversation");
            canvas.text_centered(thread, message, 24.0, theme::TEXT_MUTED, false);
            return;
        };
        canvas.text_with_shadow(
            Pos2::new(THREAD_X + 24.0, LIST_Y + 30.0),
            Align2::LEFT_CENTER,
            util::truncate_to_width(&title, THREAD_W - 48.0, |text| {
                canvas.text_width(text, 28.0, true)
            }),
            28.0,
            theme::TEXT_PRIMARY,
            true,
        );
        canvas.line(
            Pos2::new(THREAD_X + 24.0, LIST_Y + 58.0),
            Pos2::new(THREAD_X + THREAD_W - 24.0, LIST_Y + 58.0),
            Stroke::new(1.0, Color32::from_white_alpha(25)),
        );

        let messages = match &self.chat.messages {
            Load::Ready(messages) => messages.clone(),
            Load::Loading | Load::Idle => Vec::new(),
            Load::Failed(error) => {
                canvas.text(
                    Pos2::new(THREAD_X + 24.0, LIST_Y + 80.0),
                    Align2::LEFT_TOP,
                    error,
                    18.0,
                    theme::NEGATIVE,
                    false,
                );
                Vec::new()
            }
        };

        if messages.is_empty() {
            canvas.text_centered(
                Rect::from_min_size(
                    Pos2::new(THREAD_X, LIST_Y + 120.0),
                    Vec2::new(THREAD_W, 80.0),
                ),
                if matches!(self.chat.messages, Load::Loading) {
                    "Loading messages..."
                } else {
                    "No messages yet. Say hello."
                },
                18.0,
                theme::TEXT_MUTED,
                false,
            );
        } else {
            let mut ordered = messages.clone();
            ordered.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            let start = ordered.len().saturating_sub(10);
            let mut message_y = LIST_Y + 78.0;
            for message in &ordered[start..] {
                if message.content.trim().is_empty() {
                    continue;
                }
                let max_width = 620.0;
                let galley = canvas.wrapped(
                    Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(max_width, 1000.0)),
                    &message.content,
                    18.0,
                    theme::TEXT_PRIMARY,
                    false,
                );
                let text_w = galley.size().x / canvas.scale;
                let text_h = galley.size().y / canvas.scale;
                let width = (text_w + 28.0).min(max_width + 28.0);
                let height = text_h + 20.0;
                if message_y + height > LIST_Y + LIST_H - 82.0 {
                    break;
                }
                let mine = message.sender_user_id == user_id;
                let x = if mine {
                    THREAD_X + THREAD_W - 24.0 - width
                } else {
                    THREAD_X + 24.0
                };
                let bubble = Rect::from_min_size(Pos2::new(x, message_y), Vec2::new(width, height));
                canvas.rect_filled(
                    bubble,
                    8,
                    if mine {
                        Color32::from_rgb(47, 82, 100)
                    } else {
                        Color32::from_rgb(35, 48, 55)
                    },
                );
                canvas.galley_clipped(
                    Pos2::new(bubble.min.x + 14.0, bubble.min.y + 10.0),
                    galley,
                    theme::TEXT_PRIMARY,
                    bubble.shrink(8.0),
                );
                message_y += height + 10.0;
            }
        }

        let compose_rect = Rect::from_min_size(
            Pos2::new(THREAD_X + 24.0, LIST_Y + LIST_H - 66.0),
            Vec2::new(THREAD_W - 188.0, 52.0),
        );
        let compose = canvas.focusable("chat:compose", compose_rect, Sense::click());
        if compose.activated {
            self.chat.editing = true;
            self.chat.focus_requested = true;
        }
        if self.chat.focus_requested {
            self.chat.focus_requested = false;
            self.chat.editing = true;
        }
        canvas.rect_filled(compose_rect, 0, theme::panel(220));
        canvas.rect_stroke(
            compose_rect,
            0,
            if self.chat.editing {
                Stroke::new(2.0, theme::ACCENT)
            } else {
                Stroke::new(1.0, Color32::from_white_alpha(25))
            },
        );
        let edit = canvas.ui.put(
            canvas.screen_rect(compose_rect),
            egui::TextEdit::singleline(&mut self.chat.compose)
                .frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(14, 15)))
                .hint_text("Say something")
                .font(FontId::new(
                    18.0 * canvas.scale,
                    egui::FontFamily::Proportional,
                ))
                .text_color(theme::TEXT_PRIMARY)
                .align(egui::Align2::LEFT_CENTER)
                .min_size(canvas.screen_rect(compose_rect).size()),
        );
        if edit.clicked() {
            self.chat.editing = true;
        }
        if self.chat.editing && !edit.has_focus() {
            edit.request_focus();
        }
        if !self.chat.editing && edit.has_focus() {
            canvas
                .ui
                .memory_mut(|memory| memory.surrender_focus(edit.id));
        }
        if edit.changed() {
            self.chat.editing = true;
        }

        let send = Rect::from_min_size(
            Pos2::new(THREAD_X + THREAD_W - 150.0, LIST_Y + LIST_H - 66.0),
            Vec2::new(126.0, 52.0),
        );
        if screens::button(canvas, "chat:send", send, "Send", ButtonKind::Primary, true) {
            self.send_chat_message();
        }

        if let Some((message, error)) = &self.chat.status {
            canvas.text(
                Pos2::new(THREAD_X + 24.0, LIST_Y + LIST_H - 82.0),
                Align2::LEFT_BOTTOM,
                message,
                15.0,
                if *error {
                    theme::NEGATIVE
                } else {
                    theme::SUCCESS
                },
                false,
            );
        }
    }

    pub fn send_chat_message(&mut self) {
        let Some(conversation_id) = self.chat.selected.clone() else {
            return;
        };
        let text = self.chat.compose.trim().to_owned();
        if text.is_empty() {
            return;
        }
        let Some(token) = self
            .session
            .as_ref()
            .map(|session| session.token.expose_secret().to_owned())
        else {
            self.chat.status = Some(("Sign in to send messages.".to_owned(), true));
            return;
        };
        self.chat.status = Some(("Sending...".to_owned(), false));
        self.api
            .send_chat_message(token, conversation_id, text, self.api_tx.clone());
    }
}

fn conversation_title(conversation: &ChatConversation, me: u64) -> String {
    if let Some(name) = &conversation.name
        && !name.is_empty()
    {
        return name.clone();
    }
    let mut names = Vec::new();
    for id in &conversation.participant_user_ids {
        if *id == me {
            continue;
        }
        let user = conversation
            .user_data
            .get(&id.to_string())
            .or_else(|| conversation.user_data.values().find(|user| user.id == *id));
        if let Some(user) = user {
            let name = if user.display_name.is_empty() {
                &user.name
            } else {
                &user.display_name
            };
            if !name.is_empty() {
                names.push(name.clone());
            }
        }
    }
    if names.is_empty() {
        "Conversation".to_owned()
    } else {
        names.join(", ")
    }
}
