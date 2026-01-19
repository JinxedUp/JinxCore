use pumpkin::command::{
    args::ConsumedArgs,
    CommandExecutor, CommandResult, CommandSender, tree::CommandTree,
};
use pumpkin::server::Server;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::branding;

struct MeExecutor;

impl CommandExecutor for MeExecutor {
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
            let uuid = player.gameprofile.id;
            let gm = format!("{:?}", player.gamemode.load()).to_lowercase();
            let dim = player.world().dimension.minecraft_name;
            let pos = player.position();
            let address = player.client.address().await;
            let body = TextComponent::text(format!(
                "Player: {}\nUUID: {}\nGamemode: {}\nWorld: {}\nX: {:.2}\nY: {:.2}\nZ: {:.2}\nAddress: {}",
                player.gameprofile.name,
                uuid,
                gm,
                dim,
                pos.x,
                pos.y,
                pos.z,
                address
            ))
            .color_named(NamedColor::White);
            sender.send_message(branding::brand(body)).await;
            Ok(())
        })
    }
}

pub fn me_command_tree() -> CommandTree {
    CommandTree::new(["me", "whoami"], "Show your player info.")
        .execute(MeExecutor)
}
