use pumpkin::command::{
    args::ConsumedArgs,
    CommandExecutor, CommandResult, CommandSender,
};
use pumpkin::server::Server;
use pumpkin_util::text::{color::NamedColor, TextComponent};

use crate::branding;

pub struct CreditsExecutor;

impl CommandExecutor for CreditsExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let body = TextComponent::text("Made by Jinx, with a lot of love ❤")
                .color_named(NamedColor::Yellow);
            sender.send_message(branding::brand(body)).await;
            Ok(())
        })
    }
}

pub fn jinx_credits_command() -> CreditsExecutor {
    CreditsExecutor
}
