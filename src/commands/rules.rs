use std::fs;
use std::sync::Arc;

use pumpkin::command::{
    CommandExecutor, CommandResult, CommandSender, args::ConsumedArgs, tree::CommandTree,
};
use pumpkin::server::Server;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::{PluginState, branding};

const RULES_FILE_NAME: &str = "rules.txt";

struct RulesExecutor {
    state: Arc<PluginState>,
}

fn ensure_rules_file(path: &std::path::Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if !path.exists() {
        let default_text = "1. Be respectful.\n2. No cheating.\n3. No griefing.\n";
        fs::write(path, default_text).map_err(|e| e.to_string())?;
    }
    Ok(())
}

impl CommandExecutor for RulesExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let path = self.state.data_dir.join(RULES_FILE_NAME);
            if let Err(err) = ensure_rules_file(&path) {
                let msg = branding::brand(
                    TextComponent::text(format!("Failed to initialize rules.txt: {err}"))
                        .color_named(NamedColor::Red),
                );
                sender.send_message(msg).await;
                return Ok(());
            }

            let Ok(content) = fs::read_to_string(&path) else {
                let msg = branding::brand(
                    TextComponent::text("Failed to read rules.txt.")
                        .color_named(NamedColor::Red),
                );
                sender.send_message(msg).await;
                return Ok(());
            };

            let body = TextComponent::text(content).color_named(NamedColor::White);
            sender.send_message(branding::brand(body)).await;
            Ok(())
        })
    }
}

pub fn rules_command_tree(state: Arc<PluginState>) -> CommandTree {
    CommandTree::new(["rules"], "Show server rules.")
        .execute(RulesExecutor { state })
}
