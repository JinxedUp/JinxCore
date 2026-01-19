use pumpkin::command::{
    CommandExecutor, CommandResult, CommandSender, args::ConsumedArgs, tree::CommandTree,
};
use pumpkin::server::Server;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::branding;

struct TpsExecutor;

impl CommandExecutor for TpsExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let avg_nanos = server.get_average_tick_time_nanos();
            if avg_nanos <= 0 {
                sender
                    .send_message(branding::brand(
                        TextComponent::text("TPS: N/A").color_named(NamedColor::Red),
                    ))
                    .await;
                return Ok(());
            }

            let mspt = avg_nanos as f64 / 1_000_000.0;
            let tps = 1_000_000_000.0 / avg_nanos as f64;
            let target = server.tick_rate_manager.tickrate() as f64;
            let tps_display = tps.min(target);

            let message = TextComponent::text(format!(
                "TPS: {tps_display:.2} (MSPT: {mspt:.2}) Target: {target:.1}"
            ))
            .color_named(NamedColor::Green);
            sender.send_message(branding::brand(message)).await;

            Ok(())
        })
    }
}

pub fn tps_command_tree() -> CommandTree {
    CommandTree::new(["tps"], "Show server TPS.").execute(TpsExecutor)
}
