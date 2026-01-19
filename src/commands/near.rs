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

const NEAR_RADIUS: f64 = 200.0;
const MAX_LIST: usize = 10;

struct NearExecutor;

impl CommandExecutor for NearExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(player) = sender.as_player() else {
                return Ok(());
            };
            let origin = player.position();
            let world = &player.living_entity.entity.world;

            let mut candidates = collect_players(server).await;
            candidates.retain(|p| Arc::ptr_eq(&p.living_entity.entity.world, world));

            let mut nearby = Vec::new();
            for other in candidates {
                if other.gameprofile.id == player.gameprofile.id {
                    continue;
                }
                let pos = other.position();
                let dx = pos.x - origin.x;
                let dy = pos.y - origin.y;
                let dz = pos.z - origin.z;
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                if dist <= NEAR_RADIUS {
                    nearby.push((dist, other.gameprofile.name.clone()));
                }
            }

            if nearby.is_empty() {
                let msg = branding::brand(
                    TextComponent::text("No nearby players.")
                        .color_named(NamedColor::Yellow),
                );
                sender.send_message(msg).await;
                return Ok(());
            }

            nearby.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            let mut lines = String::from("Nearby players:\n");
            for (dist, name) in nearby.into_iter().take(MAX_LIST) {
                lines.push_str(&format!("{name} ({dist:.1}m)\n"));
            }

            let body = TextComponent::text(lines.trim_end().to_string())
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

pub fn near_command_tree() -> CommandTree {
    CommandTree::new(["near"], "List nearby players.")
        .execute(NearExecutor)
}
