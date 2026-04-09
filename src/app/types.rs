use cocoa::base::id;
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum UserEvent {
    StatsUpdated(UiState),
}

#[derive(Debug, Clone)]
pub struct UiState {
    pub title: String,
    pub memory_summary: String,
    pub total_summary: String,
    pub ipv4_summary: String,
    pub ipv6_summary: String,
    pub subnet_summary: String,
    pub gateway_summary: String,
    pub session_total_summary: String,
    pub uptime_summary: String,
    pub external_ipv4_summary: String,
    pub external_ipv6_summary: String,
}

#[derive(Debug, Clone)]
pub struct NetSnapshot {
    pub received: u64,
    pub transmitted: u64,
    pub captured_at: Instant,
}

#[derive(Debug, Clone)]
pub struct InterfaceSelection {
    pub names: Vec<String>,
    pub priority: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct NetRates {
    pub download_bps: u64,
    pub upload_bps: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct SystemMetrics {
    pub cpu_percent: u64,
    pub used_memory_bytes: u64,
    pub total_memory_bytes: u64,
}

#[derive(Debug, Clone, Default)]
pub struct ConnectionDetails {
    pub ipv4: String,
    pub ipv6: String,
    pub subnet_mask: String,
    pub default_gateway: String,
}

#[derive(Debug, Clone, Default)]
pub struct ExternalIpDetails {
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
}

pub struct MenuItems {
    pub memory_item: id,
    pub total_item: id,
    pub ipv4_item: id,
    pub ipv6_item: id,
    pub subnet_item: id,
    pub gateway_item: id,
    pub session_total_item: id,
    pub uptime_item: id,
    pub external_ipv4_item: id,
    pub external_ipv6_item: id,
}
