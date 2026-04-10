use std::{
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Instant,
};

use sysinfo::{Networks, System};
use tao::event_loop::EventLoopProxy;

use crate::app::{
    constants::{DETAILS_REFRESH_INTERVAL, EXTERNAL_IP_REFRESH_INTERVAL, SAMPLE_INTERVAL, SMOOTHING_FACTOR},
    format::{
        format_external_ipv4_summary, format_external_ipv6_summary, format_gateway_summary,
        format_local_ipv4_summary, format_local_ipv6_summary, format_memory_summary,
        format_session_total_summary, format_status_title, format_subnet_summary,
        format_total_summary, format_uptime_summary,
    },
    network::{collect_connection_details, collect_external_ip_details, default_connection_details},
    types::{InterfaceSelection, NetRates, NetSnapshot, SystemMetrics, UiState, UserEvent},
};

pub fn spawn_monitor_worker(
    proxy: EventLoopProxy<UserEvent>,
    force_refresh_ip: &'static AtomicBool,
    external_ip_refreshing: &'static AtomicBool,
) {
    thread::spawn(move || {
        let started_at = Instant::now();
        let mut networks = Networks::new_with_refreshed_list();
        let mut system = System::new();
        let mut smoothed_rates: Option<NetRates> = None;
        let mut connection_details = default_connection_details();
        let mut details_refreshed_at = Instant::now() - DETAILS_REFRESH_INTERVAL;
        let mut external_ip_details = crate::app::types::ExternalIpDetails::default();
        let mut external_ip_refreshed_at = Instant::now() - EXTERNAL_IP_REFRESH_INTERVAL;

        system.refresh_cpu();
        system.refresh_memory();

        let mut selection = select_interfaces(&networks);
        let mut selection_profile = selection
            .as_ref()
            .map(|selected| (selected.priority, selected.names.len()));
        let mut previous = selection
            .as_ref()
            .and_then(|selected| collect_snapshot_for_selection(&networks, selected));
        let mut session_baseline = previous.clone();

        loop {
            thread::sleep(SAMPLE_INTERVAL);
            networks.refresh();
            system.refresh_cpu();
            system.refresh_memory();

            let current_profile = current_selection_profile(&networks);
            if current_profile != selection_profile {
                selection = select_interfaces(&networks);
                selection_profile = selection
                    .as_ref()
                    .map(|selected| (selected.priority, selected.names.len()));
                previous = selection
                    .as_ref()
                    .and_then(|selected| collect_snapshot_for_selection(&networks, selected));
                session_baseline = previous.clone();
                smoothed_rates = None;
                details_refreshed_at = Instant::now() - DETAILS_REFRESH_INTERVAL;
                external_ip_refreshed_at = Instant::now() - EXTERNAL_IP_REFRESH_INTERVAL;
                continue;
            }

            let Some(selected) = selection.as_ref() else {
                continue;
            };

            let Some(current_snapshot) = collect_snapshot_for_selection(&networks, selected) else {
                selection = select_interfaces(&networks);
                selection_profile = selection
                    .as_ref()
                    .map(|refreshed| (refreshed.priority, refreshed.names.len()));
                previous = selection
                    .as_ref()
                    .and_then(|refreshed| collect_snapshot_for_selection(&networks, refreshed));
                session_baseline = previous.clone();
                smoothed_rates = None;
                details_refreshed_at = Instant::now() - DETAILS_REFRESH_INTERVAL;
                external_ip_refreshed_at = Instant::now() - EXTERNAL_IP_REFRESH_INTERVAL;
                continue;
            };

            if force_refresh_ip.swap(false, Ordering::SeqCst) {
                connection_details = collect_connection_details(selected);
                details_refreshed_at = Instant::now();
                external_ip_details = collect_external_ip_details();
                external_ip_refreshed_at = Instant::now();
                external_ip_refreshing.store(false, Ordering::SeqCst);
            }

            if details_refreshed_at.elapsed() >= DETAILS_REFRESH_INTERVAL {
                connection_details = collect_connection_details(selected);
                details_refreshed_at = Instant::now();
            }

            if external_ip_refreshed_at.elapsed() >= EXTERNAL_IP_REFRESH_INTERVAL {
                external_ip_details = collect_external_ip_details();
                external_ip_refreshed_at = Instant::now();
            }

            let Some(previous_snapshot) = previous.as_ref() else {
                previous = Some(current_snapshot);
                session_baseline = previous.clone();
                continue;
            };

            let raw_rates = compute_rates(previous_snapshot, &current_snapshot);
            let display_rates = smooth_rates(smoothed_rates, raw_rates);
            let metrics = collect_system_metrics(&system);

            smoothed_rates = Some(display_rates);
            previous = Some(current_snapshot.clone());

            let (external_ipv4_summary, external_ipv6_summary) =
                if external_ip_refreshing.load(Ordering::SeqCst) {
                    (
                        "外网IP(v4): 刷新中...".to_string(),
                        "外网IP(v6): 刷新中...".to_string(),
                    )
                } else {
                    (
                        format_external_ipv4_summary(&external_ip_details),
                        format_external_ipv6_summary(&external_ip_details),
                    )
                };

            let state = UiState {
                title: format_status_title(display_rates, metrics),
                memory_summary: format_memory_summary(metrics),
                total_summary: format_total_summary(&current_snapshot),
                ipv4_summary: format_local_ipv4_summary(&connection_details),
                ipv6_summary: format_local_ipv6_summary(&connection_details),
                subnet_summary: format_subnet_summary(&connection_details),
                gateway_summary: format_gateway_summary(&connection_details),
                session_total_summary: format_session_total_summary(
                    session_baseline.as_ref(),
                    &current_snapshot,
                ),
                uptime_summary: format_uptime_summary(started_at),
                external_ipv4_summary,
                external_ipv6_summary,
            };

            if proxy.send_event(UserEvent::StatsUpdated(state)).is_err() {
                break;
            }
        }
    });
}

