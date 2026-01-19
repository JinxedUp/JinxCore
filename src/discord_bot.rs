use std::sync::Arc;
use std::thread;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use tokio::sync::mpsc;

use pumpkin::server::Server;
use pumpkin_util::text::TextComponent;

use crate::config::Config;

#[derive(Clone)]
pub struct DiscordBridge {
    sender: mpsc::UnboundedSender<DiscordOutbound>,
}

pub enum DiscordEvent {
    Chat,
    Join,
    Leave,
}

enum DiscordOutbound {
    Message(String),
}

pub fn start_discord_bot(config: &Config, server: Arc<Server>) -> Option<DiscordBridge> {
    if !config.discord_bot_enabled {
        return None;
    }
    let token = config.discord_bot_token.trim();
    if token.is_empty() {
        return None;
    }
    if config.discord_bot_channel_id == 0 {
        return None;
    }

    let (tx, mut rx) = mpsc::unbounded_channel::<DiscordOutbound>();
    let channel_id = ChannelId::new(config.discord_bot_channel_id);
    let token = token.to_string();
    let to_mc_format = config.discord_to_mc_format.clone();

    thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("discord runtime");
        runtime.block_on(async move {
            let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
            let handler = DiscordHandler {
                channel_id,
                server,
                to_mc_format,
            };

            let mut client = match Client::builder(&token, intents)
                .event_handler(handler)
                .await
            {
                Ok(client) => client,
                Err(err) => {
                    log::error!("Discord bot failed to start: {err}");
                    return;
                }
            };

            let http = client.http.clone();
            tokio::spawn(async move {
                while let Some(outbound) = rx.recv().await {
                    match outbound {
                        DiscordOutbound::Message(content) => {
                            let builder = serenity::builder::CreateMessage::new().content(content);
                            let _ = channel_id
                                .send_message(&http, builder)
                                .await;
                        }
                    }
                }
            });

            if let Err(err) = client.start().await {
                log::error!("Discord bot stopped: {err}");
            }
        });
    });

    Some(DiscordBridge { sender: tx })
}

pub fn send_discord_event(
    bridge: Option<&DiscordBridge>,
    config: &Config,
    event: DiscordEvent,
    player: &str,
    message: Option<&str>,
) {
    let Some(bridge) = bridge else {
        return;
    };
    if !config.discord_bot_enabled {
        return;
    }
    let template = match event {
        DiscordEvent::Chat => &config.discord_chat_format,
        DiscordEvent::Join => &config.discord_join_format,
        DiscordEvent::Leave => &config.discord_leave_format,
    };
    let content = format_message(template, player, message);
    let _ = bridge.sender.send(DiscordOutbound::Message(content));
}

fn format_message(template: &str, player: &str, message: Option<&str>) -> String {
    let mut output = template.replace("{PLAYER}", player);
    if let Some(msg) = message {
        output = output.replace("{MESSAGE}", msg);
    }
    output
}

fn format_mc_message(template: &str, user: &str, message: &str) -> String {
    template
        .replace("{USER}", user)
        .replace("{MESSAGE}", message)
}

struct DiscordHandler {
    channel_id: ChannelId,
    server: Arc<Server>,
    to_mc_format: String,
}

#[async_trait]
impl EventHandler for DiscordHandler {
    async fn message(&self, _ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
        if msg.channel_id != self.channel_id {
            return;
        }
        let content = msg.content.trim();
        if content.is_empty() {
            return;
        }

        let formatted = format_mc_message(&self.to_mc_format, &msg.author.name, content);
        let text = TextComponent::text(formatted);
        broadcast_system_message(&self.server, text).await;
    }
}

async fn broadcast_system_message(server: &Server, message: TextComponent) {
    for world in server.worlds.read().await.iter() {
        for player in world.players.read().await.values() {
            player.send_system_message(&message).await;
        }
    }
}
