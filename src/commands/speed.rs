use pumpkin::command::{
    CommandExecutor, CommandResult, CommandSender,
    args::{Arg, ConsumedArgs, FindArg, bounded_num::BoundedNumArgumentConsumer, players::PlayersArgumentConsumer},
    tree::CommandTree,
    tree::builder::argument,
};
use pumpkin::server::Server;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::branding;

const ARG_MODE: &str = "mode";
const ARG_VALUE: &str = "value";
const ARG_TARGET: &str = "target";

const WALK_BASE: f32 = 0.1;
const FLY_BASE: f32 = 0.05;

fn value_consumer() -> BoundedNumArgumentConsumer<f32> {
    BoundedNumArgumentConsumer::new().name(ARG_VALUE).min(0.0).max(10.0)
}

struct SpeedExecutor;

impl CommandExecutor for SpeedExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(mode)) = args.get(ARG_MODE) else {
                return Ok(());
            };
            let Ok(Ok(value)) = BoundedNumArgumentConsumer::<f32>::find_arg(args, ARG_VALUE) else {
                return Ok(());
            };

            let is_fly = match mode.to_ascii_lowercase().as_str() {
                "walk" => false,
                "fly" => true,
                _ => {
                    let msg = branding::brand(
                        TextComponent::text("Mode must be walk or fly.")
                            .color_named(NamedColor::Yellow),
                    );
                    sender.send_message(msg).await;
                    return Ok(());
                }
            };

            let targets = if let Some(Arg::Players(players)) = args.get(ARG_TARGET) {
                players.clone()
            } else {
                let Some(player) = sender.as_player() else {
                    let msg = branding::brand(
                        TextComponent::text("You must specify a target from console.")
                            .color_named(NamedColor::Yellow),
                    );
                    sender.send_message(msg).await;
                    return Ok(());
                };
                vec![player]
            };

            let mut count = 0usize;
            for target in targets.iter() {
                let mut abilities = target.abilities.lock().await;
                if is_fly {
                    abilities.fly_speed = FLY_BASE * value;
                } else {
                    abilities.walk_speed = WALK_BASE * value;
                }
                drop(abilities);
                target.send_abilities_update().await;
                count += 1;
            }

            let mode_label = if is_fly { "fly" } else { "walk" };
            let msg = if count == 1 {
                branding::brand(
                    TextComponent::text(format!("Set {mode_label} speed to {value}."))
                        .color_named(NamedColor::Green),
                )
            } else {
                branding::brand(
                    TextComponent::text(format!(
                        "Set {mode_label} speed to {value} for {count} players."
                    ))
                    .color_named(NamedColor::Green),
                )
            };
            sender.send_message(msg).await;
            Ok(())
        })
    }
}

pub fn speed_command_tree() -> CommandTree {
    CommandTree::new(["speed"], "Set walk or fly speed.")
        .then(
            argument(ARG_MODE, pumpkin::command::args::simple::SimpleArgConsumer)
                .then(
                    argument(ARG_VALUE, value_consumer())
                        .execute(SpeedExecutor)
                        .then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(SpeedExecutor)),
                ),
        )
}
