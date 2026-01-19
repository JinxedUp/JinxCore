use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use pumpkin::plugin::{
    BoxFuture, Context, EventHandler, EventPriority, Plugin, PluginFuture, PluginMetadata,
    PLUGIN_API_VERSION,
};
use pumpkin::plugin::events::player::player_chat::PlayerChatEvent;
use pumpkin::plugin::events::player::player_join::PlayerJoinEvent;
use pumpkin::plugin::events::player::player_leave::PlayerLeaveEvent;
use pumpkin::server::Server;
use pumpkin_util::text::TextComponent;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionLvl};

mod antispam;
mod branding;
mod chatfilter;
mod chatformat;
mod config;
mod commands;
mod scoreboard;
mod seen;
mod webhook;
mod metrics;
mod discord_bot;

use antispam::{AntiSpamHandler, PERMISSION_ANTISPAM_BYPASS};
use chatfilter::{ChatFilterHandler, PERMISSION_CHATFILTER_BYPASS};
use chatformat::ChatFormatHandler;
use config::Config;
use scoreboard::start_scoreboard_task;
use seen::{SeenEntry, update_on_join, update_on_leave};
use metrics::{start_system_sampler, SystemMetrics};
use discord_bot::{DiscordBridge, DiscordEvent, start_discord_bot, send_discord_event};
use webhook::{send_webhook, WebhookEvent};

const PERMISSION_ADMIN: &str = "JinxCore:admin";
const PERMISSION_GMC: &str = "JinxCore:gmc";
const PERMISSION_GMS: &str = "JinxCore:gms";
const PERMISSION_GMSP: &str = "JinxCore:gmsp";
const PERMISSION_GMA: &str = "JinxCore:gma";
const PERMISSION_HEAL: &str = "JinxCore:heal";
const PERMISSION_FEED: &str = "JinxCore:feed";
const PERMISSION_FLY: &str = "JinxCore:fly";
const PERMISSION_GOD: &str = "JinxCore:god";
const PERMISSION_SEEN: &str = "JinxCore:seen";
const PERMISSION_WHOIS: &str = "JinxCore:whois";
const PERMISSION_CLEARINV: &str = "JinxCore:clearinv";
const PERMISSION_RULES: &str = "JinxCore:rules";
const PERMISSION_DISCORD: &str = "JinxCore:discord";
const PERMISSION_WEBSITE: &str = "JinxCore:website";
const PERMISSION_STORE: &str = "JinxCore:store";
const PERMISSION_SOCIALS: &str = "JinxCore:socials";
const PERMISSION_SPEED: &str = "JinxCore:speed";
const PERMISSION_NEAR: &str = "JinxCore:near";
const PERMISSION_KIT: &str = "JinxCore:kit";
const PERMISSION_CREATEKIT: &str = "JinxCore:createkit";
const PERMISSION_SUICIDE: &str = "JinxCore:suicide";
const PERMISSION_PING: &str = "JinxCore:ping";
const PERMISSION_COORDS: &str = "JinxCore:coords";
const PERMISSION_PLAYTIME: &str = "JinxCore:playtime";
const PERMISSION_CLEARCHAT: &str = "JinxCore:clearchat";
const PERMISSION_ME: &str = "JinxCore:me";
const PERMISSION_DAY: &str = "JinxCore:day";
const PERMISSION_NIGHT: &str = "JinxCore:night";
const PERMISSION_RAIN: &str = "JinxCore:rain";
const PERMISSION_CLEAR: &str = "JinxCore:clear";
const PERMISSION_THUNDER: &str = "JinxCore:thunder";
const PERMISSION_CALC: &str = "JinxCore:calc";
const PERMISSION_ONLINE: &str = "JinxCore:online";
const PERMISSION_FLIP: &str = "JinxCore:flip";
const PERMISSION_PLUGINS: &str = "JinxCore:plugins";
const PERMISSION_GIVE_ALIAS: &str = "JinxCore:give";
const PERMISSION_STARTERKIT: &str = "JinxCore:starterkit";
const PERMISSION_DELSTARTERKIT: &str = "JinxCore:delstarterkit";
const PERMISSION_TPS: &str = "JinxCore:tps";
const PERMISSION_UPTIME: &str = "JinxCore:uptime";

