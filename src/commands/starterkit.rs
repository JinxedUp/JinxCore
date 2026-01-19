use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use pumpkin::command::{
    args::ConsumedArgs,
    CommandExecutor, CommandResult, CommandSender, tree::CommandTree,
    tree::builder::require,
};
use pumpkin::entity::player::Player;
use pumpkin::server::Server;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item::Item;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use pumpkin_world::item::ItemStack;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{branding, PluginState};

const STARTER_KIT_FILE_NAME: &str = "starterkit.yml";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StarterKitFile {
    items: Vec<StarterKitItem>,
    equipment: Vec<StarterKitEquipment>,
    claimed: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StarterKitItem {
    slot: usize,
    id: String,
    count: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StarterKitEquipment {
    slot: String,
    id: String,
    count: u8,
}

impl Default for StarterKitFile {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            equipment: Vec::new(),
            claimed: Vec::new(),
        }
    }
}

struct StarterKitExecutor {
    state: Arc<PluginState>,
}

struct DeleteStarterKitExecutor {
    state: Arc<PluginState>,
}

fn starterkit_path(data_dir: &Path) -> PathBuf {
    data_dir.join(STARTER_KIT_FILE_NAME)
}

fn load_starterkit(path: &Path) -> Result<StarterKitFile, String> {
    if !path.exists() {
        return Ok(StarterKitFile::default());
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let file = serde_yaml::from_str::<StarterKitFile>(&content).map_err(|e| e.to_string())?;
    Ok(file)
}

fn save_starterkit(path: &Path, file: &StarterKitFile) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_yaml::to_string(file).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

fn equipment_slot_name(slot: &EquipmentSlot) -> &'static str {
    match slot {
        EquipmentSlot::MainHand(_) => "mainhand",
        EquipmentSlot::OffHand(_) => "offhand",
        EquipmentSlot::Feet(_) => "feet",
        EquipmentSlot::Legs(_) => "legs",
        EquipmentSlot::Chest(_) => "chest",
        EquipmentSlot::Head(_) => "head",
        EquipmentSlot::Body(_) => "body",
        EquipmentSlot::Saddle(_) => "saddle",
    }
}

fn equipment_slot_from_name(name: &str) -> Option<EquipmentSlot> {
    match name.to_ascii_lowercase().as_str() {
        "mainhand" => Some(EquipmentSlot::MAIN_HAND),
        "offhand" => Some(EquipmentSlot::OFF_HAND),
        "feet" => Some(EquipmentSlot::FEET),
        "legs" => Some(EquipmentSlot::LEGS),
        "chest" => Some(EquipmentSlot::CHEST),
        "head" => Some(EquipmentSlot::HEAD),
        "body" => Some(EquipmentSlot::BODY),
        "saddle" => Some(EquipmentSlot::SADDLE),
        _ => None,
    }
}

fn item_from_id(id: &str, count: u8) -> Option<ItemStack> {
    let registry_key = id.strip_prefix("minecraft:").unwrap_or(id);
    let item = Item::from_registry_key(registry_key)?;
    Some(ItemStack::new(count, item))
}

impl CommandExecutor for StarterKitExecutor {
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

            let inventory = player.inventory();
            let mut items = Vec::new();
            for (index, slot) in inventory.main_inventory.iter().enumerate() {
                let slot_lock = slot.lock().await;
                if slot_lock.is_empty() {
                    continue;
                }
                let id = format!("minecraft:{}", slot_lock.item.registry_key);
                items.push(StarterKitItem {
                    slot: index,
                    id,
                    count: slot_lock.item_count,
                });
            }

            let mut equipment_items = Vec::new();
            let equipment = inventory.entity_equipment.lock().await;
            for (slot, stack) in equipment.equipment.iter() {
                let stack_lock = stack.lock().await;
                if stack_lock.is_empty() {
                    continue;
                }
                let id = format!("minecraft:{}", stack_lock.item.registry_key);
                equipment_items.push(StarterKitEquipment {
                    slot: equipment_slot_name(slot).to_string(),
                    id,
                    count: stack_lock.item_count,
                });
            }
            drop(equipment);

