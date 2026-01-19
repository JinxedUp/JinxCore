use std::sync::{Arc, RwLock};

use pumpkin::plugin::{BoxFuture, Cancellable, EventHandler};
use pumpkin::plugin::events::player::player_chat::PlayerChatEvent;
use pumpkin::server::Server;
use pumpkin::net::ClientPlatform;
use pumpkin_protocol::bedrock::server::text::SText;
use pumpkin_protocol::java::client::play::CSystemChatMessage;
use pumpkin_util::text::TextComponent;

use crate::config::Config;
use crate::webhook::{send_webhook, WebhookEvent};
use crate::discord_bot::{DiscordBridge, DiscordEvent, send_discord_event};

pub struct ChatFormatHandler {
    config: Arc<RwLock<Config>>,
    discord: Option<DiscordBridge>,
}

impl ChatFormatHandler {
    pub fn new(config: Arc<RwLock<Config>>, discord: Option<DiscordBridge>) -> Self {
        Self { config, discord }
    }
}

impl EventHandler<PlayerChatEvent> for ChatFormatHandler {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PlayerChatEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.cancelled() {
                return;
            }

            let config = {
                let guard = self.config.read().unwrap();
                guard.clone()
            };

            if !config.chat_format_enabled || config.chat_format.trim().is_empty() {
                return;
            }

            let name = event.player.gameprofile.name.clone();
            let decorated = TextComponent::chat_decorated(
                config.chat_format.clone(),
                name.clone(),
                event.message.clone(),
            );

            let je_packet = CSystemChatMessage::new(&decorated, false);
            let be_packet = SText::new(decorated.clone().get_text(), name);

            event.set_cancelled(true);

            send_webhook(
                &config,
                WebhookEvent::Chat,
                &event.player.gameprofile.name,
                Some(&event.message),
            );
            send_discord_event(
                self.discord.as_ref(),
                &config,
                DiscordEvent::Chat,
                &event.player.gameprofile.name,
                Some(&event.message),
            );

            if event.recipients.is_empty() {
                let world = event.player.world();
                world.broadcast_editioned(&je_packet, &be_packet).await;
                return;
            }

            for recipient in &event.recipients {
                match &recipient.client {
                    ClientPlatform::Java(client) => {
                        client.enqueue_packet(&je_packet).await;
                    }
                    ClientPlatform::Bedrock(client) => {
                        client.send_game_packet(&be_packet).await;
                    }
                }
            }
        })
    }
}
