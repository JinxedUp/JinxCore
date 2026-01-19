use std::time::Duration;

use pumpkin::command::{
    args::ConsumedArgs,
    CommandExecutor, CommandResult, CommandSender,
};
use pumpkin::server::Server;
use pumpkin_util::text::{color::NamedColor, TextComponent};
use crate::{PluginState, branding};

pub struct HealthExecutor {
    pub state: std::sync::Arc<PluginState>,
}

impl CommandExecutor for HealthExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {

            let avg_nanos = server.get_average_tick_time_nanos();
            let mspt = if avg_nanos <= 0 { 0.0 } else { avg_nanos as f64 / 1_000_000.0 };
            let tps = if mspt <= 0.0 {
                0.0
            } else {
                let target = server.tick_rate_manager.tickrate() as f64;
                let current: f64 = 1000.0 / mspt;
                current.min(target)
            };

            let uptime = format_uptime(self.state.start_time.elapsed());

            let metrics = self.state.system_metrics.read().unwrap().clone();
            let total_mem = metrics.mem_total_kib;
            let used_mem = metrics.mem_used_kib;
            let mem_label = format!(
                "{}/{}",
                format_gib_from_kib(used_mem),
                format_gib_from_kib(total_mem)
            );
            let mem_bar = usage_bar(used_mem as f64, total_mem as f64, 20, &mem_label);

            let total_disk = metrics.disk_total_bytes;
            let used_disk = metrics.disk_used_bytes;
            let disk_label = format!(
                "{}/{}",
                format_gib_from_bytes(used_disk),
                format_gib_from_bytes(total_disk)
            );
            let disk_bar = usage_bar(used_disk as f64, total_disk as f64, 20, &disk_label);

            let lines = vec![
                line(
                    "Ram usage",
                    TextComponent::text(mem_bar)
                        .color_named(usage_color(used_mem as f64, total_mem as f64)),
                ),
                line(
                    "Storage usage",
                    TextComponent::text(disk_bar)
                        .color_named(usage_color(used_disk as f64, total_disk as f64)),
                ),
                line(
                    "Average TPS",
                    TextComponent::text(format!("{tps:.2}")).color_named(tps_color(tps)),
                ),
                line(
                    "Average MSPT",
                    TextComponent::text(format!("{mspt:.2}")).color_named(mspt_color(mspt)),
                ),
                line(
                    "Uptime",
                    TextComponent::text(uptime).color_named(NamedColor::Green),
                ),
            ];

            let body = join_lines(lines);
            sender.send_message(branding::brand(body)).await;
            Ok(())
        })
    }
}

fn usage_bar(used: f64, total: f64, width: usize, label: &str) -> String {
    if total <= 0.0 {
        return "[....................] 0% (n/a)".to_string();
    }
    let ratio = (used / total).clamp(0.0, 1.0);
    let filled = (ratio * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    let percent = (ratio * 100.0).round() as u64;
    format!(
        "[{}{}] {}% ({})",
        "#".repeat(filled),
        ".".repeat(empty),
        percent,
        label
    )
}

fn format_gib_from_kib(kib: u64) -> String {
    let gib = kib as f64 / 1024.0 / 1024.0;
    format!("{gib:.1}GB")
}

fn format_gib_from_bytes(bytes: u64) -> String {
    let gib = bytes as f64 / 1024.0 / 1024.0 / 1024.0;
    format!("{gib:.1}GB")
}

fn line(label: &str, value: TextComponent) -> TextComponent {
    TextComponent::text(format!("{label}: "))
        .color_named(NamedColor::Gray)
        .add_child(value)
}

fn join_lines(lines: Vec<TextComponent>) -> TextComponent {
    let mut body = TextComponent::text("");
    for (idx, line) in lines.into_iter().enumerate() {
        if idx > 0 {
            body = body.add_child(TextComponent::text("\n"));
        }
        body = body.add_child(line);
    }
    body
}

fn usage_color(used: f64, total: f64) -> NamedColor {
    if total <= 0.0 {
        return NamedColor::Gray;
    }
    let ratio = used / total;
    if ratio >= 0.9 {
        NamedColor::Red
    } else if ratio >= 0.75 {
        NamedColor::Yellow
    } else {
        NamedColor::Green
    }
}

fn tps_color(tps: f64) -> NamedColor {
    if tps >= 18.0 {
        NamedColor::Green
    } else if tps >= 15.0 {
        NamedColor::Yellow
    } else {
        NamedColor::Red
    }
}

fn mspt_color(mspt: f64) -> NamedColor {
    if mspt <= 50.0 {
        NamedColor::Green
    } else if mspt <= 75.0 {
        NamedColor::Yellow
    } else {
        NamedColor::Red
    }
}

fn format_uptime(duration: Duration) -> String {
    let total_secs = duration.as_secs();
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

pub fn jinx_health_command(state: std::sync::Arc<PluginState>) -> HealthExecutor {
    HealthExecutor { state }
}