            let path = starterkit_path(&self.state.data_dir);
            let mut file = load_starterkit(&path).unwrap_or_default();
            file.items = items;
            file.equipment = equipment_items;

            if let Err(err) = save_starterkit(&path, &file) {
                let msg = branding::brand(
                    TextComponent::text(format!("Failed to save starterkit.yml: {err}"))
                        .color_named(NamedColor::Red),
                );
                sender.send_message(msg).await;
                return Ok(());
            }

            let msg = branding::brand(
                TextComponent::text("Starter kit saved.")
                    .color_named(NamedColor::Green),
            );
            sender.send_message(msg).await;
            Ok(())
        })
    }
}

impl CommandExecutor for DeleteStarterKitExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let path = starterkit_path(&self.state.data_dir);
            if path.exists() {
                if let Err(err) = fs::remove_file(&path) {
                    let msg = branding::brand(
                        TextComponent::text(format!("Failed to delete starterkit.yml: {err}"))
                            .color_named(NamedColor::Red),
                    );
                    sender.send_message(msg).await;
                    return Ok(());
                }
                let msg = branding::brand(
                    TextComponent::text("Starter kit deleted.")
                        .color_named(NamedColor::Green),
                );
                sender.send_message(msg).await;
            } else {
                let msg = branding::brand(
                    TextComponent::text("No starter kit found.")
                        .color_named(NamedColor::Yellow),
                );
                sender.send_message(msg).await;
            }
            Ok(())
        })
    }
}

pub async fn apply_starterkit(player: &Arc<Player>, data_dir: &Path) {
    let path = starterkit_path(data_dir);
    let mut file = match load_starterkit(&path) {
        Ok(file) => file,
        Err(_) => return,
    };
    if file.items.is_empty() && file.equipment.is_empty() {
        return;
    }
    if file.claimed.contains(&player.gameprofile.id) {
        return;
    }

    let inventory = player.inventory();
    let mut applied = 0usize;
    let mut skipped = 0usize;

    for entry in &file.items {
        if entry.slot >= inventory.main_inventory.len() {
            skipped += 1;
            continue;
        }
        let Some(stack) = item_from_id(&entry.id, entry.count) else {
            skipped += 1;
            continue;
        };
        let mut slot_lock = inventory.main_inventory[entry.slot].lock().await;
        *slot_lock = stack;
        applied += 1;
    }

    if !file.equipment.is_empty() {
        let mut equipment = inventory.entity_equipment.lock().await;
        for entry in &file.equipment {
            let Some(slot) = equipment_slot_from_name(&entry.slot) else {
                skipped += 1;
                continue;
            };
            let Some(stack) = item_from_id(&entry.id, entry.count) else {
                skipped += 1;
                continue;
            };
            equipment.put(&slot, stack).await;
            applied += 1;
        }
    }

    if applied == 0 {
        return;
    }

    file.claimed.push(player.gameprofile.id);
    let _ = save_starterkit(&path, &file);

    let msg = if skipped > 0 {
        branding::brand(
            TextComponent::text(format!(
                "Starter kit received ({applied} items, {skipped} skipped)."
            ))
            .color_named(NamedColor::Green),
        )
    } else {
        branding::brand(
            TextComponent::text(format!("Starter kit received ({applied} items)."))
                .color_named(NamedColor::Green),
        )
    };
    player.send_system_message(&msg).await;
}

pub fn starterkit_command_tree(state: Arc<PluginState>) -> CommandTree {
    CommandTree::new(["starterkit"], "Set the starter kit from your inventory.")
        .then(require(|sender| sender.is_player()).execute(StarterKitExecutor { state }))
}

pub fn delstarterkit_command_tree(state: Arc<PluginState>) -> CommandTree {
    CommandTree::new(["delstarterkit"], "Delete the starter kit.")
        .execute(DeleteStarterKitExecutor { state })
}
