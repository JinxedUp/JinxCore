use std::sync::atomic::Ordering;

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

async fn toggle_god(player: &pumpkin::entity::player::Player) -> bool {
    let mut abilities = player.abilities.lock().await;
    let enable = !abilities.invulnerable;
    abilities.invulnerable = enable;
    drop(abilities);
    player
        .living_entity
        .entity
        .invulnerable
        .store(enable, Ordering::Relaxed);
    player.send_abilities_update().await;
    enable
}

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

            let enabled = toggle_god(&player).await;
            let state = if enabled { "enabled" } else { "disabled" };
            let msg = branding::brand(
                TextComponent::text(format!("God mode {state}.")).color_named(NamedColor::Aqua),
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
                let enabled = toggle_god(target).await;
                let state = if enabled { "enabled" } else { "disabled" };
                let msg = branding::brand(
                    TextComponent::text(format!("God mode {state}."))
                        .color_named(NamedColor::Aqua),
                );
                target.send_system_message(&msg).await;
            }

            let msg = if target_count == 1 {
                let name = first_name.unwrap_or_else(|| "player".to_string());
                branding::brand(
                    TextComponent::text(format!("God mode toggled for {name}."))
                        .color_named(NamedColor::Aqua),
                )
            } else {
                branding::brand(
                    TextComponent::text(format!("God mode toggled for {target_count} players."))
                        .color_named(NamedColor::Aqua),
                )
            };
            sender.send_message(msg).await;
            Ok(())
        })
    }
}

pub fn god_command_tree() -> CommandTree {
    CommandTree::new(["god"], "Toggle god mode.")
        .then(require(|sender| sender.is_player()).execute(SelfExecutor))
        .then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(TargetExecutor))
}
