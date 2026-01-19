use std::sync::Arc;

use pumpkin::command::{
    CommandExecutor, CommandResult, CommandSender, args::ConsumedArgs, tree::CommandTree,
};
use pumpkin::server::Server;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::{PluginState, branding};
use crate::commands::socials_common::{ensure_socials_file, load_socials, socials_path};

struct DiscordExecutor {
    state: Arc<PluginState>,
}

impl CommandExecutor for DiscordExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let path = socials_path(&self.state.data_dir);
            if let Err(err) = ensure_socials_file(&path) {
                let msg = branding::brand(
                    TextComponent::text(format!("Failed to initialize socials.txt: {err}"))
                        .color_named(NamedColor::Red),
                );
                sender.send_message(msg).await;
                return Ok(());
            }

            let socials = match load_socials(&path) {
                Ok(value) => value,
                Err(err) => {
                    let msg = branding::brand(
                        TextComponent::text(format!("Failed to read socials.txt: {err}"))
                            .color_named(NamedColor::Red),
                    );
                    sender.send_message(msg).await;
                    return Ok(());
                }
            };

            match socials.get("discord") {
                Some(link) => {
                    let body = TextComponent::text(format!("Discord: {link}"))
                        .color_named(NamedColor::Aqua);
                    sender.send_message(branding::brand(body)).await;
                }
                None => {
                    let body = TextComponent::text("Discord link is not configured.")
                        .color_named(NamedColor::Yellow);
                    sender.send_message(branding::brand(body)).await;
                }
            }
            Ok(())
        })
    }
}

pub fn discord_command_tree(state: Arc<PluginState>) -> CommandTree {
    CommandTree::new(["discord"], "Show the Discord link.")
        .execute(DiscordExecutor { state })
}
