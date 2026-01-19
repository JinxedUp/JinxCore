use pumpkin::command::{
    args::ConsumedArgs,
    CommandExecutor, CommandResult, CommandSender, tree::CommandTree,
};
use pumpkin::server::Server;
use std::fs;
use std::path::Path;

use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::branding;

struct PluginsExecutor;

fn collect_plugin_files() -> Vec<String> {
    let mut names = Vec::new();
    let path = Path::new("plugins");
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return names,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
            continue;
        };
        let ext = ext.to_ascii_lowercase();
        if ext != "dll" && ext != "so" && ext != "dylib" {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        names.push(file_name.to_string());
    }

    names.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    names
}

impl CommandExecutor for PluginsExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let names = collect_plugin_files();
            let count = names.len();
            let list = if names.is_empty() {
                "None".to_string()
            } else {
                names.join(", ")
            };

            let body = TextComponent::text(format!("Plugins ({count}): {list}"))
                .color_named(NamedColor::White);
            sender.send_message(branding::brand(body)).await;
            Ok(())
        })
    }
}

pub fn plugins_alias_command_tree() -> CommandTree {
    CommandTree::new(["pl"], "List loaded plugins.")
        .execute(PluginsExecutor)
}
