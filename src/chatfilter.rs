use std::collections::HashSet;
use std::sync::{Arc, RwLock};

use pumpkin::plugin::{BoxFuture, Cancellable, EventHandler};
use pumpkin::plugin::events::player::player_chat::PlayerChatEvent;
use pumpkin::server::Server;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::config::{ChatFilterMode, Config};

pub const PERMISSION_CHATFILTER_BYPASS: &str = "JinxCore:chatfilter.bypass";

pub struct ChatFilterHandler {
    config: Arc<RwLock<Config>>,
}

impl ChatFilterHandler {
    pub fn new(config: Arc<RwLock<Config>>) -> Self {
        Self { config }
    }

    fn filter_message(
        message: &str,
        words: &HashSet<String>,
        replacement: &str,
    ) -> (bool, String) {
        let mut output = String::new();
        let mut token = String::new();
        let mut matched = false;

        for ch in message.chars() {
            if ch.is_alphanumeric() {
                token.push(ch);
            } else {
                if !token.is_empty() {
                    let token_lower = token.to_lowercase();
                    if words.contains(&token_lower) {
                        output.push_str(replacement);
                        matched = true;
                    } else {
                        output.push_str(&token);
                    }
                    token.clear();
                }
                output.push(ch);
            }
        }

        if !token.is_empty() {
            let token_lower = token.to_lowercase();
            if words.contains(&token_lower) {
                output.push_str(replacement);
                matched = true;
            } else {
                output.push_str(&token);
            }
        }

        (matched, output)
    }
}

impl EventHandler<PlayerChatEvent> for ChatFilterHandler {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PlayerChatEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let config = {
                let guard = self.config.read().unwrap();
                guard.clone()
            };

            if !config.chatfilter_enabled {
                return;
            }

            if event
                .player
                .has_permission(PERMISSION_CHATFILTER_BYPASS)
                .await
            {
                return;
            }

            if config.chatfilter_words.is_empty() {
                return;
            }

            let words: HashSet<String> = config
                .chatfilter_words
                .iter()
                .map(|word| word.to_lowercase())
                .collect();

            let (matched, filtered) =
                Self::filter_message(&event.message, &words, &config.chatfilter_replacement);

            if !matched {
                return;
            }

            match config.chatfilter_mode {
                ChatFilterMode::Cancel => {
                    event.set_cancelled(true);
                    if !config.chatfilter_notify_message.is_empty() {
                        let message =
                            TextComponent::text(config.chatfilter_notify_message.clone())
                                .color_named(NamedColor::Red);
                        event.player.send_system_message(&message).await;
                    }
                }
                ChatFilterMode::Replace => {
                    event.message = filtered;
                    if !config.chatfilter_notify_message.is_empty() {
                        let message =
                            TextComponent::text(config.chatfilter_notify_message.clone())
                                .color_named(NamedColor::Yellow);
                        event.player.send_system_message(&message).await;
                    }
                }
            }
        })
    }
}
