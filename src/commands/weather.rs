use pumpkin::command::{
    args::ConsumedArgs,
    CommandExecutor, CommandResult, CommandSender, tree::CommandTree,
};
use pumpkin::server::Server;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::branding;

const WEATHER_DURATION: i32 = 12_000;

struct RainExecutor;
struct ClearExecutor;
struct ThunderExecutor;

impl CommandExecutor for RainExecutor {
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
            let mut weather = world.weather.lock().await;
            weather
                .set_weather_parameters(&world, 0, WEATHER_DURATION, true, false)
                .await;
            let msg = branding::brand(
                TextComponent::text("Weather set to rain.")
                    .color_named(NamedColor::Green),
            );
            sender.send_message(msg).await;
            Ok(())
        })
    }
}

impl CommandExecutor for ClearExecutor {
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
            let mut weather = world.weather.lock().await;
            weather
                .set_weather_parameters(&world, WEATHER_DURATION, 0, false, false)
                .await;
            let msg = branding::brand(
                TextComponent::text("Weather cleared.")
                    .color_named(NamedColor::Green),
            );
            sender.send_message(msg).await;
            Ok(())
        })
    }
}

impl CommandExecutor for ThunderExecutor {
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
            let mut weather = world.weather.lock().await;
            weather
                .set_weather_parameters(&world, 0, WEATHER_DURATION, true, true)
                .await;
            let msg = branding::brand(
                TextComponent::text("Weather set to thunder.")
                    .color_named(NamedColor::Green),
            );
            sender.send_message(msg).await;
            Ok(())
        })
    }
}

pub fn rain_command_tree() -> CommandTree {
    CommandTree::new(["rain"], "Set weather to rain.")
        .execute(RainExecutor)
}

pub fn clear_command_tree() -> CommandTree {
    CommandTree::new(["clear"], "Clear weather.")
        .execute(ClearExecutor)
}

pub fn thunder_command_tree() -> CommandTree {
    CommandTree::new(["thunder"], "Set weather to thunder.")
        .execute(ThunderExecutor)
}
