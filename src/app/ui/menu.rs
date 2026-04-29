use std::ffi::CStr;

use cocoa::{
    appkit::{NSApp, NSMenu, NSMenuItem, NSTextField, NSView},
    base::{NO, id, nil},
    foundation::{NSPoint, NSRect, NSSize, NSString},
};
use objc::runtime::Sel;
use objc::{class, msg_send, sel, sel_impl};

use crate::app::{autostart::is_launch_at_login_enabled, constants::*, types::MenuItems};

pub fn make_copyable_item(initial_title: &str, action_target: id) -> id {
    make_clickable_menu_row(initial_title, action_target, sel!(copyValueFromGesture:))
}

pub fn make_refreshable_ip_item(initial_title: &str, action_target: id) -> id {
    make_clickable_menu_row(initial_title, action_target, sel!(refreshIpFromGesture:))
}

pub fn make_external_ip_item(initial_title: &str, action_target: id) -> id {
    unsafe {
        let item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
            nsstring(initial_title),
            sel!(refreshExternalIpFromGesture:),
            nsstring(""),
        );
        let _: () = msg_send![item, setEnabled: NO];
        let _: () = msg_send![item, setTarget: action_target];
        let container = make_menu_row_container(initial_title);

        let single_click: id = msg_send![class!(NSClickGestureRecognizer), alloc];
        let single_click: id = msg_send![
            single_click,
            initWithTarget: action_target
            action: sel!(copyValueFromGesture:)
        ];
        let _: () = msg_send![single_click, setNumberOfClicksRequired: 1usize];

        let double_click: id = msg_send![class!(NSClickGestureRecognizer), alloc];
        let double_click: id = msg_send![
            double_click,
            initWithTarget: action_target
            action: sel!(refreshExternalIpFromGesture:)
        ];
        let _: () = msg_send![double_click, setNumberOfClicksRequired: 2usize];
        let supports_dependency: bool =
            msg_send![single_click, respondsToSelector: sel!(requireGestureRecognizerToFail:)];
        if supports_dependency {
            let _: () = msg_send![single_click, requireGestureRecognizerToFail: double_click];
        }

        let _: () = msg_send![container, addGestureRecognizer: single_click];
        let _: () = msg_send![container, addGestureRecognizer: double_click];
        let _: () = msg_send![item, setView: container];
        item
    }
}

fn make_clickable_menu_row(initial_title: &str, action_target: id, action: Sel) -> id {
    unsafe {
        let item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
            nsstring(initial_title),
            action,
            nsstring(""),
        );
        let _: () = msg_send![item, setEnabled: NO];
        let _: () = msg_send![item, setTarget: action_target];
        let container = make_menu_row_container(initial_title);

        let recognizer: id = msg_send![class!(NSClickGestureRecognizer), alloc];
        let recognizer: id = msg_send![recognizer, initWithTarget: action_target action: action];
        let _: () = msg_send![recognizer, setNumberOfClicksRequired: 1usize];
        let _: () = msg_send![container, addGestureRecognizer: recognizer];
        let _: () = msg_send![item, setView: container];
        item
    }
}

fn make_menu_row_container(initial_title: &str) -> id {
    unsafe {
        let label_width = menu_label_column_width();
        let right_width = (MENU_ROW_WIDTH
            - MENU_ROW_HORIZONTAL_PADDING * 2.0
            - label_width
            - MENU_ROW_COLUMN_GAP)
            .max(24.0);
        let container = NSView::initWithFrame_(
            NSView::alloc(nil),
            NSRect::new(
                NSPoint::new(0.0, 0.0),
                NSSize::new(MENU_ROW_WIDTH, MENU_ROW_HEIGHT),
            ),
        );
        let left_label = NSTextField::initWithFrame_(
            NSTextField::alloc(nil),
            NSRect::new(
                NSPoint::new(MENU_ROW_HORIZONTAL_PADDING, 0.0),
                NSSize::new(label_width, MENU_ROW_HEIGHT),
            ),
        );
        let right_label = NSTextField::initWithFrame_(
            NSTextField::alloc(nil),
            NSRect::new(
                NSPoint::new(
                    MENU_ROW_HORIZONTAL_PADDING + label_width + MENU_ROW_COLUMN_GAP,
                    0.0,
                ),
                NSSize::new(right_width, MENU_ROW_HEIGHT),
            ),
        );
        let font: id = msg_send![class!(NSFont), menuFontOfSize: 0.0f64];
        let left_cell: id = msg_send![left_label, cell];
        let right_cell: id = msg_send![right_label, cell];
        let secondary_color: id = msg_send![class!(NSColor), secondaryLabelColor];

        let _: () = msg_send![left_label, setFont: font];
        let _: () = msg_send![right_label, setFont: font];
        let _: () = msg_send![left_cell, setLineBreakMode: 4usize];
        let _: () = msg_send![right_cell, setLineBreakMode: 4usize];
        let _: () = msg_send![left_label, setEditable: NO];
        let _: () = msg_send![right_label, setEditable: NO];
        let _: () = msg_send![left_label, setBezeled: NO];
        let _: () = msg_send![right_label, setBezeled: NO];
        let _: () = msg_send![left_label, setBordered: NO];
        let _: () = msg_send![right_label, setBordered: NO];
        let _: () = msg_send![left_label, setDrawsBackground: NO];
        let _: () = msg_send![right_label, setDrawsBackground: NO];
        let _: () = msg_send![left_label, setSelectable: NO];
        let _: () = msg_send![right_label, setSelectable: NO];
        let _: () = msg_send![left_label, setAlignment: 0usize];
        let _: () = msg_send![right_label, setAlignment: 2usize];
        let _: () = msg_send![left_label, setTextColor: secondary_color];
        let (left_text, right_text) = menu_columns_text(initial_title);
        let _: () = msg_send![left_label, setStringValue: nsstring(&left_text)];
        let _: () = msg_send![right_label, setStringValue: nsstring(&right_text)];
        container.addSubview_(left_label);
        container.addSubview_(right_label);
        container
    }
}

