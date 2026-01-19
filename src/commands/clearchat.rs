use std::collections::HashSet;

use pumpkin::command::{
    args::ConsumedArgs,
    CommandExecutor, CommandResult, CommandSender, tree::CommandTree,
};
use pumpkin::entity::player::Player;
use pumpkin::server::Server;
use pumpkin_util::text::TextComponent;

const CLEAR_LINES: usize = 100;

struct ClearChatExecutor;

impl CommandExecutor for ClearChatExecutor {
    fn execute<'a>(
        &'a self,
        _sender: &'a CommandSender,
        server: &'a Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let players = collect_players(server).await;
            let blank = TextComponent::text("");
            for _ in 0..CLEAR_LINES {
                for player in &players {
                    player.send_system_message(&blank).await;
                }
            }
            Ok(())
        })
    }
}

async fn collect_players(server: &Server) -> Vec<std::sync::Arc<Player>> {
    let mut players = Vec::new();
    let mut seen = HashSet::new();

    for world in server.worlds.read().await.iter() {
        for (uuid, player) in world.players.read().await.iter() {
            if seen.insert(*uuid) {
                players.push(std::sync::Arc::clone(player));
            }
        }
    }

    players
}

pub fn clearchat_command_tree() -> CommandTree {
    CommandTree::new(["clearchat"], "Clear chat for all players.")
        .execute(ClearChatExecutor)
}