struct PluginState {
    config: Arc<RwLock<Config>>,
    data_dir: PathBuf,
    start_time: Instant,
    seen: Arc<RwLock<HashMap<uuid::Uuid, SeenEntry>>>,
    kit_cooldowns: Arc<RwLock<HashMap<uuid::Uuid, HashMap<String, Instant>>>>,
    system_metrics: Arc<RwLock<SystemMetrics>>,
    discord_bridge: Option<DiscordBridge>,
    playtime_total_secs: Arc<RwLock<HashMap<uuid::Uuid, u64>>>,
    playtime_session_start: Arc<RwLock<HashMap<uuid::Uuid, Instant>>>,
}

struct JoinMessageHandler {
    config: Arc<RwLock<Config>>,
    seen: Arc<RwLock<HashMap<uuid::Uuid, SeenEntry>>>,
    discord: Option<DiscordBridge>,
    playtime_total_secs: Arc<RwLock<HashMap<uuid::Uuid, u64>>>,
    playtime_session_start: Arc<RwLock<HashMap<uuid::Uuid, Instant>>>,
    data_dir: PathBuf,
}

impl EventHandler<PlayerJoinEvent> for JoinMessageHandler {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PlayerJoinEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let config = {
                let guard = self.config.read().unwrap();
                guard.clone()
            };

            let name_component = TextComponent::text(event.player.gameprofile.name.clone());
            let address = event.player.client.address().await.to_string();
            {
                let mut seen = self.seen.write().unwrap();
                update_on_join(
                    &mut seen,
                    event.player.gameprofile.id,
                    event.player.gameprofile.name.clone(),
                    Some(address),
                );
            }
            {
                let mut sessions = self.playtime_session_start.write().unwrap();
                sessions.entry(event.player.gameprofile.id).or_insert_with(Instant::now);
                let mut totals = self.playtime_total_secs.write().unwrap();
                totals.entry(event.player.gameprofile.id).or_insert(0);
            }
            send_webhook(
                &config,
                WebhookEvent::Join,
                &event.player.gameprofile.name,
                None,
            );
            send_discord_event(
                self.discord.as_ref(),
                &config,
                DiscordEvent::Join,
                &event.player.gameprofile.name,
                None,
            );
            commands::apply_starterkit(&event.player, &self.data_dir).await;

            if !config.join_enabled {
                return;
            }

            let prefix =
                TextComponent::text(config.join_prefix.clone()).color_named(config.join_color);
            let message = TextComponent::text(" ").add_child(name_component);
            event.join_message = prefix.add_child(message);
        })
    }
}

struct LeaveMessageHandler {
    config: Arc<RwLock<Config>>,
    seen: Arc<RwLock<HashMap<uuid::Uuid, SeenEntry>>>,
    discord: Option<DiscordBridge>,
    playtime_total_secs: Arc<RwLock<HashMap<uuid::Uuid, u64>>>,
    playtime_session_start: Arc<RwLock<HashMap<uuid::Uuid, Instant>>>,
}

impl EventHandler<PlayerLeaveEvent> for LeaveMessageHandler {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PlayerLeaveEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let config = {
                let guard = self.config.read().unwrap();
                guard.clone()
            };

            let name_component = TextComponent::text(event.player.gameprofile.name.clone());
            {
                let mut seen = self.seen.write().unwrap();
                update_on_leave(
                    &mut seen,
                    event.player.gameprofile.id,
                    event.player.gameprofile.name.clone(),
                );
            }
            {
                let mut sessions = self.playtime_session_start.write().unwrap();
                if let Some(start) = sessions.remove(&event.player.gameprofile.id) {
                    let elapsed = start.elapsed().as_secs();
                    let mut totals = self.playtime_total_secs.write().unwrap();
                    let entry = totals.entry(event.player.gameprofile.id).or_insert(0);
                    *entry += elapsed;
                }
            }
            send_webhook(
                &config,
                WebhookEvent::Leave,
                &event.player.gameprofile.name,
                None,
            );
            send_discord_event(
                self.discord.as_ref(),
                &config,
                DiscordEvent::Leave,
                &event.player.gameprofile.name,
                None,
            );

            if !config.leave_enabled {
                return;
            }

            let prefix =
                TextComponent::text(config.leave_prefix.clone()).color_named(config.leave_color);
            let message = TextComponent::text(" ").add_child(name_component);
            event.leave_message = prefix.add_child(message);
        })
    }
}

struct JinxUtilitiesPlugin;

// command trees live in src/commands/*

