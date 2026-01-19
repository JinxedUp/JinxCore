use std::sync::OnceLock;
use std::thread;

use reqwest::blocking::Client;
use serde::Serialize;

use crate::config::Config;

static CLIENT: OnceLock<Client> = OnceLock::new();

#[derive(Clone, Copy)]
pub enum WebhookEvent {
    Chat,
    Join,
    Leave,
}

#[derive(Serialize)]
struct WebhookEmbed {
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<u32>,
}

#[derive(Serialize)]
struct WebhookPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<Vec<WebhookEmbed>>,
}

fn format_message(template: &str, player: &str, message: Option<&str>) -> String {
    let mut output = template.replace("{PLAYER}", player);
    if let Some(msg) = message {
        output = output.replace("{MESSAGE}", msg);
    }
    output
}

pub fn send_webhook(config: &Config, event: WebhookEvent, player: &str, message: Option<&str>) {
    if !config.webhook_enabled {
        return;
    }
    if config.webhook_url.trim().is_empty() {
        return;
    }

    let (enabled, template) = match event {
        WebhookEvent::Chat => (config.webhook_send_chat, config.webhook_chat_format.as_str()),
        WebhookEvent::Join => (config.webhook_send_join, config.webhook_join_format.as_str()),
        WebhookEvent::Leave => (config.webhook_send_leave, config.webhook_leave_format.as_str()),
    };
    if !enabled {
        return;
    }

    let formatted = format_message(template, player, message);
    let url = config.webhook_url.clone();
    let username = if config.webhook_use_player_name {
        Some(player.to_string())
    } else {
        None
    };
    let avatar_url = if config.webhook_avatar_url.trim().is_empty() {
        None
    } else {
        Some(
            config
                .webhook_avatar_url
                .replace("{PLAYER}", player)
                .to_string(),
        )
    };

    let payload = match event {
        WebhookEvent::Chat => WebhookPayload {
            content: Some(formatted),
            username,
            avatar_url,
            embeds: None,
        },
        WebhookEvent::Join => WebhookPayload {
            content: None,
            username,
            avatar_url,
            embeds: Some(vec![WebhookEmbed {
                description: formatted,
                color: Some(0x57F287),
            }]),
        },
        WebhookEvent::Leave => WebhookPayload {
            content: None,
            username,
            avatar_url,
            embeds: Some(vec![WebhookEmbed {
                description: formatted,
                color: Some(0xED4245),
            }]),
        },
    };
    let client = CLIENT.get_or_init(Client::new).clone();
    thread::spawn(move || {
        let _ = client.post(url).json(&payload).send();
    });
}
