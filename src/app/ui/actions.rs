use std::sync::{
    Mutex, Once,
    atomic::{AtomicBool, Ordering},
};

use cocoa::{
    appkit::{NSPasteboard, NSPasteboardTypeString},
    base::{id, nil},
};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};

use crate::app::{
    autostart::{is_launch_at_login_enabled, set_launch_at_login},
    constants::COPY_FEEDBACK_SUFFIX,
    ui::menu,
};

static FORCE_REFRESH_IP: AtomicBool = AtomicBool::new(false);
static LAST_COPIED_ITEM: Mutex<Option<usize>> = Mutex::new(None);

pub fn force_refresh_ip_flag() -> &'static AtomicBool {
    &FORCE_REFRESH_IP
}

pub fn create_menu_action_target() -> id {
    unsafe {
        let class = menu_action_target_class();
        msg_send![class, new]
    }
}

fn menu_action_target_class() -> &'static Class {
    static REGISTER: Once = Once::new();
    static mut CLASS: *const Class = std::ptr::null();

    REGISTER.call_once(|| unsafe {
        let superclass = class!(NSObject);
        let mut decl =
            ClassDecl::new("TrafficMonitorMenuActionTarget", superclass).expect("create class");
        decl.add_method(
            sel!(copyValueFromGesture:),
            copy_value_from_gesture as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(refreshIpFromGesture:),
            refresh_ip_from_gesture as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(toggleLaunchAtLogin:),
            toggle_launch_at_login as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(menuDidClose:),
            menu_did_close as extern "C" fn(&Object, Sel, id),
        );
        CLASS = decl.register();
    });

    unsafe { &*CLASS }
}

extern "C" fn copy_value_from_gesture(_: &Object, _: Sel, sender: id) {
    let Some(item) = menu_item_from_gesture_sender(sender) else {
        return;
    };
    mark_item_as_copied(item);
    copy_item_value_to_pasteboard(item);
}

extern "C" fn refresh_ip_from_gesture(_: &Object, _: Sel, sender: id) {
    let Some(item) = menu_item_from_gesture_sender(sender) else {
        return;
    };
    mark_item_as_copied(item);
    copy_item_value_to_pasteboard(item);
    FORCE_REFRESH_IP.store(true, Ordering::SeqCst);
}

extern "C" fn toggle_launch_at_login(_: &Object, _: Sel, sender: id) {
    let target_enabled = !menu::menu_item_is_checked(sender);
    if let Err(err) = set_launch_at_login(target_enabled) {
        eprintln!("failed to toggle launch at login: {err}");
    }
    menu::set_menu_item_checked(sender, is_launch_at_login_enabled());
}

extern "C" fn menu_did_close(_: &Object, _: Sel, _: id) {
    clear_last_copied_feedback();
}

fn menu_item_from_gesture_sender(sender: id) -> Option<id> {
    if sender == nil {
        return None;
    }
    let view: id = unsafe { msg_send![sender, view] };
    if view == nil {
        return None;
    }
    let item: id = unsafe { msg_send![view, enclosingMenuItem] };
    if item == nil { None } else { Some(item) }
}

fn clear_copy_feedback_from_item(item: id) {
    if item == nil {
        return;
    }

    let title: id = unsafe { msg_send![item, title] };
    let Some(full_text) = menu::nsstring_to_string(title) else {
        return;
    };
    let clean_text = strip_copy_feedback_suffix(&full_text);
    if clean_text != full_text {
        menu::set_menu_item_title(item, clean_text);
    }
}

fn mark_item_as_copied(item: id) {
    if let Ok(mut last_copied_item) = LAST_COPIED_ITEM.lock() {
        if let Some(previous) = *last_copied_item {
            let previous_item = previous as id;
            if previous_item != item {
                clear_copy_feedback_from_item(previous_item);
            }
        }
        *last_copied_item = Some(item as usize);
    }
}

fn clear_last_copied_feedback() {
    if let Ok(mut last_copied_item) = LAST_COPIED_ITEM.lock() {
        if let Some(previous) = last_copied_item.take() {
            clear_copy_feedback_from_item(previous as id);
        }
    }
}

fn copy_item_value_to_pasteboard(item: id) {
    let title: id = unsafe { msg_send![item, title] };
    let Some(full_text) = menu::nsstring_to_string(title) else {
        return;
    };
    let clean_text = strip_copy_feedback_suffix(&full_text);
    let value_text = extract_second_column(clean_text);

    unsafe {
        let pasteboard = NSPasteboard::generalPasteboard(nil);
        let _ = pasteboard.clearContents();
        let _ = pasteboard.setString_forType(menu::nsstring(value_text), NSPasteboardTypeString);
    }

    menu::set_menu_item_title(item, &with_copy_feedback_suffix(clean_text));
}

fn extract_second_column(text: &str) -> &str {
    text.split_once(':')
        .map(|(_, value)| value.trim())
        .unwrap_or(text)
}

fn with_copy_feedback_suffix(title: &str) -> String {
    let trimmed = strip_copy_feedback_suffix(title);
    format!("{trimmed}{COPY_FEEDBACK_SUFFIX}")
}

fn strip_copy_feedback_suffix(title: &str) -> &str {
    title.trim_end_matches(COPY_FEEDBACK_SUFFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_feedback_suffix_is_idempotent() {
        let base = "CPU: 12%";
        let once = with_copy_feedback_suffix(base);
        let twice = with_copy_feedback_suffix(&once);

        assert_eq!(once, "CPU: 12%  [已复制]");
        assert_eq!(twice, "CPU: 12%  [已复制]");
        assert_eq!(strip_copy_feedback_suffix(&once), base);
    }
}
