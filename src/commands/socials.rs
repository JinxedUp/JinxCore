use std::sync::Arc;

use pumpkin::command::{
    args::ConsumedArgs,
    CommandExecutor, CommandResult, CommandSender, tree::CommandTree,
};
use pumpkin::server::Server;
use pumpkin_util::text::{color::NamedColor, TextComponent};

use crate::{PluginState, branding};
use crate::commands::socials_common::{ensure_socials_file, load_socials, socials_path};

struct SocialsExecutor {
    state: Arc<PluginState>,
}

impl CommandExecutor for SocialsExecutor {
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

            if socials.is_empty() {
                let body = TextComponent::text("No socials configured.")
                    .color_named(NamedColor::Yellow);
                sender.send_message(branding::brand(body)).await;
                return Ok(());
            }

            let mut keys: Vec<_> = socials.keys().cloned().collect();
            keys.sort();
            let mut lines = String::from("Socials:\n");
            for key in keys {
                if let Some(value) = socials.get(&key) {
                    lines.push_str(&format!("{key}: {value}\n"));
                }
            }

            let body = TextComponent::text(lines.trim_end().to_string())
                .color_named(NamedColor::Aqua);
            sender.send_message(branding::brand(body)).await;
            Ok(())
        })
    }
}

pub fn socials_command_tree(state: Arc<PluginState>) -> CommandTree {
    CommandTree::new(["socials"], "Show all social links.")
        .execute(SocialsExecutor { state })
}
