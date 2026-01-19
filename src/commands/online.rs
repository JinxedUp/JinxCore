use std::collections::HashSet;
use std::sync::Arc;

use pumpkin::command::{
    args::ConsumedArgs,
    CommandExecutor, CommandResult, CommandSender, tree::CommandTree,
};
use pumpkin::entity::player::Player;
use pumpkin::server::Server;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::branding;

struct OnlineExecutor;

impl CommandExecutor for OnlineExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let players = collect_players(server).await;
            let mut names: Vec<String> = players
                .into_iter()
                .map(|player| player.gameprofile.name.clone())
                .collect();
            names.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

            let count = names.len();
            let max = server.basic_config.max_players;
            let max_display = if max == 0 {
                "inf".to_string()
            } else {
                max.to_string()
            };
            let list = if names.is_empty() {
                "None".to_string()
            } else {
                names.join(", ")
            };

            let body = TextComponent::text(format!(
                "Online ({count}/{max_display}):\n{list}"
            ))
            .color_named(NamedColor::White);
            sender.send_message(branding::brand(body)).await;
            Ok(())
        })
    }
}

async fn collect_players(server: &Server) -> Vec<Arc<Player>> {
    let mut players = Vec::new();
    let mut seen = HashSet::new();

    for world in server.worlds.read().await.iter() {
        for (uuid, player) in world.players.read().await.iter() {
            if seen.insert(*uuid) {
                players.push(Arc::clone(player));
            }
        }
    }

    players
}

pub fn online_command_tree() -> CommandTree {
    CommandTree::new(["online"], "List online players.")
        .execute(OnlineExecutor)
}
