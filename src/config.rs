use std::fs;
use std::path::{Path, PathBuf};

use pumpkin_util::text::color::NamedColor;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatFilterMode {
    Replace,
    Cancel,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub join_enabled: bool,
    pub join_prefix: String,
    pub join_color: NamedColor,
    pub leave_enabled: bool,
    pub leave_prefix: String,
    pub leave_color: NamedColor,
    pub chat_format_enabled: bool,
    pub chat_format: String,
    pub webhook_enabled: bool,
    pub webhook_url: String,
    pub webhook_send_chat: bool,
    pub webhook_send_join: bool,
    pub webhook_send_leave: bool,
    pub webhook_use_player_name: bool,
    pub webhook_avatar_url: String,
    pub webhook_chat_format: String,
    pub webhook_join_format: String,
    pub webhook_leave_format: String,
    pub discord_bot_enabled: bool,
    pub discord_bot_token: String,
    pub discord_bot_channel_id: u64,
    pub discord_chat_format: String,
    pub discord_join_format: String,
    pub discord_leave_format: String,
    pub discord_to_mc_format: String,
    pub antispam_enabled: bool,
    pub antispam_window_ms: u64,
    pub antispam_max_messages: usize,
    pub antispam_mute_seconds: u64,
    pub antispam_notify_message: String,
    pub chatfilter_enabled: bool,
    pub chatfilter_mode: ChatFilterMode,
    pub chatfilter_replacement: String,
    pub chatfilter_notify_message: String,
    pub chatfilter_words: Vec<String>,
    pub scoreboard_enabled: bool,
    pub scoreboard_title: String,
    pub scoreboard_update_interval_sec: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            join_enabled: true,
            join_prefix: "[+]".to_string(),
            join_color: NamedColor::Green,
            leave_enabled: true,
            leave_prefix: "[-]".to_string(),
            leave_color: NamedColor::Red,
            chat_format_enabled: true,
            chat_format: "<{DISPLAYNAME}> {MESSAGE}".to_string(),
            webhook_enabled: false,
            webhook_url: String::new(),
            webhook_send_chat: true,
            webhook_send_join: true,
            webhook_send_leave: true,
            webhook_use_player_name: true,
            webhook_avatar_url: "https://mc-heads.net/avatar/{PLAYER}".to_string(),
            webhook_chat_format: "{PLAYER}: {MESSAGE}".to_string(),
            webhook_join_format: "{PLAYER} joined the server.".to_string(),
            webhook_leave_format: "{PLAYER} left the server.".to_string(),
            discord_bot_enabled: false,
            discord_bot_token: String::new(),
            discord_bot_channel_id: 0,
            discord_chat_format: "{PLAYER}: {MESSAGE}".to_string(),
            discord_join_format: "{PLAYER} joined the server.".to_string(),
            discord_leave_format: "{PLAYER} left the server.".to_string(),
            discord_to_mc_format: "[Discord] {USER}: {MESSAGE}".to_string(),
            antispam_enabled: true,
            antispam_window_ms: 3000,
            antispam_max_messages: 5,
            antispam_mute_seconds: 5,
            antispam_notify_message: "Please slow down.".to_string(),
            chatfilter_enabled: true,
            chatfilter_mode: ChatFilterMode::Replace,
            chatfilter_replacement: "****".to_string(),
            chatfilter_notify_message: "Please keep chat clean.".to_string(),
            chatfilter_words: vec!["badword".to_string()],
            scoreboard_enabled: true,
            scoreboard_title: "JinxCore".to_string(),
            scoreboard_update_interval_sec: 5,
        }
    }
}

const CONFIG_FILE_NAME: &str = "config.yml";

fn config_path(data_dir: &Path) -> PathBuf {
    data_dir.join(CONFIG_FILE_NAME)
}

fn yaml_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
}

fn serialize_color(color: &NamedColor) -> String {
    serde_yaml::to_string(color)
        .unwrap_or_else(|_| "white".to_string())
        .trim()
        .to_string()
}

