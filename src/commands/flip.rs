use rand::random;

use pumpkin::command::{
    args::ConsumedArgs,
    CommandExecutor, CommandResult, CommandSender, tree::CommandTree,
};
use pumpkin::server::Server;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::branding;

struct FlipExecutor;

impl CommandExecutor for FlipExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let result = if random::<bool>() { "Heads" } else { "Tails" };
            let body = TextComponent::text(format!("Coin flip: {result}"))
                .color_named(NamedColor::White);
            sender.send_message(branding::brand(body)).await;
            Ok(())
        })
    }
}

pub fn flip_command_tree() -> CommandTree {
    CommandTree::new(["flip"], "Flip a coin.")
        .execute(FlipExecutor)
}