fn select_interfaces(networks: &Networks) -> Option<InterfaceSelection> {
    let mut selected_names = Vec::new();
    let mut selected_priority: Option<u8> = None;

    for (name, _) in networks.iter() {
        let priority = interface_priority(name);
        match selected_priority {
            None => {
                selected_priority = Some(priority);
                selected_names.push(name.to_string());
            }
            Some(best) if priority < best => {
                selected_priority = Some(priority);
                selected_names.clear();
                selected_names.push(name.to_string());
            }
            Some(best) if priority == best => {
                selected_names.push(name.to_string());
            }
            Some(_) => {}
        }
    }

    let priority = selected_priority?;

    selected_names.sort_unstable();

    Some(InterfaceSelection {
        names: selected_names,
        priority,
    })
}

fn current_selection_profile(networks: &Networks) -> Option<(u8, usize)> {
    let mut selected_priority: Option<u8> = None;
    let mut count = 0usize;

    for (name, _) in networks.iter() {
        let priority = interface_priority(name);
        match selected_priority {
            None => {
                selected_priority = Some(priority);
                count = 1;
            }
            Some(best) if priority < best => {
                selected_priority = Some(priority);
                count = 1;
            }
            Some(best) if priority == best => {
                count += 1;
            }
            Some(_) => {}
        }
    }

    selected_priority.map(|priority| (priority, count))
}

fn collect_snapshot_for_selection(
    networks: &Networks,
    selection: &InterfaceSelection,
) -> Option<NetSnapshot> {
    let captured_at = Instant::now();
    let mut received = 0_u64;
    let mut transmitted = 0_u64;

    for interface_name in &selection.names {
        let data = networks.get(interface_name.as_str())?;
        received = received.saturating_add(data.total_received());
        transmitted = transmitted.saturating_add(data.total_transmitted());
    }

    Some(NetSnapshot {
        received,
        transmitted,
        captured_at,
    })
}

fn collect_system_metrics(system: &System) -> SystemMetrics {
    let used_memory_bytes =
        collect_macos_used_memory_bytes().unwrap_or_else(|| system.used_memory());

    SystemMetrics {
        cpu_percent: system.global_cpu_info().cpu_usage().round() as u64,
        used_memory_bytes,
        total_memory_bytes: system.total_memory(),
    }
}

