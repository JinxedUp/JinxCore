use pumpkin::command::{
    args::ConsumedArgs,
    CommandExecutor, CommandResult, CommandSender, tree::CommandTree,
};
use pumpkin::server::Server;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::branding;

const DAY_TIME: i64 = 1000;
const NIGHT_TIME: i64 = 13000;

struct DayExecutor;
struct NightExecutor;

impl CommandExecutor for DayExecutor {
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
            let world = player.world();
            let mut time = world.level_time.lock().await;
            time.set_time(DAY_TIME);
            time.send_time(&world).await;
            let msg = branding::brand(
                TextComponent::text("Time set to day.")
                    .color_named(NamedColor::Green),
            );
            sender.send_message(msg).await;
            Ok(())
        })
    }
}

impl CommandExecutor for NightExecutor {
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
            let world = player.world();
            let mut time = world.level_time.lock().await;
            time.set_time(NIGHT_TIME);
            time.send_time(&world).await;
            let msg = branding::brand(
                TextComponent::text("Time set to night.")
                    .color_named(NamedColor::Green),
            );
            sender.send_message(msg).await;
            Ok(())
        })
    }
}

pub fn day_command_tree() -> CommandTree {
    CommandTree::new(["day"], "Set time to day.")
        .execute(DayExecutor)
}

pub fn night_command_tree() -> CommandTree {
    CommandTree::new(["night"], "Set time to night.")
        .execute(NightExecutor)
}
