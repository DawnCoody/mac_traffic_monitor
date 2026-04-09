use std::{env, fs, path::{Path, PathBuf}};

use crate::app::constants::{LAUNCH_AGENT_FILE_NAME, LAUNCH_AGENT_LABEL};

pub fn is_launch_at_login_enabled() -> bool {
    launch_agent_path().is_some_and(|path| path.exists())
}

pub fn set_launch_at_login(enabled: bool) -> Result<(), String> {
    if enabled {
        enable_launch_at_login()
    } else {
        disable_launch_at_login()
    }
}

fn launch_agent_path() -> Option<PathBuf> {
    let home = env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join("Library")
            .join("LaunchAgents")
            .join(LAUNCH_AGENT_FILE_NAME),
    )
}

fn enable_launch_at_login() -> Result<(), String> {
    let path = launch_agent_path().ok_or_else(|| "无法确定 HOME 目录".to_string())?;
    let exe_path = env::current_exe().map_err(|err| format!("读取当前程序路径失败: {err}"))?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("创建 LaunchAgents 目录失败: {err}"))?;
    }

    let plist = build_launch_agent_plist(&exe_path);
    fs::write(&path, plist).map_err(|err| format!("写入自启动配置失败: {err}"))
}

fn disable_launch_at_login() -> Result<(), String> {
    let Some(path) = launch_agent_path() else {
        return Ok(());
    };

    if !path.exists() {
        return Ok(());
    }

    fs::remove_file(&path).map_err(|err| format!("删除自启动配置失败: {err}"))
}

fn build_launch_agent_plist(executable_path: &Path) -> String {
    let exe = escape_xml_text(&executable_path.to_string_lossy());
    let label = escape_xml_text(LAUNCH_AGENT_LABEL);

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
</dict>
</plist>
"#
    )
}

fn escape_xml_text(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_xml_text_replaces_reserved_chars() {
        assert_eq!(
            escape_xml_text("a&b<c>d\"e'f"),
            "a&amp;b&lt;c&gt;d&quot;e&apos;f"
        );
    }
}
