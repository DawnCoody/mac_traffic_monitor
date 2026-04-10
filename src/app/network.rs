use std::{net::IpAddr, process::Command};

use crate::app::types::{ConnectionDetails, ExternalIpDetails, InterfaceSelection};

pub fn default_connection_details() -> ConnectionDetails {
    ConnectionDetails {
        ipv4: "--".to_string(),
        ipv6: "获取失败".to_string(),
        subnet_mask: "--".to_string(),
        default_gateway: "--".to_string(),
    }
}

pub fn collect_connection_details(selection: &InterfaceSelection) -> ConnectionDetails {
    let mut details = default_connection_details();
    let mut ipv4_values = Vec::new();
    let mut ipv6_values = Vec::new();
    let mut subnet_values = Vec::new();
    let multi_interface = selection.names.len() > 1;

    for interface in &selection.names {
        let (ipv4, ipv6, subnet_mask) = query_interface_address_details(interface);

        if let Some(value) = ipv4 {
            ipv4_values.push(tagged_interface_value(interface, &value, multi_interface));
        }
        if let Some(value) = ipv6 {
            ipv6_values.push(tagged_interface_value(interface, &value, multi_interface));
        }
        if let Some(value) = subnet_mask {
            subnet_values.push(tagged_interface_value(interface, &value, multi_interface));
        }
    }

    if !ipv4_values.is_empty() {
        details.ipv4 = ipv4_values.join(" + ");
    }
    if !ipv6_values.is_empty() {
        details.ipv6 = ipv6_values.join(" + ");
    }
    if !subnet_values.is_empty() {
        details.subnet_mask = subnet_values.join(" + ");
    }

    if let Some(gateway) = query_default_gateway() {
        details.default_gateway = gateway;
    }

    details
}

fn tagged_interface_value(interface: &str, value: &str, include_interface: bool) -> String {
    if include_interface {
        format!("{interface}: {value}")
    } else {
        value.to_string()
    }
}

fn query_interface_address_details(
    interface: &str,
) -> (Option<String>, Option<String>, Option<String>) {
    let output = match Command::new("ifconfig").arg(interface).output() {
        Ok(output) => output,
        Err(_) => return (None, None, None),
    };
    if !output.status.success() {
        return (None, None, None);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut ipv4 = None;
    let mut ipv6_global = None;
    let mut ipv6_link_local = None;
    let mut subnet_mask = None;

    for line in stdout.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        let Some(first) = fields.first() else {
            continue;
        };

        if *first == "inet" {
            if ipv4.is_none() {
                ipv4 = fields.get(1).map(|value| (*value).to_string());
            }

            if subnet_mask.is_none()
                && let Some(mask_index) = fields.iter().position(|field| *field == "netmask")
                && let Some(mask_raw) = fields.get(mask_index + 1)
            {
                subnet_mask = Some(normalize_netmask(mask_raw));
            }
        } else if *first == "inet6" {
            let Some(raw_value) = fields.get(1) else {
                continue;
            };
            let value = raw_value.split('%').next().unwrap_or(raw_value);
            if value == "::1" {
                continue;
            }

            if value.starts_with("fe80:") {
                if ipv6_link_local.is_none() {
                    ipv6_link_local = Some(value.to_string());
                }
            } else if ipv6_global.is_none() {
                ipv6_global = Some(value.to_string());
            }
        }
    }

    (ipv4, ipv6_global.or(ipv6_link_local), subnet_mask)
}

fn normalize_netmask(mask_raw: &str) -> String {
    if let Some(hex_mask) = mask_raw.strip_prefix("0x")
        && let Ok(mask) = u32::from_str_radix(hex_mask, 16)
    {
        let octets = [
            ((mask >> 24) & 0xff) as u8,
            ((mask >> 16) & 0xff) as u8,
            ((mask >> 8) & 0xff) as u8,
            (mask & 0xff) as u8,
        ];
        return format!("{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3]);
    }

    mask_raw.to_string()
}

fn query_default_gateway() -> Option<String> {
    let output = Command::new("route")
        .args(["-n", "get", "default"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    parse_default_gateway_from_route_output(&output.stdout)
}

fn parse_default_gateway_from_route_output(stdout: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(stdout);
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some((key, value)) = trimmed.split_once(':')
            && key.trim().eq_ignore_ascii_case("gateway")
        {
            let gateway = value.trim();
            if !gateway.is_empty() {
                return Some(gateway.to_string());
            }
            continue;
        }

        let mut fields = trimmed.split_whitespace();
        let first = fields.next();
        let second = fields.next();
        if first.is_some_and(|field| field.eq_ignore_ascii_case("gateway")) && second.is_some() {
            return second.map(str::to_string);
        }
    }

    None
}

pub fn collect_external_ip_details() -> ExternalIpDetails {
    ExternalIpDetails {
        ipv4: query_external_ip("-4", "https://api.ipify.org", IpFamily::V4),
        ipv6: query_external_ip("-6", "https://api64.ipify.org", IpFamily::V6),
    }
}

#[derive(Clone, Copy)]
enum IpFamily {
    V4,
    V6,
}

fn query_external_ip(
    protocol_flag: &str,
    endpoint: &str,
    expected_family: IpFamily,
) -> Option<String> {
    let output = Command::new("curl")
        .args([protocol_flag, "-fsS", "--max-time", "4", endpoint])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    parse_external_ip_response(&output.stdout, expected_family)
}

fn parse_external_ip_response(stdout: &[u8], expected_family: IpFamily) -> Option<String> {
    let value = String::from_utf8_lossy(stdout).trim().to_string();
    if value.is_empty() {
        return None;
    }

    let ip = value.parse::<IpAddr>().ok()?;
    let family_matches = matches!(
        (expected_family, ip),
        (IpFamily::V4, IpAddr::V4(_)) | (IpFamily::V6, IpAddr::V6(_))
    );
    family_matches.then(|| ip.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_external_ip_response_accepts_matching_ipv4() {
        assert_eq!(
            parse_external_ip_response(b"1.2.3.4\n", IpFamily::V4),
            Some("1.2.3.4".to_string())
        );
    }

    #[test]
    fn parse_external_ip_response_rejects_ipv4_for_ipv6_slot() {
        assert_eq!(parse_external_ip_response(b"1.2.3.4\n", IpFamily::V6), None);
    }

    #[test]
    fn parse_external_ip_response_accepts_matching_ipv6() {
        assert_eq!(
            parse_external_ip_response(b"2001:db8::1\n", IpFamily::V6),
            Some("2001:db8::1".to_string())
        );
    }

    #[test]
    fn parse_default_gateway_from_route_output_standard_format() {
        let output = br#"
route to: default
destination: default
   mask: default
gateway: 192.168.10.1
interface: en0
"#;

        assert_eq!(
            parse_default_gateway_from_route_output(output),
            Some("192.168.10.1".to_string())
        );
    }

    #[test]
    fn parse_default_gateway_from_route_output_is_case_insensitive() {
        let output = b"Gateway:   10.0.0.1\n";

        assert_eq!(
            parse_default_gateway_from_route_output(output),
            Some("10.0.0.1".to_string())
        );
    }

    #[test]
    fn parse_default_gateway_from_route_output_returns_none_when_missing() {
        let output = b"destination: default\ninterface: en0\n";

        assert_eq!(parse_default_gateway_from_route_output(output), None);
    }
}
