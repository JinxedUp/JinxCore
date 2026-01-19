use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use pumpkin::plugin::{BoxFuture, Cancellable, EventHandler};
use pumpkin::plugin::events::player::player_chat::PlayerChatEvent;
use pumpkin::server::Server;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::config::Config;

pub const PERMISSION_ANTISPAM_BYPASS: &str = "JinxCore:antispam.bypass";

struct AntiSpamState {
    history: HashMap<String, VecDeque<Instant>>,
    muted_until: HashMap<String, Instant>,
}

impl AntiSpamState {
    fn new() -> Self {
        Self {
            history: HashMap::new(),
            muted_until: HashMap::new(),
        }
    }

    fn check_message(
        &mut self,
        player_id: &str,
        now: Instant,
        window: Duration,
        max_messages: usize,
        mute_duration: Duration,
    ) -> bool {
        if let Some(until) = self.muted_until.get(player_id).copied() {
            if now < until {
                return true;
            }
            self.muted_until.remove(player_id);
        }

        let history = self.history.entry(player_id.to_string()).or_default();
        while let Some(ts) = history.front().copied() {
            if now.duration_since(ts) > window {
                history.pop_front();
            } else {
                break;
            }
        }

        history.push_back(now);
        if history.len() > max_messages {
            self.muted_until
                .insert(player_id.to_string(), now + mute_duration);
            history.clear();
            return true;
        }

        false
    }
}

pub struct AntiSpamHandler {
    config: Arc<RwLock<Config>>,
    state: Arc<Mutex<AntiSpamState>>,
}

impl AntiSpamHandler {
    pub fn new(config: Arc<RwLock<Config>>) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(AntiSpamState::new())),
        }
    }
}

impl EventHandler<PlayerChatEvent> for AntiSpamHandler {
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

            if !config.antispam_enabled {
                return;
            }

            if event.player.has_permission(PERMISSION_ANTISPAM_BYPASS).await {
                return;
            }

            let player_id = event.player.gameprofile.id.to_string();
            let should_block = {
                let mut state = self.state.lock().unwrap();
                state.check_message(
                    &player_id,
                    Instant::now(),
                    Duration::from_millis(config.antispam_window_ms),
                    config.antispam_max_messages,
                    Duration::from_secs(config.antispam_mute_seconds),
                )
            };

            if !should_block {
                return;
            }

            event.set_cancelled(true);
            if !config.antispam_notify_message.is_empty() {
                let message = TextComponent::text(config.antispam_notify_message.clone())
                    .color_named(NamedColor::Red);
                event.player.send_system_message(&message).await;
            }
        })
    }
}
