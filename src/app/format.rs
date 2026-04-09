use std::time::{Duration, Instant};

use crate::app::types::{ConnectionDetails, ExternalIpDetails, NetRates, NetSnapshot, SystemMetrics};

pub fn format_status_title(rates: NetRates, metrics: SystemMetrics) -> String {
    let upload = format_tray_rate(rates.upload_bps);
    let download = format_tray_rate(rates.download_bps);
    let cpu = format_tray_cpu_percent(metrics.cpu_percent);
    let memory_percent = format_tray_memory_percent(metrics);

    format!("↑:{upload}\tC:{cpu}%\n↓:{download}\tM:{memory_percent}%")
}

pub fn format_tray_rate(bytes_per_second: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;

    let value = bytes_per_second as f64;

    if value >= GB {
        format!("{:.1}GB/s", value / GB)
    } else if value >= MB {
        format!("{:.1}MB/s", value / MB)
    } else if value >= KB {
        format!("{:.1}KB/s", value / KB)
    } else {
        format!("{:.0}B/s", value)
    }
}

pub fn format_tray_cpu_percent(percent: u64) -> String {
    percent.min(99).to_string()
}

pub fn format_tray_memory_percent(metrics: SystemMetrics) -> String {
    if metrics.total_memory_bytes == 0 {
        return "0".to_string();
    }

    let percent = ((metrics.used_memory_bytes as f64 / metrics.total_memory_bytes as f64) * 100.0)
        .round() as u64;
    percent.min(100).to_string()
}

pub fn format_memory_summary(metrics: SystemMetrics) -> String {
    let percent = format_tray_memory_percent(metrics);
    format!(
        "内存: {} / {} ({}%)",
        format_memory_bytes(metrics.used_memory_bytes),
        format_memory_bytes(metrics.total_memory_bytes),
        percent
    )
}

pub fn format_total_summary(snapshot: &NetSnapshot) -> String {
    format!(
        "累计: ↑ {}  ↓ {}",
        format_total_bytes(snapshot.transmitted),
        format_total_bytes(snapshot.received)
    )
}

pub fn format_local_ipv4_summary(details: &ConnectionDetails) -> String {
    format!("本机IP(v4): {}", details.ipv4)
}

pub fn format_local_ipv6_summary(details: &ConnectionDetails) -> String {
    format!("本机IP(v6): {}", details.ipv6)
}

pub fn format_subnet_summary(details: &ConnectionDetails) -> String {
    format!("子网掩码: {}", details.subnet_mask)
}

pub fn format_gateway_summary(details: &ConnectionDetails) -> String {
    format!("默认网关: {}", details.default_gateway)
}

pub fn format_session_total_summary(
    baseline: Option<&NetSnapshot>,
    current_snapshot: &NetSnapshot,
) -> String {
    let Some(baseline) = baseline else {
        return "自程序启动以来: ↑ --  ↓ --".to_string();
    };

    let session_upload = current_snapshot
        .transmitted
        .saturating_sub(baseline.transmitted);
    let session_download = current_snapshot.received.saturating_sub(baseline.received);

    format!(
        "自程序启动以来: ↑ {}  ↓ {}",
        format_total_bytes(session_upload),
        format_total_bytes(session_download)
    )
}

pub fn format_uptime_summary(started_at: Instant) -> String {
    format!(
        "程序已运行时间: {}",
        format_elapsed_duration(started_at.elapsed())
    )
}

pub fn format_elapsed_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{hours}小时{minutes}分{seconds}秒")
    } else if minutes > 0 {
        format!("{minutes}分{seconds}秒")
    } else {
        format!("{seconds}秒")
    }
}

pub fn format_external_ipv4_summary(details: &ExternalIpDetails) -> String {
    let value = details.ipv4.as_deref().unwrap_or("获取失败");
    format!("外网IP(v4): {value}")
}

pub fn format_external_ipv6_summary(details: &ExternalIpDetails) -> String {
    let value = details.ipv6.as_deref().unwrap_or("获取失败");
    format!("外网IP(v6): {value}")
}

