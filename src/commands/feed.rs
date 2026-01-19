use pumpkin::command::{
    CommandExecutor, CommandResult, CommandSender,
    args::{Arg, ConsumedArgs, players::PlayersArgumentConsumer},
    tree::CommandTree,
    tree::builder::{argument, require},
};
use pumpkin::server::Server;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::branding;

const ARG_TARGET: &str = "target";

struct SelfExecutor;
struct TargetExecutor;

impl CommandExecutor for SelfExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(player) = sender.as_player() else {
                return Ok(());
            };

            player.hunger_manager.restart();
            player.send_health().await;
            let msg = branding::brand(
                TextComponent::text("Hunger restored.").color_named(NamedColor::Green),
            );
            sender.send_message(msg).await;
            Ok(())
        })
    }
}

impl CommandExecutor for TargetExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Players(targets)) = args.get(ARG_TARGET) else {
                return Ok(());
            };

            let target_count = targets.len();
            let first_name = targets.get(0).map(|t| t.gameprofile.name.clone());

            for target in targets {
                target.hunger_manager.restart();
                target.send_health().await;
                let msg = branding::brand(
                    TextComponent::text("Your hunger has been restored.")
                        .color_named(NamedColor::Green),
                );
                target.send_system_message(&msg).await;
            }

            let msg = if target_count == 1 {
                let name = first_name.unwrap_or_else(|| "player".to_string());
                branding::brand(
                    TextComponent::text(format!("Fed {name}."))
                        .color_named(NamedColor::Green),
                )
            } else {
                branding::brand(
                    TextComponent::text(format!("Fed {target_count} players."))
                        .color_named(NamedColor::Green),
                )
            };
            sender.send_message(msg).await;
            Ok(())
        })
    }
}

pub fn feed_command_tree() -> CommandTree {
    CommandTree::new(["feed"], "Restore a player's hunger.")
        .then(require(|sender| sender.is_player()).execute(SelfExecutor))
        .then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(TargetExecutor))
}
