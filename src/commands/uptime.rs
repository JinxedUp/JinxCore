use std::sync::Arc;
use std::time::Instant;

use pumpkin::command::{
    CommandExecutor, CommandResult, CommandSender, args::ConsumedArgs, tree::CommandTree,
};
use pumpkin::server::Server;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::branding;
use crate::PluginState;

struct UptimeExecutor {
    state: Arc<PluginState>,
}

impl UptimeExecutor {
    fn format_duration(start_time: Instant) -> String {
        let elapsed = start_time.elapsed();
        let total_secs = elapsed.as_secs();
        let days = total_secs / 86_400;
        let hours = (total_secs % 86_400) / 3_600;
        let minutes = (total_secs % 3_600) / 60;
        let seconds = total_secs % 60;

        if days > 0 {
            format!("{days}d {hours}h {minutes}m {seconds}s")
        } else if hours > 0 {
            format!("{hours}h {minutes}m {seconds}s")
        } else if minutes > 0 {
            format!("{minutes}m {seconds}s")
        } else {
            format!("{seconds}s")
        }
    }
}

impl CommandExecutor for UptimeExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let uptime = Self::format_duration(self.state.start_time);
            let message = TextComponent::text(format!("Uptime: {uptime}"))
                .color_named(NamedColor::Aqua);
            sender.send_message(branding::brand(message)).await;
            Ok(())
        })
    }
}

pub fn uptime_command_tree(state: Arc<PluginState>) -> CommandTree {
    CommandTree::new(["uptime"], "Show server uptime.").execute(UptimeExecutor { state })
}