pub fn format_total_bytes(bytes: u64) -> String {
    format_bytes(bytes, false, false)
}

pub fn format_memory_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    const TB: f64 = 1024.0 * 1024.0 * 1024.0 * 1024.0;

    let value = bytes as f64;

    if value >= TB {
        format!("{:.2}TB", value / TB)
    } else if value >= GB {
        format!("{:.2}GB", value / GB)
    } else if value >= MB {
        format!("{:.2}MB", value / MB)
    } else if value >= KB {
        format!("{:.2}KB", value / KB)
    } else {
        format!("{:.0}B", value)
    }
}

fn format_bytes(bytes: u64, per_second: bool, compact: bool) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    const TB: f64 = 1024.0 * 1024.0 * 1024.0 * 1024.0;

    let value = bytes as f64;
    let suffix = if per_second { "/s" } else { "" };

    if value >= TB {
        format_value(value / TB, "T", suffix, compact)
    } else if value >= GB {
        format_value(value / GB, "G", suffix, compact)
    } else if value >= MB {
        format_value(value / MB, "M", suffix, compact)
    } else if value >= KB {
        format_value(value / KB, "K", suffix, compact)
    } else {
        format!("{:.0}B{suffix}", value)
    }
}

fn format_value(value: f64, unit: &str, suffix: &str, compact: bool) -> String {
    if compact {
        if value >= 10.0 {
            format!("{:.0}{unit}{suffix}", value)
        } else {
            format!("{:.1}{unit}{suffix}", value)
        }
    } else if value >= 10.0 {
        format!("{:.0}{unit}{suffix}", value)
    } else {
        format!("{:.1}{unit}{suffix}", value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_tray_rate_handles_common_units() {
        assert_eq!(format_tray_rate(512), "512B/s");
        assert_eq!(format_tray_rate(1_024), "1.0KB/s");
        assert_eq!(format_tray_rate(1_024 * 1_024), "1.0MB/s");
    }

    #[test]
    fn tray_cpu_percent_is_capped() {
        let metrics = SystemMetrics {
            cpu_percent: 100,
            used_memory_bytes: 100,
            total_memory_bytes: 100,
        };

        assert_eq!(format_tray_cpu_percent(metrics.cpu_percent), "99");
    }

    #[test]
    fn tray_memory_percent_rounds_to_integer() {
        let metrics_high = SystemMetrics {
            cpu_percent: 0,
            used_memory_bytes: 995,
            total_memory_bytes: 1_000,
        };
        let metrics_normal = SystemMetrics {
            cpu_percent: 0,
            used_memory_bytes: 994,
            total_memory_bytes: 1_000,
        };
        let metrics_full = SystemMetrics {
            cpu_percent: 0,
            used_memory_bytes: 1_000,
            total_memory_bytes: 1_000,
        };

        assert_eq!(format_tray_memory_percent(metrics_high), "100");
        assert_eq!(format_tray_memory_percent(metrics_normal), "99");
        assert_eq!(format_tray_memory_percent(metrics_full), "100");
    }

    #[test]
    fn tray_memory_percent_handles_zero_total_memory() {
        let metrics = SystemMetrics {
            cpu_percent: 0,
            used_memory_bytes: 0,
            total_memory_bytes: 0,
        };

        assert_eq!(format_tray_memory_percent(metrics), "0");
    }

    #[test]
    fn format_total_bytes_uses_binary_units() {
        assert_eq!(format_total_bytes(1_536), "1.5K");
        assert_eq!(format_total_bytes(10 * 1_024), "10K");
    }

    #[test]
    fn format_memory_summary_keeps_two_decimals() {
        let metrics = SystemMetrics {
            cpu_percent: 0,
            used_memory_bytes: 3 * 1024 * 1024 * 1024 + 512 * 1024 * 1024,
            total_memory_bytes: 8 * 1024 * 1024 * 1024,
        };

        assert_eq!(
            format_memory_summary(metrics),
            "内存: 3.50GB / 8.00GB (44%)"
        );
    }
}
