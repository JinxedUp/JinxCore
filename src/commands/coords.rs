use pumpkin::command::{
    args::ConsumedArgs,
    CommandExecutor, CommandResult, CommandSender, tree::CommandTree,
};
use pumpkin::server::Server;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::branding;

struct CoordsExecutor;

impl CommandExecutor for CoordsExecutor {
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
            let pos = player.position();
            let world = player.world().dimension.minecraft_name;
            let body = TextComponent::text(format!(
                "World: {world}\nX: {:.2}\nY: {:.2}\nZ: {:.2}",
                pos.x, pos.y, pos.z
            ))
            .color_named(NamedColor::White);
            sender.send_message(branding::brand(body)).await;
            Ok(())
        })
    }
}

pub fn coords_command_tree() -> CommandTree {
    CommandTree::new(["coords"], "Show your coordinates.")
        .execute(CoordsExecutor)
}
