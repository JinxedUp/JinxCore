use std::collections::HashMap;
use std::fs;
use std::path::Path;

const SOCIALS_FILE_NAME: &str = "socials.txt";

pub fn socials_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join(SOCIALS_FILE_NAME)
}

pub fn ensure_socials_file(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if !path.exists() {
        let default_text = "discord: https://discord.gg/yourserver\n\
website: https://example.com\n\
store: https://store.example.com\n";
        fs::write(path, default_text).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn load_socials(path: &Path) -> Result<HashMap<String, String>, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim().to_lowercase();
        let value = value.trim();
        if !key.is_empty() && !value.is_empty() {
            map.insert(key, value.to_string());
        }
    }
    Ok(map)
}
