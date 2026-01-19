use pumpkin::command::{
    CommandExecutor, CommandResult, CommandSender,
    args::{Arg, ConsumedArgs, players::PlayersArgumentConsumer},
    tree::CommandTree,
    tree::builder::{argument, require},
};
use pumpkin::command::dispatcher::CommandError;
use pumpkin::server::Server;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use pumpkin_world::item::ItemStack;

use crate::branding;

const ARG_TARGET: &str = "target";

async fn clear_player(target: &pumpkin::entity::player::Player) -> u64 {
    let inventory = target.inventory();
    let mut count: u64 = 0;
    for slot in &inventory.main_inventory {
        let mut slot_lock = slot.lock().await;
        count += u64::from(slot_lock.item_count);
        *slot_lock = ItemStack::EMPTY.clone();
    }

    let entity_equipment_lock = inventory.entity_equipment.lock().await;
    for slot in entity_equipment_lock.equipment.values() {
        let mut slot_lock = slot.lock().await;
        if slot_lock.is_empty() {
            continue;
        }
        count += 1u64;
        *slot_lock = ItemStack::EMPTY.clone();
    }

    count
}

struct SelfExecutor;
struct TargetExecutor;

impl CommandExecutor for SelfExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let target = sender.as_player().ok_or(CommandError::InvalidRequirement)?;
            let count = clear_player(&target).await;
            let msg = branding::brand(
                TextComponent::text(format!("Cleared {count} items from your inventory."))
                    .color_named(NamedColor::Green),
            );
            sender.send_message(msg).await;
            Ok(())
        })
    }
}

impl CommandExecutor for TargetExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Players(targets)) = args.get(ARG_TARGET) else {
                return Ok(());
            };

            let target_count = targets.len();
            let first_name = targets.get(0).map(|t| t.gameprofile.name.clone());

            let mut item_count = 0;
            for target in targets {
                item_count += clear_player(target).await;
                let msg = branding::brand(
                    TextComponent::text("Your inventory was cleared.")
                        .color_named(NamedColor::Yellow),
                );
                target.send_system_message(&msg).await;
            }

            let msg = if target_count == 1 {
                let name = first_name.unwrap_or_else(|| "player".to_string());
                branding::brand(
                    TextComponent::text(format!(
                        "Cleared {item_count} items from {name}."
                    ))
                    .color_named(NamedColor::Green),
                )
            } else {
                branding::brand(
                    TextComponent::text(format!(
                        "Cleared {item_count} items from {target_count} players."
                    ))
                    .color_named(NamedColor::Green),
                )
            };
            sender.send_message(msg).await;
            Ok(())
        })
    }
}

pub fn clearinv_command_tree() -> CommandTree {
    CommandTree::new(["clearinv"], "Clear inventories.")
        .then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(TargetExecutor))
        .then(require(|sender| sender.is_player()).execute(SelfExecutor))
}
