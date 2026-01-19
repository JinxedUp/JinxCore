use pumpkin::command::{
    CommandExecutor, CommandResult, CommandSender,
    args::{Arg, ConsumedArgs, players::PlayersArgumentConsumer},
    tree::CommandTree,
    tree::builder::{argument, require},
};
use pumpkin::server::Server;
use pumpkin_util::GameMode;
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

            if player.gamemode.load() == GameMode::Spectator {
                let msg = branding::brand(
                    TextComponent::text("You are already in Spectator.")
                        .color_named(NamedColor::Yellow),
                );
                sender.send_message(msg).await;
                return Ok(());
            }
            player.set_gamemode(GameMode::Spectator).await;
            let msg = branding::brand(
                TextComponent::text("Gamemode set to Spectator.")
                    .color_named(NamedColor::Green),
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
                if target.gamemode.load() == GameMode::Spectator {
                    let msg = branding::brand(
                        TextComponent::text("You are already in Spectator.")
                            .color_named(NamedColor::Yellow),
                    );
                    target.send_system_message(&msg).await;
                } else {
                    target.set_gamemode(GameMode::Spectator).await;
                    let msg = branding::brand(
                        TextComponent::text("Your gamemode is now Spectator.")
                            .color_named(NamedColor::Green),
                    );
                    target.send_system_message(&msg).await;
                }
            }

            let msg = if target_count == 1 {
                let name = first_name.unwrap_or_else(|| "player".to_string());
                branding::brand(
                    TextComponent::text(format!("Set {name} to Spectator."))
                        .color_named(NamedColor::Green),
                )
            } else {
                branding::brand(
                    TextComponent::text(format!("Set {target_count} players to Spectator."))
                        .color_named(NamedColor::Green),
                )
            };
            sender.send_message(msg).await;
            Ok(())
        })
    }
}

pub fn gmsp_command_tree() -> CommandTree {
    CommandTree::new(["gmsp"], "Set Spectator mode.")
        .then(require(|sender| sender.is_player()).execute(SelfExecutor))
        .then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(TargetExecutor))
}

pub fn spectator_command_tree() -> CommandTree {
    CommandTree::new(["spectator"], "Set Spectator mode.")
        .then(require(|sender| sender.is_player()).execute(SelfExecutor))
        .then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(TargetExecutor))
}

pub fn sp_command_tree() -> CommandTree {
    CommandTree::new(["sp"], "Set Spectator mode.")
        .then(require(|sender| sender.is_player()).execute(SelfExecutor))
        .then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(TargetExecutor))
}
