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

struct WhoisExecutor {
    state: std::sync::Arc<PluginState>,
}

impl CommandExecutor for WhoisExecutor {
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
                let uuid = player.gameprofile.id;
                let gm = format!("{:?}", player.gamemode.load()).to_lowercase();
                let op = player.permission_lvl.load() as u8;
                let dim = player.world().dimension.minecraft_name;
                let address = player.client.address().await;
                let body = TextComponent::text(format!(
                    "Player: {name}\nUUID: {uuid}\nGamemode: {gm}\nOp level: {op}\nWorld: {dim}\nAddress: {address}"
                ))
                .color_named(NamedColor::White);
                sender.send_message(branding::brand(body)).await;
                return Ok(());
            }

            let entry = {
                let seen = self.state.seen.read().unwrap();
                find_by_name(&seen, name).cloned()
            };
            if let Some(entry) = entry {
                let elapsed =
                    SystemTime::now().duration_since(entry.last_seen).unwrap_or_default();
                let body = TextComponent::text(format!(
                    "Player: {}\nUUID: {}\nOnline: no\nLast seen: {} ago",
                    entry.name,
                    entry.uuid,
                    format_duration(elapsed)
                ))
                .color_named(NamedColor::Yellow);
                sender.send_message(branding::brand(body)).await;
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

pub fn whois_command_tree(state: std::sync::Arc<PluginState>) -> CommandTree {
    CommandTree::new(["whois"], "Show player info.")
        .then(argument(ARG_NAME, SimpleArgConsumer).execute(WhoisExecutor { state }))
}
