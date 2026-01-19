use std::time::SystemTime;

use pumpkin::command::{
    CommandExecutor, CommandResult, CommandSender,
    args::{Arg, ConsumedArgs, simple::SimpleArgConsumer},
    tree::CommandTree,
    tree::builder::argument,
};
use pumpkin::server::Server;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::{PluginState, branding};
use crate::seen::{find_by_name, format_duration};

const ARG_NAME: &str = "player";

struct SeenExecutor {
    state: std::sync::Arc<PluginState>,
}

impl CommandExecutor for SeenExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(name)) = args.get(ARG_NAME) else {
                return Ok(());
            };

            if let Some(player) = server.get_player_by_name(name).await {
                let world = player.world();
                let dim = world.dimension.minecraft_name;
                let msg = branding::brand(
                    TextComponent::text(format!("{name} is online ({dim})."))
                        .color_named(NamedColor::Green),
                );
                sender.send_message(msg).await;
                return Ok(());
            }

            let entry = {
                let seen = self.state.seen.read().unwrap();
                find_by_name(&seen, name).cloned()
            };
            if let Some(entry) = entry {
                let elapsed =
                    SystemTime::now().duration_since(entry.last_seen).unwrap_or_default();
                let msg = branding::brand(
                    TextComponent::text(format!(
                        "{} was last seen {} ago.",
                        entry.name,
                        format_duration(elapsed)
                    ))
                    .color_named(NamedColor::Yellow),
                );
                sender.send_message(msg).await;
            } else {
                let msg = branding::brand(
                    TextComponent::text(format!("No data for {name}."))
                        .color_named(NamedColor::Red),
                );
                sender.send_message(msg).await;
            }

            Ok(())
        })
    }
}

pub fn seen_command_tree(state: std::sync::Arc<PluginState>) -> CommandTree {
    CommandTree::new(["seen"], "Show last seen for a player.")
        .then(argument(ARG_NAME, SimpleArgConsumer).execute(SeenExecutor { state }))
}
