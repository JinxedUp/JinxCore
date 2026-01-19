use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SeenEntry {
    pub uuid: Uuid,
    pub name: String,
    pub last_seen: SystemTime,
    pub online: bool,
    pub last_address: Option<String>,
}

impl SeenEntry {
    pub fn new(uuid: Uuid, name: String, address: Option<String>) -> Self {
        Self {
            uuid,
            name,
            last_seen: SystemTime::now(),
            online: true,
            last_address: address,
        }
    }
}

pub fn update_on_join(
    seen: &mut HashMap<Uuid, SeenEntry>,
    uuid: Uuid,
    name: String,
    address: Option<String>,
) {
    let entry = seen
        .entry(uuid)
        .or_insert_with(|| SeenEntry::new(uuid, name.clone(), address.clone()));
    entry.name = name;
    entry.online = true;
    entry.last_seen = SystemTime::now();
    entry.last_address = address;
}

pub fn update_on_leave(seen: &mut HashMap<Uuid, SeenEntry>, uuid: Uuid, name: String) {
    let entry = seen
        .entry(uuid)
        .or_insert_with(|| SeenEntry::new(uuid, name.clone(), None));
    entry.name = name;
    entry.online = false;
    entry.last_seen = SystemTime::now();
}

pub fn find_by_name<'a>(
    seen: &'a HashMap<Uuid, SeenEntry>,
    name: &str,
) -> Option<&'a SeenEntry> {
    let name_lower = name.to_lowercase();
    seen.values()
        .find(|entry| entry.name.to_lowercase() == name_lower)
}

pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let days = total_secs / 86_400;
    let hours = (total_secs % 86_400) / 3_600;
    let minutes = (total_secs % 3_600) / 60;
    let seconds = total_secs % 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m {seconds}s")
    } else if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}