impl Plugin for JinxUtilitiesPlugin {
    fn on_load(&mut self, server: Arc<Context>) -> PluginFuture<'_, Result<(), String>> {
        Box::pin(async move {
            let data_dir = server.get_data_folder();
            let config = config::load_or_create(&data_dir)?;
            let discord_bridge = start_discord_bot(&config, Arc::clone(&server.server));
            print_startup_banner(&data_dir);
            let config = Arc::new(RwLock::new(config));
            let state = Arc::new(PluginState {
                config: Arc::clone(&config),
                data_dir,
                start_time: Instant::now(),
                seen: Arc::new(RwLock::new(HashMap::new())),
                kit_cooldowns: Arc::new(RwLock::new(HashMap::new())),
                system_metrics: Arc::new(RwLock::new(SystemMetrics::default())),
                discord_bridge,
                playtime_total_secs: Arc::new(RwLock::new(HashMap::new())),
                playtime_session_start: Arc::new(RwLock::new(HashMap::new())),
            });

            let admin_permission = Permission::new(
                PERMISSION_ADMIN,
                "Admin access to JinxCore commands.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server.register_permission(admin_permission).await.ok();

            let tps_permission = Permission::new(
                PERMISSION_TPS,
                "View server TPS.",
                PermissionDefault::Allow,
            );
            server.register_permission(tps_permission).await.ok();

            let uptime_permission = Permission::new(
                PERMISSION_UPTIME,
                "View server uptime.",
                PermissionDefault::Allow,
            );
            server.register_permission(uptime_permission).await.ok();

            let gmc_permission = Permission::new(
                PERMISSION_GMC,
                "Set Creative mode.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server.register_permission(gmc_permission).await.ok();

            let gms_permission = Permission::new(
                PERMISSION_GMS,
                "Set Survival mode.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server.register_permission(gms_permission).await.ok();

            let gmsp_permission = Permission::new(
                PERMISSION_GMSP,
                "Set Spectator mode.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server.register_permission(gmsp_permission).await.ok();

            let gma_permission = Permission::new(
                PERMISSION_GMA,
                "Set Adventure mode.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server.register_permission(gma_permission).await.ok();

            let heal_permission = Permission::new(
                PERMISSION_HEAL,
                "Heal a player.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server.register_permission(heal_permission).await.ok();

            let feed_permission = Permission::new(
                PERMISSION_FEED,
                "Feed a player.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server.register_permission(feed_permission).await.ok();

            let fly_permission = Permission::new(
                PERMISSION_FLY,
                "Toggle flight.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server.register_permission(fly_permission).await.ok();

            let god_permission = Permission::new(
                PERMISSION_GOD,
                "Toggle god mode.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server.register_permission(god_permission).await.ok();

            let seen_permission = Permission::new(
                PERMISSION_SEEN,
                "View last seen status.",
                PermissionDefault::Allow,
            );
            server.register_permission(seen_permission).await.ok();

            let whois_permission = Permission::new(
                PERMISSION_WHOIS,
                "View player info.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server.register_permission(whois_permission).await.ok();

            let clearinv_permission = Permission::new(
                PERMISSION_CLEARINV,
                "Clear a player's inventory.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server.register_permission(clearinv_permission).await.ok();

            let rules_permission = Permission::new(
                PERMISSION_RULES,
                "View server rules.",
                PermissionDefault::Allow,
            );
            server.register_permission(rules_permission).await.ok();

            let discord_permission = Permission::new(
                PERMISSION_DISCORD,
                "View the Discord link.",
                PermissionDefault::Allow,
            );
            server.register_permission(discord_permission).await.ok();

            let website_permission = Permission::new(
                PERMISSION_WEBSITE,
                "View the website link.",
                PermissionDefault::Allow,
            );
            server.register_permission(website_permission).await.ok();

            let store_permission = Permission::new(
                PERMISSION_STORE,
                "View the store link.",
                PermissionDefault::Allow,
            );
            server.register_permission(store_permission).await.ok();

            let socials_permission = Permission::new(
                PERMISSION_SOCIALS,
                "View all social links.",
                PermissionDefault::Allow,
            );
            server.register_permission(socials_permission).await.ok();

            let speed_permission = Permission::new(
                PERMISSION_SPEED,
                "Adjust walk/fly speed.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server.register_permission(speed_permission).await.ok();

            let near_permission = Permission::new(
                PERMISSION_NEAR,
                "List nearby players.",
                PermissionDefault::Allow,
            );
            server.register_permission(near_permission).await.ok();

            let kit_permission = Permission::new(
                PERMISSION_KIT,
                "Use kits.",
                PermissionDefault::Allow,
            );
            server.register_permission(kit_permission).await.ok();

            let createkit_permission = Permission::new(
                PERMISSION_CREATEKIT,
                "Create kits.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server.register_permission(createkit_permission).await.ok();

            let suicide_permission = Permission::new(
                PERMISSION_SUICIDE,
                "Suicide command.",
                PermissionDefault::Allow,
            );
            server.register_permission(suicide_permission).await.ok();

            let ping_permission = Permission::new(
                PERMISSION_PING,
                "Check ping.",
                PermissionDefault::Allow,
            );
            server.register_permission(ping_permission).await.ok();

            let coords_permission = Permission::new(
                PERMISSION_COORDS,
                "Show coordinates.",
                PermissionDefault::Allow,
            );
            server.register_permission(coords_permission).await.ok();

            let playtime_permission = Permission::new(
                PERMISSION_PLAYTIME,
                "Show playtime.",
                PermissionDefault::Allow,
            );
            server.register_permission(playtime_permission).await.ok();

            let clearchat_permission = Permission::new(
                PERMISSION_CLEARCHAT,
                "Clear chat.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server.register_permission(clearchat_permission).await.ok();

            let me_permission = Permission::new(
                PERMISSION_ME,
                "Show your player info.",
                PermissionDefault::Allow,
            );
            server.register_permission(me_permission).await.ok();

            let day_permission = Permission::new(
                PERMISSION_DAY,
                "Set time to day.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server.register_permission(day_permission).await.ok();

            let night_permission = Permission::new(
                PERMISSION_NIGHT,
                "Set time to night.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server.register_permission(night_permission).await.ok();

            let rain_permission = Permission::new(
                PERMISSION_RAIN,
                "Set weather to rain.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server.register_permission(rain_permission).await.ok();

            let clear_permission = Permission::new(
                PERMISSION_CLEAR,
                "Clear weather.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server.register_permission(clear_permission).await.ok();

            let thunder_permission = Permission::new(
                PERMISSION_THUNDER,
                "Set weather to thunder.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server.register_permission(thunder_permission).await.ok();

            let calc_permission = Permission::new(
                PERMISSION_CALC,
                "Calculator command.",
                PermissionDefault::Allow,
            );
            server.register_permission(calc_permission).await.ok();

            let online_permission = Permission::new(
                PERMISSION_ONLINE,
                "List online players.",
                PermissionDefault::Allow,
            );
            server.register_permission(online_permission).await.ok();

            let flip_permission = Permission::new(
                PERMISSION_FLIP,
                "Flip a coin.",
                PermissionDefault::Allow,
            );
            server.register_permission(flip_permission).await.ok();

            let plugins_permission = Permission::new(
                PERMISSION_PLUGINS,
                "List loaded plugins.",
                PermissionDefault::Allow,
            );
            server.register_permission(plugins_permission).await.ok();

            let give_permission = Permission::new(
                PERMISSION_GIVE_ALIAS,
                "Alias for /give.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server.register_permission(give_permission).await.ok();

            let starterkit_permission = Permission::new(
                PERMISSION_STARTERKIT,
                "Set the starter kit.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server.register_permission(starterkit_permission).await.ok();

            let delstarterkit_permission = Permission::new(
                PERMISSION_DELSTARTERKIT,
                "Delete the starter kit.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server.register_permission(delstarterkit_permission).await.ok();

            let bypass_permission = Permission::new(
                PERMISSION_ANTISPAM_BYPASS,
                "Bypass the anti-spam filter.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server.register_permission(bypass_permission).await.ok();

            let chatfilter_bypass_permission = Permission::new(
                PERMISSION_CHATFILTER_BYPASS,
                "Bypass the chat filter.",
                PermissionDefault::Op(PermissionLvl::Two),
            );
            server
                .register_permission(chatfilter_bypass_permission)
                .await
                .ok();

            server
                .register_command(commands::jinx_command_tree(Arc::clone(&state)), PERMISSION_ADMIN)
                .await;
            server
                .register_command(commands::tps_command_tree(), PERMISSION_TPS)
                .await;
            server
                .register_command(
                    commands::uptime_command_tree(Arc::clone(&state)),
                    PERMISSION_UPTIME,
                )
                .await;
            server
                .register_command(commands::seen_command_tree(Arc::clone(&state)), PERMISSION_SEEN)
                .await;
            server
                .register_command(commands::whois_command_tree(Arc::clone(&state)), PERMISSION_WHOIS)
                .await;
            server
                .register_command(commands::clearinv_command_tree(), PERMISSION_CLEARINV)
                .await;
            server
                .register_command(commands::rules_command_tree(Arc::clone(&state)), PERMISSION_RULES)
                .await;
            server
                .register_command(
                    commands::discord_command_tree(Arc::clone(&state)),
                    PERMISSION_DISCORD,
                )
                .await;
            server
                .register_command(
                    commands::website_command_tree(Arc::clone(&state)),
                    PERMISSION_WEBSITE,
                )
                .await;
            server
                .register_command(
                    commands::store_command_tree(Arc::clone(&state)),
                    PERMISSION_STORE,
                )
                .await;
            server
                .register_command(
                    commands::socials_command_tree(Arc::clone(&state)),
                    PERMISSION_SOCIALS,
                )
                .await;
            server
                .register_command(commands::speed_command_tree(), PERMISSION_SPEED)
                .await;
            server
                .register_command(commands::near_command_tree(), PERMISSION_NEAR)
                .await;
            server
                .register_command(
                    commands::createkit_command_tree(Arc::clone(&state)),
                    PERMISSION_CREATEKIT,
                )
                .await;
            server
                .register_command(
                    commands::kit_command_tree(Arc::clone(&state)),
                    PERMISSION_KIT,
                )
                .await;
            server
                .register_command(commands::suicide_command_tree(), PERMISSION_SUICIDE)
                .await;
            server
                .register_command(commands::ping_command_tree(), PERMISSION_PING)
                .await;
            server
                .register_command(commands::coords_command_tree(), PERMISSION_COORDS)
                .await;
            server
                .register_command(commands::playtime_command_tree(Arc::clone(&state)), PERMISSION_PLAYTIME)
                .await;
            server
                .register_command(commands::clearchat_command_tree(), PERMISSION_CLEARCHAT)
                .await;
            server
                .register_command(commands::me_command_tree(), PERMISSION_ME)
                .await;
            server
                .register_command(commands::day_command_tree(), PERMISSION_DAY)
                .await;
            server
                .register_command(commands::night_command_tree(), PERMISSION_NIGHT)
                .await;
            server
                .register_command(commands::rain_command_tree(), PERMISSION_RAIN)
                .await;
            server
                .register_command(commands::clear_command_tree(), PERMISSION_CLEAR)
                .await;
            server
                .register_command(commands::thunder_command_tree(), PERMISSION_THUNDER)
                .await;
            server
                .register_command(commands::calc_command_tree(), PERMISSION_CALC)
                .await;
            server
                .register_command(commands::online_command_tree(), PERMISSION_ONLINE)
                .await;
            server
                .register_command(commands::flip_command_tree(), PERMISSION_FLIP)
                .await;
            server
                .register_command(commands::plugins_alias_command_tree(), PERMISSION_PLUGINS)
                .await;
            server
                .register_command(commands::give_alias_command_tree(), PERMISSION_GIVE_ALIAS)
                .await;
            server
                .register_command(
                    commands::starterkit_command_tree(Arc::clone(&state)),
                    PERMISSION_STARTERKIT,
                )
                .await;
            server
                .register_command(
                    commands::delstarterkit_command_tree(Arc::clone(&state)),
                    PERMISSION_DELSTARTERKIT,
                )
                .await;
            server
                .register_command(commands::gmc_command_tree(), PERMISSION_GMC)
                .await;
            server
                .register_command(commands::gms_command_tree(), PERMISSION_GMS)
                .await;
            server
                .register_command(commands::gmsp_command_tree(), PERMISSION_GMSP)
                .await;
            server
                .register_command(commands::gma_command_tree(), PERMISSION_GMA)
                .await;
            server
                .register_command(commands::creative_command_tree(), PERMISSION_GMC)
                .await;
            server
                .register_command(commands::survival_command_tree(), PERMISSION_GMS)
                .await;
            server
                .register_command(commands::spectator_command_tree(), PERMISSION_GMSP)
                .await;
            server
                .register_command(commands::adventure_command_tree(), PERMISSION_GMA)
                .await;
            server
                .register_command(commands::c_command_tree(), PERMISSION_GMC)
                .await;
            server
                .register_command(commands::s_command_tree(), PERMISSION_GMS)
                .await;
            server
                .register_command(commands::sp_command_tree(), PERMISSION_GMSP)
                .await;
            server
                .register_command(commands::a_command_tree(), PERMISSION_GMA)
                .await;
            server
                .register_command(commands::heal_command_tree(), PERMISSION_HEAL)
                .await;
            server
                .register_command(commands::feed_command_tree(), PERMISSION_FEED)
                .await;
            server
                .register_command(commands::fly_command_tree(), PERMISSION_FLY)
                .await;
            server
                .register_command(commands::god_command_tree(), PERMISSION_GOD)
                .await;

            start_scoreboard_task(Arc::clone(&server.server), Arc::clone(&state));
            start_system_sampler(Arc::clone(&state));

            server
                .register_event::<PlayerJoinEvent, _>(
                    Arc::new(JoinMessageHandler {
                        config: Arc::clone(&config),
                        seen: Arc::clone(&state.seen),
                        discord: state.discord_bridge.clone(),
                        playtime_total_secs: Arc::clone(&state.playtime_total_secs),
                        playtime_session_start: Arc::clone(&state.playtime_session_start),
                        data_dir: state.data_dir.clone(),
                    }),
                    EventPriority::Normal,
                    true,
                )
                .await;
            server
                .register_event::<PlayerLeaveEvent, _>(
                    Arc::new(LeaveMessageHandler {
                        config: Arc::clone(&config),
                        seen: Arc::clone(&state.seen),
                        discord: state.discord_bridge.clone(),
                        playtime_total_secs: Arc::clone(&state.playtime_total_secs),
                        playtime_session_start: Arc::clone(&state.playtime_session_start),
                    }),
                    EventPriority::Normal,
                    true,
                )
                .await;
            server
                .register_event::<PlayerChatEvent, _>(
                    Arc::new(AntiSpamHandler::new(Arc::clone(&config))),
                    EventPriority::High,
                    true,
                )
                .await;
            server
                .register_event::<PlayerChatEvent, _>(
                    Arc::new(ChatFilterHandler::new(Arc::clone(&config))),
                    EventPriority::High,
                    true,
                )
                .await;
            server
                .register_event::<PlayerChatEvent, _>(
                    Arc::new(ChatFormatHandler::new(
                        Arc::clone(&config),
                        state.discord_bridge.clone(),
                    )),
                    EventPriority::Lowest,
                    true,
                )
                .await;
            Ok(())
        })
    }
}

#[unsafe(no_mangle)]
pub static METADATA: PluginMetadata<'static> = PluginMetadata {
    name: "JinxCore",
    version: env!("CARGO_PKG_VERSION"),
    authors: env!("CARGO_PKG_AUTHORS"),
    description: env!("CARGO_PKG_DESCRIPTION"),
};

#[unsafe(no_mangle)]
pub static PUMPKIN_API_VERSION: u32 = PLUGIN_API_VERSION;

#[unsafe(no_mangle)]
pub fn plugin() -> Box<dyn Plugin> {
    Box::new(JinxUtilitiesPlugin)
}

fn print_startup_banner(data_dir: &PathBuf) {
    let first_boot_path = data_dir.join("first_boot.done");
    let is_first_boot = !first_boot_path.exists();
    if is_first_boot {
        let _ = fs::write(&first_boot_path, "first_boot_complete");
        println!("\x1b[95m==================================================\x1b[0m");
        println!("\x1b[96m                  JinxCore\x1b[0m");
        println!("\x1b[95m==================================================\x1b[0m");
        println!("\x1b[92m Thank you for installing JinxCore!\x1b[0m");
        println!("\x1b[0m");
        println!("\x1b[37m This is the first time JinxCore has been loaded");
        println!(" on this server. JinxCore provides a reliable");
        println!(" foundation for advanced server features");
        println!(" and future expansions.\x1b[0m");
        println!("\x1b[0m");
        println!("\x1b[37m Built with long-term stability, clean design,");
        println!(" and server performance in mind.\x1b[0m");
        println!("\x1b[0m");
        println!("\x1b[36m Developed with passion and care by Jinx.\x1b[0m");
        println!("\x1b[0m");
        println!("\x1b[37m Setup complete — all required configuration");
        println!(" files have been successfully created.");
        println!(" JinxCore is now ready for use.\x1b[0m");
        println!("\x1b[95m==================================================\x1b[0m");
    } else {
        println!("\x1b[95m====================================\x1b[0m");
        println!("\x1b[96m             JinxCore\x1b[0m");
        println!("\x1b[95m====================================\x1b[0m");
        println!("\x1b[32m JinxCore loaded successfully.\x1b[0m");
        println!("\x1b[37m Status: Enabled.\x1b[0m");
        println!("\x1b[95m====================================\x1b[0m");
    }
}
