use std::sync::Arc;

use pumpkin::command::{
    CommandExecutor, CommandResult, CommandSender,
    args::{Arg, ConsumedArgs, players::PlayersArgumentConsumer},
    tree::CommandTree,
    tree::builder::{argument, require},
};
use pumpkin::server::Server;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::{PluginState, branding};
use crate::seen::format_duration;

const ARG_TARGET: &str = "target";

struct SelfExecutor {
    state: Arc<PluginState>,
}

struct TargetExecutor {
    state: Arc<PluginState>,
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
            let total = current_playtime_secs(&self.state, player.gameprofile.id);
            let body = TextComponent::text(format!(
                "Your playtime: {}",
                format_duration(std::time::Duration::from_secs(total))
            ))
            .color_named(NamedColor::Green);
            sender.send_message(branding::brand(body)).await;
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
            if targets.len() != 1 {
                let msg = branding::brand(
                    TextComponent::text("Please specify a single player.")
                        .color_named(NamedColor::Yellow),
                );
                sender.send_message(msg).await;
                return Ok(());
            }
            let target = &targets[0];
            let total = current_playtime_secs(&self.state, target.gameprofile.id);
            let body = TextComponent::text(format!(
                "{}'s playtime: {}",
                target.gameprofile.name,
                format_duration(std::time::Duration::from_secs(total))
            ))
            .color_named(NamedColor::Green);
            sender.send_message(branding::brand(body)).await;
            Ok(())
        })
    }
}

fn current_playtime_secs(state: &PluginState, uuid: uuid::Uuid) -> u64 {
    let base = {
        let totals = state.playtime_total_secs.read().unwrap();
        totals.get(&uuid).copied().unwrap_or(0)
    };
    let session = {
        let sessions = state.playtime_session_start.read().unwrap();
        sessions.get(&uuid).map(|start| start.elapsed().as_secs())
    };
    base + session.unwrap_or(0)
}

pub fn playtime_command_tree(state: Arc<PluginState>) -> CommandTree {
    CommandTree::new(["playtime"], "Show playtime.")
        .then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(TargetExecutor { state: Arc::clone(&state) }))
        .then(require(|sender| sender.is_player()).execute(SelfExecutor { state }))
}
