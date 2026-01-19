use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};
use std::thread;
use std::time::Duration;

use pumpkin::entity::player::Player;
use pumpkin::SHOULD_STOP;
use pumpkin::server::Server;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{
    CDisplayObjective, CUpdateScore, Mode, RenderType,
};
use pumpkin_protocol::packet::Packet;
use pumpkin_protocol::ser::NetworkWriteExt;
use pumpkin_protocol::{ClientPacket, NumberFormat};
use pumpkin_protocol::ser::{WritingError, serializer::Serializer};
use pumpkin_util::text::{TextComponent, color::NamedColor};
use pumpkin_data::packet::clientbound::PLAY_SET_OBJECTIVE;
use pumpkin_data::scoreboard::ScoreboardDisplaySlot;
use serde::Serialize;
use uuid::Uuid;

use crate::config::Config;
use crate::PluginState;

const OBJECTIVE_NAME: &str = "jinx_sidebar";
const SCOREBOARD_FILE_NAME: &str = "scoreboard.txt";
const MAX_LINES: usize = 15;
static INITIALIZED_PLAYERS: LazyLock<Mutex<HashSet<Uuid>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

struct CUpdateObjectivesFixed {
    objective_name: String,
    mode: u8,
    display_name: TextComponent,
    render_type: RenderType,
    number_format: Option<NumberFormat>,
}

impl CUpdateObjectivesFixed {
    fn new(
        objective_name: String,
        mode: Mode,
        display_name: TextComponent,
        render_type: RenderType,
        number_format: Option<NumberFormat>,
    ) -> Self {
        Self {
            objective_name,
            mode: mode as u8,
            display_name,
            render_type,
            number_format,
        }
    }
}

impl Packet for CUpdateObjectivesFixed {
    const PACKET_ID: pumpkin_protocol::codec::var_int::VarIntType = PLAY_SET_OBJECTIVE;
}

impl ClientPacket for CUpdateObjectivesFixed {
    fn write_packet_data(&self, write: impl std::io::Write) -> Result<(), WritingError> {
        let mut write = write;
        write.write_string(&self.objective_name)?;
        write.write_u8(self.mode)?;

        if self.mode == Mode::Add as u8 || self.mode == Mode::Update as u8 {
            let translated = TextComponent(self.display_name.0.clone().to_translated());
            write_text_component(&mut write, &translated)?;
            let render_type = match &self.render_type {
                RenderType::Integer => 0,
                RenderType::Hearts => 1,
            };
            write.write_var_int(&VarInt(render_type))?;
            write.write_option(&self.number_format, |p, v| match v {
                NumberFormat::Blank => p.write_var_int(&VarInt(0)),
                NumberFormat::Styled(style) => {
                    p.write_var_int(&VarInt(1))?;
                    pumpkin_nbt::serializer::to_bytes_unnamed(style, p)
                        .map_err(|err: pumpkin_nbt::Error| WritingError::Serde(err.to_string()))
                }
                NumberFormat::Fixed(text_component) => {
                    p.write_var_int(&VarInt(2))?;
                    let translated = TextComponent(text_component.0.clone().to_translated());
                    write_text_component(p, &translated)
                }
            })?;
        }

        Ok(())
    }
}

fn write_text_component(
    write: &mut impl std::io::Write,
    component: &TextComponent,
) -> Result<(), WritingError> {
    let mut serializer = Serializer::new(write);
    component.serialize(&mut serializer).map_err(|err| {
        WritingError::Serde(format!("Failed to serialize TextComponent: {err}"))
    })
}

pub fn start_scoreboard_task(server: Arc<Server>, state: Arc<PluginState>) {
    let server_ref = Arc::clone(&server);
    thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("scoreboard runtime");

        loop {
            if SHOULD_STOP.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }

            let config = {
                let guard = state.config.read().unwrap();
                guard.clone()
            };

            runtime.block_on(async {
                if config.scoreboard_enabled {
                    update_sidebar(&server_ref, &state, &config).await;
                } else {
                    clear_sidebar(&server_ref).await;
                }
            });

            let sleep_for = Duration::from_secs(config.scoreboard_update_interval_sec.max(1));
            thread::sleep(sleep_for);
        }
    });
}

async fn update_sidebar(server: &Server, state: &PluginState, config: &Config) {
    let players = collect_players(server).await;
    if players.is_empty() {
        return;
    }
    {
        let current_ids = players
            .iter()
            .map(|player| player.gameprofile.id)
            .collect::<HashSet<_>>();
        let mut initialized = INITIALIZED_PLAYERS.lock().unwrap();
        initialized.retain(|id| current_ids.contains(id));
    }

    let avg_nanos = server.get_average_tick_time_nanos();
    let tps = if avg_nanos <= 0 {
        0.0
    } else {
        let target = server.tick_rate_manager.tickrate() as f64;
        let current = 1_000_000_000.0 / avg_nanos as f64;
        current.min(target)
    };
    let uptime = format_uptime(state.start_time.elapsed().as_secs());

    let (title_text, raw_lines) =
        load_scoreboard_text(&state.data_dir, &config.scoreboard_title);
    let title = parse_colored_text(&title_text);

    let rendered_lines = raw_lines
        .into_iter()
        .map(|line| {
            line.replace("%online%", &players.len().to_string())
                .replace("%tps%", &format!("{tps:.2}"))
                .replace("%uptime%", &uptime)
        })
        .collect::<Vec<_>>();

    let mut lines = Vec::new();
    let mut score = rendered_lines.len() as i32;
    for (index, line) in rendered_lines.into_iter().enumerate() {
        let entry_name = format!("line_{index}");
        lines.push((entry_name, score, parse_colored_text(&line)));
        score -= 1;
    }

    for player in players {
        send_sidebar_packets(&player, &title, &lines).await;
    }
}