fn make_section_header_item(title: &str, action_target: id) -> id {
    unsafe {
        let item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
            nsstring(title),
            sel!(noop:),
            nsstring(""),
        );
        let _: () = msg_send![item, setEnabled: NO];
        let _: () = msg_send![item, setTarget: action_target];
        let container = NSView::initWithFrame_(
            NSView::alloc(nil),
            NSRect::new(
                NSPoint::new(0.0, 0.0),
                NSSize::new(MENU_ROW_WIDTH, MENU_ROW_HEIGHT),
            ),
        );
        let label = NSTextField::initWithFrame_(
            NSTextField::alloc(nil),
            NSRect::new(
                NSPoint::new(MENU_ROW_HORIZONTAL_PADDING, 0.0),
                NSSize::new(
                    MENU_ROW_WIDTH - MENU_ROW_HORIZONTAL_PADDING * 2.0,
                    MENU_ROW_HEIGHT,
                ),
            ),
        );
        let font: id = msg_send![class!(NSFont), boldSystemFontOfSize: 11.0f64];
        let color: id = msg_send![class!(NSColor), secondaryLabelColor];
        let _: () = msg_send![label, setFont: font];
        let _: () = msg_send![label, setTextColor: color];
        let _: () = msg_send![label, setEditable: NO];
        let _: () = msg_send![label, setBezeled: NO];
        let _: () = msg_send![label, setBordered: NO];
        let _: () = msg_send![label, setDrawsBackground: NO];
        let _: () = msg_send![label, setSelectable: NO];
        let _: () = msg_send![label, setAlignment: 0usize];
        let _: () = msg_send![label, setStringValue: nsstring(title)];
        container.addSubview_(label);
        let _: () = msg_send![item, setView: container];
        item
    }
}

fn make_toggle_item(item: id, action_target: id, action: Sel) {
    unsafe {
        let _: () = msg_send![item, setEnabled: 1u8];
        let _: () = msg_send![item, setTarget: action_target];
        let _: () = msg_send![item, setAction: action];
    }
}

pub fn build_native_menu(action_target: id) -> (id, MenuItems, id) {
    unsafe {
        let menu = NSMenu::alloc(nil).initWithTitle_(nsstring("Menu"));
        let _: () = msg_send![menu, setDelegate: action_target];

        menu.addItem_(make_section_header_item("系统资源", action_target));

        let memory_item = make_copyable_item("内存: --", action_target);
        menu.addItem_(memory_item);

        menu.addItem_(NSMenuItem::separatorItem(nil));

        menu.addItem_(make_section_header_item("网络配置", action_target));

        let ipv4_item = make_refreshable_ip_item("本机IP(v4): --", action_target);
        menu.addItem_(ipv4_item);

        let ipv6_item = make_refreshable_ip_item("本机IP(v6): 获取失败", action_target);
        menu.addItem_(ipv6_item);

        let subnet_item = make_copyable_item("子网掩码: --", action_target);
        menu.addItem_(subnet_item);

        let gateway_item = make_copyable_item("默认网关: --", action_target);
        menu.addItem_(gateway_item);

        let external_ipv4_item =
            make_external_ip_item("外网IP(v4): 自动获取中...", action_target);
        menu.addItem_(external_ipv4_item);

        let external_ipv6_item =
            make_external_ip_item("外网IP(v6): 自动获取中...", action_target);
        menu.addItem_(external_ipv6_item);

        menu.addItem_(NSMenuItem::separatorItem(nil));

        menu.addItem_(make_section_header_item("流量统计", action_target));

        let total_item = make_copyable_item("累计: ↑ --  ↓ --", action_target);
        menu.addItem_(total_item);

        let session_total_item = make_copyable_item("自程序启动以来: ↑ --  ↓ --", action_target);
        menu.addItem_(session_total_item);

        let uptime_item = make_copyable_item("程序已运行时间: --", action_target);
        menu.addItem_(uptime_item);

        menu.addItem_(NSMenuItem::separatorItem(nil));

        let autostart_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
            nsstring("开机自启动"),
            sel!(toggleLaunchAtLogin:),
            nsstring(""),
        );
        make_toggle_item(autostart_item, action_target, sel!(toggleLaunchAtLogin:));
        set_menu_item_checked(autostart_item, is_launch_at_login_enabled());
        menu.addItem_(autostart_item);

        menu.addItem_(NSMenuItem::separatorItem(nil));

        let quit_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
            nsstring("退出 (Quit)"),
            sel!(terminate:),
            nsstring("q"),
        );
        let _: () = msg_send![quit_item, setTarget: NSApp()];
        menu.addItem_(quit_item);

        let menu_items = MenuItems {
            memory_item,
            total_item,
            ipv4_item,
            ipv6_item,
            subnet_item,
            gateway_item,
            session_total_item,
            uptime_item,
            external_ipv4_item,
            external_ipv6_item,
        };

        (menu, menu_items, quit_item)
    }
}

