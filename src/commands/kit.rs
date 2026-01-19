use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use pumpkin::command::{
    CommandExecutor, CommandResult, CommandSender,
    args::{Arg, ConsumedArgs, FindArg, bounded_num::BoundedNumArgumentConsumer},
    tree::CommandTree,
    tree::builder::{argument, require},
};
use pumpkin::server::Server;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item::Item;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use pumpkin_world::item::ItemStack;
use serde::{Deserialize, Serialize};

use crate::{PluginState, branding};

const ARG_NAME: &str = "name";
const ARG_DELAY: &str = "delay";
const KITS_FILE_NAME: &str = "kits.yml";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KitFile {
    kits: HashMap<String, KitDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KitDefinition {
    delay_seconds: u64,
    items: Vec<KitItem>,
    equipment: Vec<KitEquipment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KitItem {
    slot: usize,
    id: String,
    count: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KitEquipment {
    slot: String,
    id: String,
    count: u8,
}

impl Default for KitFile {
    fn default() -> Self {
        Self {
            kits: HashMap::new(),
        }
    }
}

struct CreateKitExecutor {
    state: Arc<PluginState>,
}

struct KitExecutor {
    state: Arc<PluginState>,
}

fn kits_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join(KITS_FILE_NAME)
}

fn load_kits(path: &Path) -> Result<KitFile, String> {
    if !path.exists() {
        return Ok(KitFile::default());
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let file = serde_yaml::from_str::<KitFile>(&content).map_err(|e| e.to_string())?;
    Ok(file)
}

fn save_kits(path: &Path, file: &KitFile) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_yaml::to_string(file).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

fn delay_consumer() -> BoundedNumArgumentConsumer<i64> {
    BoundedNumArgumentConsumer::new().name(ARG_DELAY).min(0)
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

impl CommandExecutor for CreateKitExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(player) = sender.as_player() else {
                return Ok(());
            };
            let Some(Arg::Simple(name)) = args.get(ARG_NAME) else {
                return Ok(());
            };
            let Ok(Ok(delay)) = BoundedNumArgumentConsumer::<i64>::find_arg(args, ARG_DELAY) else {
                return Ok(());
            };

            let kit_name = name.to_ascii_lowercase();
            let inventory = player.inventory();
            let mut items = Vec::new();
            for (index, slot) in inventory.main_inventory.iter().enumerate() {
                let slot_lock = slot.lock().await;
                if slot_lock.is_empty() {
                    continue;
                }
                let id = format!("minecraft:{}", slot_lock.item.registry_key);
                items.push(KitItem {
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
                equipment_items.push(KitEquipment {
                    slot: equipment_slot_name(slot).to_string(),
                    id,
                    count: stack_lock.item_count,
                });
            }
            drop(equipment);

            let path = kits_path(&self.state.data_dir);
            let mut file = load_kits(&path).unwrap_or_default();
            file.kits.insert(
                kit_name.clone(),
                KitDefinition {
                    delay_seconds: delay as u64,
                    items,
                    equipment: equipment_items,
                },
            );

            if let Err(err) = save_kits(&path, &file) {
                let msg = branding::brand(
                    TextComponent::text(format!("Failed to save kits.yml: {err}"))
                        .color_named(NamedColor::Red),
                );
                sender.send_message(msg).await;
                return Ok(());
            }

            let msg = branding::brand(
                TextComponent::text(format!("Created kit {kit_name}."))
                    .color_named(NamedColor::Green),
            );
            sender.send_message(msg).await;
            Ok(())
        })
    }
}

impl CommandExecutor for KitExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(player) = sender.as_player() else {
                return Ok(());
            };
            let Some(Arg::Simple(name)) = args.get(ARG_NAME) else {
                return Ok(());
            };
            let kit_name = name.to_ascii_lowercase();

            let path = kits_path(&self.state.data_dir);
            let file = match load_kits(&path) {
                Ok(value) => value,
                Err(err) => {
                    let msg = branding::brand(
                        TextComponent::text(format!("Failed to read kits.yml: {err}"))
                            .color_named(NamedColor::Red),
                    );
                    sender.send_message(msg).await;
                    return Ok(());
                }
            };
            let Some(kit) = file.kits.get(&kit_name) else {
                let msg = branding::brand(
                    TextComponent::text("That kit does not exist.")
                        .color_named(NamedColor::Yellow),
                );
                sender.send_message(msg).await;
                return Ok(());
            };

            if kit.delay_seconds > 0 {
                let mut remaining = None;
                {
                    let mut cooldowns = self.state.kit_cooldowns.write().unwrap();
                    let per_player = cooldowns
                        .entry(player.gameprofile.id)
                        .or_insert_with(HashMap::new);
                    if let Some(last_used) = per_player.get(&kit_name) {
                        let elapsed = last_used.elapsed();
                        if elapsed < Duration::from_secs(kit.delay_seconds) {
                            remaining = Some(kit.delay_seconds - elapsed.as_secs());
                        }
                    }
                    if remaining.is_none() {
                        per_player.insert(kit_name.clone(), Instant::now());
                    }
                }
                if let Some(remaining) = remaining {
                    let msg = branding::brand(
                        TextComponent::text(format!(
                            "Kit cooldown: {remaining}s remaining."
                        ))
                        .color_named(NamedColor::Yellow),
                    );
                    sender.send_message(msg).await;
                    return Ok(());
                }
            }

            let inventory = player.inventory();
            let mut applied = 0usize;
            let mut skipped = 0usize;

            for entry in &kit.items {
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

            if !kit.equipment.is_empty() {
                let mut equipment = inventory.entity_equipment.lock().await;
                for entry in &kit.equipment {
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

            let msg = if skipped > 0 {
                branding::brand(
                    TextComponent::text(format!(
                        "Loaded kit {kit_name} ({applied} items, {skipped} skipped)."
                    ))
                    .color_named(NamedColor::Green),
                )
            } else {
                branding::brand(
                    TextComponent::text(format!("Loaded kit {kit_name} ({applied} items)."))
                        .color_named(NamedColor::Green),
                )
            };
            sender.send_message(msg).await;
            Ok(())
        })
    }
}

pub fn createkit_command_tree(state: Arc<PluginState>) -> CommandTree {
    CommandTree::new(["createkit"], "Create a kit from your inventory.")
        .then(
            require(|sender| sender.is_player())
                .then(
                    argument(ARG_NAME, pumpkin::command::args::simple::SimpleArgConsumer)
                        .then(argument(ARG_DELAY, delay_consumer()).execute(CreateKitExecutor { state })),
                ),
        )
}

pub fn kit_command_tree(state: Arc<PluginState>) -> CommandTree {
    CommandTree::new(["kit"], "Receive a kit.")
        .then(
            require(|sender| sender.is_player())
                .then(
                    argument(ARG_NAME, pumpkin::command::args::simple::SimpleArgConsumer)
                        .execute(KitExecutor { state }),
                ),
        )
}