async fn clear_sidebar(server: &Server) {
    let players = collect_players(server).await;
    let remove = CUpdateObjectivesFixed::new(
        OBJECTIVE_NAME.to_string(),
        Mode::Remove,
        TextComponent::text(""),
        RenderType::Integer,
        None,
    );
    for player in players {
        player.client.enqueue_packet(&remove).await;
    }
    INITIALIZED_PLAYERS.lock().unwrap().clear();
}

async fn send_sidebar_packets(
    player: &Arc<Player>,
    title: &TextComponent,
    lines: &[(String, i32, TextComponent)],
) {
    let player_id = player.gameprofile.id;
    let mut initialized = INITIALIZED_PLAYERS.lock().unwrap();
    let is_new = initialized.insert(player_id);
    let mode = if is_new { Mode::Add } else { Mode::Update };

    let objective = CUpdateObjectivesFixed::new(
        OBJECTIVE_NAME.to_string(),
        mode,
        title.clone(),
        RenderType::Integer,
        None,
    );
    player.client.enqueue_packet(&objective).await;

    if is_new {
        let display =
            CDisplayObjective::new(ScoreboardDisplaySlot::Sidebar, OBJECTIVE_NAME.to_string());
        player.client.enqueue_packet(&display).await;
    }

    for (name, score, display_name) in lines {
        let score_packet = CUpdateScore::new(
            name.clone(),
            OBJECTIVE_NAME.to_string(),
            VarInt(*score),
            Some(display_name.clone()),
            None,
        );
        player.client.enqueue_packet(&score_packet).await;
    }
}

async fn collect_players(server: &Server) -> Vec<Arc<Player>> {
    let mut players = Vec::new();
    let mut seen = HashSet::new();

    for world in server.worlds.read().await.iter() {
        for (uuid, player) in world.players.read().await.iter() {
            if seen.insert(*uuid) {
                players.push(Arc::clone(player));
            }
        }
    }

    players
}

fn format_uptime(total_secs: u64) -> String {
    let days = total_secs / 86_400;
    let hours = (total_secs % 86_400) / 3_600;
    let minutes = (total_secs % 3_600) / 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

fn load_scoreboard_text(data_dir: &Path, fallback_title: &str) -> (String, Vec<String>) {
    let path = data_dir.join(SCOREBOARD_FILE_NAME);
    ensure_scoreboard_file(&path, fallback_title);

    let content = fs::read_to_string(&path).unwrap_or_default();
    let mut lines = content
        .lines()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    let title = if !lines.is_empty() {
        lines.remove(0)
    } else {
        fallback_title.to_string()
    };

    let lines = lines
        .into_iter()
        .take(MAX_LINES)
        .collect::<Vec<_>>();

    (title, lines)
}

fn ensure_scoreboard_file(path: &PathBuf, fallback_title: &str) {
    if path.exists() {
        return;
    }

    let default_lines = format!(
        "{title}\nOnline: %online%\nTPS: %tps%\nUptime: %uptime%\n",
        title = fallback_title
    );

    let _ = fs::write(path, default_lines);
}

fn parse_colored_text(input: &str) -> TextComponent {
    let mut chars = input.chars().peekable();
    let mut current = TextComponent::text("");
    let mut current_color: Option<NamedColor> = None;
    let mut buffer = String::new();

    while let Some(ch) = chars.next() {
        if ch == '&' {
            if let Some(code) = chars.peek().copied() {
                if let Some(color) = color_from_code(code) {
                    if !buffer.is_empty() {
                        let mut part = TextComponent::text(buffer.clone());
                        if let Some(named) = current_color {
                            part = part.color_named(named);
                        }
                        current = current.add_child(part);
                        buffer.clear();
                    }
                    current_color = Some(color);
                    chars.next();
                    continue;
                }
            }
        }
        buffer.push(ch);
    }

    if !buffer.is_empty() {
        let mut part = TextComponent::text(buffer);
        if let Some(named) = current_color {
            part = part.color_named(named);
        }
        current = current.add_child(part);
    }

    current
}

fn color_from_code(code: char) -> Option<NamedColor> {
    match code.to_ascii_lowercase() {
        '0' => Some(NamedColor::Black),
        '1' => Some(NamedColor::DarkBlue),
        '2' => Some(NamedColor::DarkGreen),
        '3' => Some(NamedColor::DarkAqua),
        '4' => Some(NamedColor::DarkRed),
        '5' => Some(NamedColor::DarkPurple),
        '6' => Some(NamedColor::Gold),
        '7' => Some(NamedColor::Gray),
        '8' => Some(NamedColor::DarkGray),
        '9' => Some(NamedColor::Blue),
        'a' => Some(NamedColor::Green),
        'b' => Some(NamedColor::Aqua),
        'c' => Some(NamedColor::Red),
        'd' => Some(NamedColor::LightPurple),
        'e' => Some(NamedColor::Yellow),
        'f' => Some(NamedColor::White),
        _ => None,
    }
}