pub fn set_menu_item_title(item: id, title: &str) {
    unsafe {
        let _: () = msg_send![item, setTitle: nsstring(title)];
        let view: id = msg_send![item, view];
        if view != nil {
            let subviews: id = msg_send![view, subviews];
            let count: usize = msg_send![subviews, count];
            if count > 1 {
                let left_label: id = msg_send![subviews, objectAtIndex: 0usize];
                let right_label: id = msg_send![subviews, objectAtIndex: 1usize];
                let left_supports: bool =
                    msg_send![left_label, respondsToSelector: sel!(setStringValue:)];
                let right_supports: bool =
                    msg_send![right_label, respondsToSelector: sel!(setStringValue:)];
                if left_supports && right_supports {
                    let (left_text, right_text) = menu_columns_text(title);
                    let _: () = msg_send![left_label, setStringValue: nsstring(&left_text)];
                    let _: () = msg_send![right_label, setStringValue: nsstring(&right_text)];
                    return;
                }
            }
            if count > 0 {
                let label: id = msg_send![subviews, objectAtIndex: 0usize];
                let supports_string_value: bool =
                    msg_send![label, respondsToSelector: sel!(setStringValue:)];
                if supports_string_value {
                    let _: () = msg_send![label, setStringValue: nsstring(title)];
                }
            }
        }
    }
}

fn menu_columns_text(title: &str) -> (String, String) {
    if let Some((label, value)) = title.split_once(':') {
        return (format!("{}:", label.trim()), value.trim().to_string());
    }
    if let Some((label, value)) = title.split_once('：') {
        return (format!("{}：", label.trim()), value.trim().to_string());
    }

    (title.to_string(), String::new())
}

fn menu_label_column_width() -> f64 {
    const LABELS: [&str; 10] = [
        "内存:",
        "累计:",
        "本机IP(v4):",
        "本机IP(v6):",
        "子网掩码:",
        "默认网关:",
        "自程序启动以来:",
        "程序已运行时间:",
        "外网IP(v4):",
        "外网IP(v6):",
    ];

    LABELS
        .iter()
        .map(|label| estimate_label_width(label))
        .fold(MENU_ROW_MIN_LABEL_WIDTH, f64::max)
}

fn estimate_label_width(label: &str) -> f64 {
    label
        .chars()
        .map(|ch| if ch.is_ascii() { 6.0 } else { 10.0 })
        .sum::<f64>()
        + 8.0
}

pub fn set_menu_item_checked(item: id, checked: bool) {
    unsafe {
        let state: i64 = if checked { 1 } else { 0 };
        let _: () = msg_send![item, setState: state];
    }
}

pub fn menu_item_is_checked(item: id) -> bool {
    let state: i64 = unsafe { msg_send![item, state] };
    state != 0
}

pub fn nsstring(value: &str) -> id {
    unsafe {
        let string = NSString::alloc(nil).init_str(value);
        msg_send![string, autorelease]
    }
}

pub fn nsstring_to_string(value: id) -> Option<String> {
    if value == nil {
        return None;
    }
    let c_str: *const std::os::raw::c_char = unsafe { msg_send![value, UTF8String] };
    if c_str.is_null() {
        return None;
    }

    Some(
        unsafe { CStr::from_ptr(c_str) }
            .to_string_lossy()
            .into_owned(),
    )
}
