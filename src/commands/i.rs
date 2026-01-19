use pumpkin::command::{
    args::{ConsumedArgs, Arg, message::MsgArgConsumer},
    CommandExecutor, CommandResult, CommandSender, tree::CommandTree,
    tree::builder::argument,
};
use pumpkin::server::Server;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::branding;

const ARG_CMD: &str = "cmd";

struct GiveAliasExecutor;

impl CommandExecutor for GiveAliasExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Msg(raw)) = args.get(ARG_CMD) else {
                let msg = branding::brand(
                    TextComponent::text("Usage: /i <targets> <item> [count]")
                        .color_named(NamedColor::Yellow),
                );
                sender.send_message(msg).await;
                return Ok(());
            };

            let command = format!("give {raw}");
            let dispatcher = server.command_dispatcher.read().await;
            dispatcher.handle_command(sender, server, &command).await;
            Ok(())
        })
    }
}

pub fn give_alias_command_tree() -> CommandTree {
    CommandTree::new(["i"], "Alias for /give.")
        .then(argument(ARG_CMD, MsgArgConsumer).execute(GiveAliasExecutor))
}
