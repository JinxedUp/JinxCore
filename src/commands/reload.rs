use std::sync::Arc;

use pumpkin::command::{
    CommandExecutor, CommandResult, CommandSender, args::ConsumedArgs, tree::CommandTree,
};
use pumpkin::command::tree::builder::{argument, literal};
use pumpkin::server::Server;
use pumpkin_util::text::TextComponent;
use pumpkin::command::args::simple::SimpleArgConsumer;

use crate::{PluginState, config, branding};
use crate::commands::{jinx_help_command, jinx_credits_command, jinx_health_command};

struct ReloadExecutor {
    state: Arc<PluginState>,
}

impl CommandExecutor for ReloadExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            match config::load_or_create(&self.state.data_dir) {
                Ok(new_config) => {
                    *self.state.config.write().unwrap() = new_config;
                }
                Err(err) => {
                    let message =
                        branding::brand(TextComponent::text(format!("Reload failed: {err}")));
                    sender.send_message(message).await;
                    return Ok(());
                }
            }

            let message = branding::brand(TextComponent::text("Config reloaded."));
            sender.send_message(message).await;
            Ok(())
        })
    }
}

pub fn jinx_command_tree(state: Arc<PluginState>) -> CommandTree {
    CommandTree::new(["jinx"], "JinxCore commands.")
        .then(
            literal("help")
                .execute(jinx_help_command())
                .then(argument("page", SimpleArgConsumer).execute(jinx_help_command())),
        )
        .then(literal("credits").execute(jinx_credits_command()))
        .then(literal("health").execute(jinx_health_command(Arc::clone(&state))))
        .then(literal("reload").execute(ReloadExecutor { state }))
}
