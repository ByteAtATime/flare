use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use sysinfo::{CpuRefreshKind, Disks, Networks, RefreshKind, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub usage_percent: f64,
    pub cores: Vec<CoreInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreInfo {
    pub index: usize,
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f64,
    pub file_system: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub interface: String,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryInfo {
    pub percentage: f64,
    pub is_charging: bool,
    pub is_present: bool,
    pub time_remaining_minutes: Option<u32>,
}

// Global cached CPU info updated by background thread
lazy_static::lazy_static! {
    static ref CPU_INFO_CACHE: Arc<Mutex<CpuInfo>> = {
        let cache = Arc::new(Mutex::new(CpuInfo {
            usage_percent: 0.0,
            cores: Vec::new(),
        }));

        // Spawn background thread to update CPU info
        let cache_clone = Arc::clone(&cache);
        thread::spawn(move || {
            let mut sys = System::new_with_specifics(
                RefreshKind::new().with_cpu(CpuRefreshKind::everything()),
            );

            loop {
                // Sleep first to allow initial CPU measurement
                thread::sleep(Duration::from_millis(500));
                sys.refresh_cpu_all();

                let global_usage = sys.global_cpu_usage() as f64;
                let cores = sys
                    .cpus()
                    .iter()
                    .enumerate()
                    .map(|(index, cpu)| CoreInfo {
                        index,
                        usage_percent: cpu.cpu_usage() as f64,
                    })
                    .collect();

                if let Ok(mut cache) = cache_clone.lock() {
                    *cache = CpuInfo {
                        usage_percent: global_usage,
                        cores,
                    };
                }
            }
        });

        cache
    };
}

/// Get current CPU usage information (non-blocking, returns cached value)
pub fn get_cpu_info() -> CpuInfo {
    CPU_INFO_CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

/// Get current memory usage information
pub fn get_memory_info() -> MemoryInfo {
    let mut sys = System::new_with_specifics(
        RefreshKind::new().with_memory(sysinfo::MemoryRefreshKind::everything()),
    );
    sys.refresh_memory();

    let total = sys.total_memory();
    let used = sys.used_memory();
    let available = sys.available_memory();

    let usage_percent = if total > 0 {
        (used as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    MemoryInfo {
        total_bytes: total,
        used_bytes: used,
        available_bytes: available,
        usage_percent,
    }
}

/// Get disk usage information for all mounted disks
pub fn get_disk_info() -> Vec<DiskInfo> {
    let disks = Disks::new_with_refreshed_list();

    disks
        .iter()
        .map(|disk| {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);

            let usage_percent = if total > 0 {
                (used as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_bytes: total,
                used_bytes: used,
                available_bytes: available,
                usage_percent,
                file_system: disk.file_system().to_string_lossy().to_string(),
            }
        })
        .collect()
}

/// Get network interface statistics
pub fn get_network_info() -> Vec<NetworkInfo> {
    let networks = Networks::new_with_refreshed_list();

    networks
        .iter()
        .map(|(interface_name, data)| NetworkInfo {
            interface: interface_name.clone(),
            bytes_sent: data.total_transmitted(),
            bytes_received: data.total_received(),
            packets_sent: data.total_packets_transmitted(),
            packets_received: data.total_packets_received(),
        })
        .collect()
}

/// Get battery information
/// Reads from /sys/class/power_supply/ on Linux
pub fn get_battery_info() -> Option<BatteryInfo> {
    // Try to find battery in /sys/class/power_supply/
    let power_supply_path = Path::new("/sys/class/power_supply");

    if !power_supply_path.exists() {
        return None;
    }

    // Look for BAT0, BAT1, or any battery device
    let battery_names = ["BAT0", "BAT1", "battery"];

    for battery_name in &battery_names {
        let battery_path = power_supply_path.join(battery_name);
        if battery_path.exists() {
            if let Some(info) = read_battery_from_path(&battery_path) {
                return Some(info);
            }
        }
    }

    // Try to find any directory that looks like a battery
    if let Ok(entries) = fs::read_dir(power_supply_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Check if it has a capacity file (indicates it's a battery)
                if path.join("capacity").exists() {
                    if let Some(info) = read_battery_from_path(&path) {
                        return Some(info);
                    }
                }
            }
        }
    }

    None
}

fn read_battery_from_path(battery_path: &Path) -> Option<BatteryInfo> {
    // Read capacity (percentage)
    let capacity = fs::read_to_string(battery_path.join("capacity"))
        .ok()?
        .trim()
        .parse::<f64>()
        .ok()?;

    // Read status (Charging, Discharging, Full, etc.)
    let status = fs::read_to_string(battery_path.join("status"))
        .ok()?
        .trim()
        .to_lowercase();

    let is_charging = status.contains("charging") || status.contains("full");

    // Try to calculate time remaining
    let time_remaining_minutes = if !is_charging {
        // Read current power draw and energy remaining
        let energy_now = fs::read_to_string(battery_path.join("energy_now"))
            .or_else(|_| fs::read_to_string(battery_path.join("charge_now")))
            .ok()?
            .trim()
            .parse::<u64>()
            .ok();

        let power_now = fs::read_to_string(battery_path.join("power_now"))
            .or_else(|_| fs::read_to_string(battery_path.join("current_now")))
            .ok()?
            .trim()
            .parse::<u64>()
            .ok();

        if let (Some(energy), Some(power)) = (energy_now, power_now) {
            if power > 0 {
                // Time in hours = energy / power, convert to minutes
                let hours = energy as f64 / power as f64;
                Some((hours * 60.0) as u32)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    Some(BatteryInfo {
        percentage: capacity,
        is_charging,
        is_present: true,
        time_remaining_minutes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_info() {
        // Sleep to allow background thread to initialize CPU cache
        // The background thread waits 500ms before first refresh
        thread::sleep(Duration::from_millis(700));

        let cpu_info = get_cpu_info();

        // Usage should always be in valid range
        assert!(cpu_info.usage_percent >= 0.0 && cpu_info.usage_percent <= 100.0);

        // Cores might be empty if background thread hasn't initialized yet (test environment timing)
        // But if populated, each core should have valid usage
        for core in &cpu_info.cores {
            assert!(core.usage_percent >= 0.0 && core.usage_percent <= 100.0);
        }
    }

    #[test]
    fn test_memory_info() {
        let mem_info = get_memory_info();
        assert!(mem_info.total_bytes > 0);
        assert!(mem_info.used_bytes <= mem_info.total_bytes);
        assert!(mem_info.usage_percent >= 0.0 && mem_info.usage_percent <= 100.0);
    }

    #[test]
    fn test_disk_info() {
        let disks = get_disk_info();
        assert!(!disks.is_empty());
        for disk in disks {
            assert!(disk.total_bytes > 0);
            assert!(disk.usage_percent >= 0.0 && disk.usage_percent <= 100.0);
        }
    }

    #[test]
    fn test_network_info() {
        let networks = get_network_info();
        // May be empty on some systems
        for net in networks {
            assert!(!net.interface.is_empty());
        }
    }
}