fn default_config_yaml() -> String {
    let d = Config::default();
    format!(
        "# Chat\n\
chat_format_enabled: {chat_enabled}\n\
chat_format: \"{chat_format}\"\n\
\n\
# Join / Leave\n\
join_enabled: {join_enabled}\n\
join_prefix: \"{join_prefix}\"\n\
join_color: {join_color}\n\
leave_enabled: {leave_enabled}\n\
leave_prefix: \"{leave_prefix}\"\n\
leave_color: {leave_color}\n\
\n\
# Webhook\n\
webhook_enabled: {webhook_enabled}\n\
webhook_url: \"{webhook_url}\"\n\
webhook_send_chat: {webhook_send_chat}\n\
webhook_send_join: {webhook_send_join}\n\
webhook_send_leave: {webhook_send_leave}\n\
webhook_use_player_name: {webhook_use_player_name}\n\
webhook_avatar_url: \"{webhook_avatar_url}\"\n\
webhook_chat_format: \"{webhook_chat_format}\"\n\
webhook_join_format: \"{webhook_join_format}\"\n\
webhook_leave_format: \"{webhook_leave_format}\"\n\
\n\
# Discord Bot\n\
discord_bot_enabled: {discord_bot_enabled}\n\
discord_bot_token: \"{discord_bot_token}\"\n\
discord_bot_channel_id: {discord_bot_channel_id}\n\
discord_chat_format: \"{discord_chat_format}\"\n\
discord_join_format: \"{discord_join_format}\"\n\
discord_leave_format: \"{discord_leave_format}\"\n\
discord_to_mc_format: \"{discord_to_mc_format}\"\n\
\n\
# Anti-spam\n\
antispam_enabled: {antispam_enabled}\n\
antispam_window_ms: {antispam_window_ms}\n\
antispam_max_messages: {antispam_max_messages}\n\
antispam_mute_seconds: {antispam_mute_seconds}\n\
antispam_notify_message: \"{antispam_notify_message}\"\n\
\n\
# Chat filter\n\
chatfilter_enabled: {chatfilter_enabled}\n\
chatfilter_mode: {chatfilter_mode}\n\
chatfilter_replacement: \"{chatfilter_replacement}\"\n\
chatfilter_notify_message: \"{chatfilter_notify_message}\"\n\
chatfilter_words:\n\
  - \"{chatfilter_word}\"\n\
\n\
# Scoreboard\n\
scoreboard_enabled: {scoreboard_enabled}\n\
scoreboard_title: \"{scoreboard_title}\"\n\
scoreboard_update_interval_sec: {scoreboard_update_interval_sec}\n",
        chat_enabled = d.chat_format_enabled,
        chat_format = yaml_escape(&d.chat_format),
        join_enabled = d.join_enabled,
        join_prefix = yaml_escape(&d.join_prefix),
        join_color = serialize_color(&d.join_color),
        leave_enabled = d.leave_enabled,
        leave_prefix = yaml_escape(&d.leave_prefix),
        leave_color = serialize_color(&d.leave_color),
        webhook_enabled = d.webhook_enabled,
        webhook_url = yaml_escape(&d.webhook_url),
        webhook_send_chat = d.webhook_send_chat,
        webhook_send_join = d.webhook_send_join,
        webhook_send_leave = d.webhook_send_leave,
        webhook_use_player_name = d.webhook_use_player_name,
        webhook_avatar_url = yaml_escape(&d.webhook_avatar_url),
        webhook_chat_format = yaml_escape(&d.webhook_chat_format),
        webhook_join_format = yaml_escape(&d.webhook_join_format),
        webhook_leave_format = yaml_escape(&d.webhook_leave_format),
        discord_bot_enabled = d.discord_bot_enabled,
        discord_bot_token = yaml_escape(&d.discord_bot_token),
        discord_bot_channel_id = d.discord_bot_channel_id,
        discord_chat_format = yaml_escape(&d.discord_chat_format),
        discord_join_format = yaml_escape(&d.discord_join_format),
        discord_leave_format = yaml_escape(&d.discord_leave_format),
        discord_to_mc_format = yaml_escape(&d.discord_to_mc_format),
        antispam_enabled = d.antispam_enabled,
        antispam_window_ms = d.antispam_window_ms,
        antispam_max_messages = d.antispam_max_messages,
        antispam_mute_seconds = d.antispam_mute_seconds,
        antispam_notify_message = yaml_escape(&d.antispam_notify_message),
        chatfilter_enabled = d.chatfilter_enabled,
        chatfilter_mode = serde_yaml::to_string(&d.chatfilter_mode)
            .unwrap_or_else(|_| "replace".to_string())
            .trim(),
        chatfilter_replacement = yaml_escape(&d.chatfilter_replacement),
        chatfilter_notify_message = yaml_escape(&d.chatfilter_notify_message),
        chatfilter_word = d.chatfilter_words.get(0).cloned().unwrap_or_else(|| "badword".to_string()),
        scoreboard_enabled = d.scoreboard_enabled,
        scoreboard_title = yaml_escape(&d.scoreboard_title),
        scoreboard_update_interval_sec = d.scoreboard_update_interval_sec,
    )
}

pub fn load_or_create(data_dir: &Path) -> Result<Config, String> {
    if !data_dir.exists() {
        fs::create_dir_all(data_dir).map_err(|e| e.to_string())?;
    }

    let path = config_path(data_dir);
    if !path.exists() {
        fs::write(&path, default_config_yaml()).map_err(|e| e.to_string())?;
    }

    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let config = serde_yaml::from_str::<Config>(&content).map_err(|e| e.to_string())?;

    Ok(config)
}
