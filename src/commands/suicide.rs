use pumpkin::command::{
    args::ConsumedArgs,
    CommandExecutor, CommandResult, CommandSender, tree::CommandTree,
};
use pumpkin::server::Server;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::branding;

struct SuicideExecutor;

impl CommandExecutor for SuicideExecutor {
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
            player.set_health(0.0).await;
            let msg = branding::brand(
                TextComponent::text("You died.")
                    .color_named(NamedColor::Gray),
            );
            sender.send_message(msg).await;
            Ok(())
        })
    }
}

pub fn suicide_command_tree() -> CommandTree {
    CommandTree::new(["suicide"], "Kill yourself.")
        .execute(SuicideExecutor)
}
