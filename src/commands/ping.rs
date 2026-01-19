use std::sync::atomic::Ordering;
use std::time::Instant;

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

            let ping_ms = estimate_ping(player.as_ref());
            let msg = branding::brand(
                TextComponent::text(format!("Your ping: {ping_ms}ms"))
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

            if target_count == 1 {
                let target = &targets[0];
                let ping_ms = estimate_ping(target.as_ref());
                let name = first_name.unwrap_or_else(|| "player".to_string());
                let msg = branding::brand(
                    TextComponent::text(format!("{name}'s ping: {ping_ms}ms"))
                        .color_named(NamedColor::Green),
                );
                sender.send_message(msg).await;
                return Ok(());
            }

            let msg = branding::brand(
                TextComponent::text("Please specify a single player.")
                    .color_named(NamedColor::Yellow),
            );
            sender.send_message(msg).await;
            Ok(())
        })
    }
}

fn estimate_ping(player: &pumpkin::entity::player::Player) -> u64 {
    let now = Instant::now();
    let last = player.last_keep_alive_time.load();
    let waiting = player.wait_for_keep_alive.load(Ordering::Relaxed);
    let mut ms = now.duration_since(last).as_millis() as u64;
    if !waiting {
        ms = ms.min(1000);
    }
    ms
}

pub fn ping_command_tree() -> CommandTree {
    CommandTree::new(["ping"], "Check ping.")
        .then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(TargetExecutor))
        .then(require(|sender| sender.is_player()).execute(SelfExecutor))
}