fn collect_macos_used_memory_bytes() -> Option<u64> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("vm_stat").output().ok()?;
        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut lines = stdout.lines();
        let page_size_bytes = lines
            .next()
            .and_then(parse_vm_stat_page_size)
            .unwrap_or(16_384);

        let mut anonymous_pages = None;
        let mut wired_pages = None;
        let mut compressor_pages = None;

        for line in lines {
            let trimmed = line.trim_start();

            if anonymous_pages.is_none()
                && let Some(raw) = trimmed.strip_prefix("Anonymous pages:")
            {
                anonymous_pages = parse_vm_stat_count(raw);
                continue;
            }
            if wired_pages.is_none()
                && let Some(raw) = trimmed.strip_prefix("Pages wired down:")
            {
                wired_pages = parse_vm_stat_count(raw);
                continue;
            }
            if compressor_pages.is_none()
                && let Some(raw) = trimmed.strip_prefix("Pages occupied by compressor:")
            {
                compressor_pages = parse_vm_stat_count(raw);
                continue;
            }
        }

        let anonymous = anonymous_pages?;
        let wired = wired_pages?;
        let compressor = compressor_pages?;
        let used_pages = anonymous.saturating_add(wired).saturating_add(compressor);

        Some(used_pages.saturating_mul(page_size_bytes))
    }

    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

fn parse_vm_stat_page_size(line: &str) -> Option<u64> {
    line.split("page size of ")
        .nth(1)
        .and_then(|part| part.split_whitespace().next())
        .and_then(|value| value.parse::<u64>().ok())
}

fn parse_vm_stat_count(raw: &str) -> Option<u64> {
    let digits: String = raw.chars().filter(|ch| ch.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse::<u64>().ok()
    }
}

fn interface_priority(name: &str) -> u8 {
    if prefer_primary_physical_interface(name) {
        0
    } else if is_physical_interface(name) && !should_ignore_interface(name) {
        1
    } else if !should_ignore_interface(name) {
        2
    } else {
        3
    }
}

fn prefer_primary_physical_interface(name: &str) -> bool {
    matches!(name, "en0" | "en1" | "en2")
}

fn is_physical_interface(name: &str) -> bool {
    name.starts_with("en")
}

pub fn compute_rates(previous: &NetSnapshot, current: &NetSnapshot) -> NetRates {
    let elapsed = current
        .captured_at
        .saturating_duration_since(previous.captured_at)
        .as_secs_f64()
        .max(0.001);

    let download_delta = current.received.saturating_sub(previous.received) as f64;
    let upload_delta = current.transmitted.saturating_sub(previous.transmitted) as f64;

    NetRates {
        download_bps: (download_delta / elapsed).round() as u64,
        upload_bps: (upload_delta / elapsed).round() as u64,
    }
}

pub fn smooth_rates(previous: Option<NetRates>, current: NetRates) -> NetRates {
    let Some(previous) = previous else {
        return current;
    };

    NetRates {
        download_bps: smooth_value(previous.download_bps, current.download_bps),
        upload_bps: smooth_value(previous.upload_bps, current.upload_bps),
    }
}

fn smooth_value(previous: u64, current: u64) -> u64 {
    let previous = previous as f64;
    let current = current as f64;
    (previous * (1.0 - SMOOTHING_FACTOR) + current * SMOOTHING_FACTOR).round() as u64
}

fn should_ignore_interface(name: &str) -> bool {
    [
        "lo", "bridge", "awdl", "llw", "utun", "anpi", "gif", "stf", "vmenet", "vmnet",
    ]
    .iter()
    .any(|prefix| name.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn compute_rates_uses_elapsed_seconds() {
        let base = Instant::now();
        let previous = NetSnapshot {
            received: 1_000,
            transmitted: 2_000,
            captured_at: base,
        };
        let current = NetSnapshot {
            received: 3_000,
            transmitted: 5_000,
            captured_at: base + Duration::from_secs(2),
        };

        let rates = compute_rates(&previous, &current);
        assert_eq!(rates.download_bps, 1_000);
        assert_eq!(rates.upload_bps, 1_500);
    }

    #[test]
    fn smooth_rates_returns_current_without_previous() {
        let current = NetRates {
            download_bps: 2_048,
            upload_bps: 1_024,
        };

        let smoothed = smooth_rates(None, current);
        assert_eq!(smoothed.download_bps, 2_048);
        assert_eq!(smoothed.upload_bps, 1_024);
    }

    #[test]
    fn smooth_rates_applies_weighted_average() {
        let previous = NetRates {
            download_bps: 1_000,
            upload_bps: 2_000,
        };
        let current = NetRates {
            download_bps: 2_000,
            upload_bps: 1_000,
        };

        let smoothed = smooth_rates(Some(previous), current);
        assert_eq!(smoothed.download_bps, 1_350);
        assert_eq!(smoothed.upload_bps, 1_650);
    }
}
