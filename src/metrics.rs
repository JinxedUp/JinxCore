use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use pumpkin::SHOULD_STOP;
use sysinfo::{Disks, System};

use crate::PluginState;
const SYSTEM_SAMPLE_INTERVAL_SECS: u64 = 1;
const DISK_SAMPLE_INTERVAL_SECS: u64 = 30;

#[derive(Clone, Debug)]
pub struct SystemMetrics {
    pub mem_used_kib: u64,
    pub mem_total_kib: u64,
    pub disk_used_bytes: u64,
    pub disk_total_bytes: u64,
    pub last_updated: Instant,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            mem_used_kib: 0,
            mem_total_kib: 0,
            disk_used_bytes: 0,
            disk_total_bytes: 0,
            last_updated: Instant::now(),
        }
    }
}

pub fn start_system_sampler(state: Arc<PluginState>) {
    thread::spawn(move || {
        let mut system = System::new();
        let mut disks = Disks::new_with_refreshed_list();
        let mut last_disk_sample = Instant::now() - Duration::from_secs(DISK_SAMPLE_INTERVAL_SECS);

        loop {
            if SHOULD_STOP.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }

            system.refresh_memory();
            let mem_used_kib = system.used_memory();
            let mem_total_kib = system.total_memory();

            let now = Instant::now();
            let (disk_used_bytes, disk_total_bytes) = if now.duration_since(last_disk_sample)
                >= Duration::from_secs(DISK_SAMPLE_INTERVAL_SECS)
            {
                disks.refresh();
                last_disk_sample = now;
                let (total, used) = disks
                    .list()
                    .iter()
                    .fold((0u64, 0u64), |acc, disk| {
                        let total = disk.total_space();
                        let avail = disk.available_space();
                        (acc.0 + total, acc.1 + (total - avail))
                    });
                (used, total)
            } else {
                let guard = state.system_metrics.read().unwrap();
                (guard.disk_used_bytes, guard.disk_total_bytes)
            };

            {
                let mut metrics = state.system_metrics.write().unwrap();
                metrics.mem_used_kib = mem_used_kib;
                metrics.mem_total_kib = mem_total_kib;
                metrics.disk_used_bytes = disk_used_bytes;
                metrics.disk_total_bytes = disk_total_bytes;
                metrics.last_updated = now;
            }

            thread::sleep(Duration::from_secs(SYSTEM_SAMPLE_INTERVAL_SECS));
        }
    });
}
