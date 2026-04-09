use cocoa::{
    appkit::{NSApp, NSApplication, NSStatusBar, NSStatusItem, NSTextField, NSVariableStatusItemLength, NSView},
    base::{NO, id, nil},
    foundation::{NSPoint, NSRect, NSSize},
};
use objc::{class, msg_send, sel, sel_impl};

use crate::{app::constants::*, app::ui::menu::nsstring};

pub struct NativeStatusItem {
    status_item: id,
    container_view: id,
    top_left_label: id,
    top_right_label: id,
    bottom_left_label: id,
    bottom_right_label: id,
}

impl NativeStatusItem {
    pub fn new() -> Self {
        unsafe {
            let app = NSApp();
            app.setActivationPolicy_(cocoa::appkit::NSApplicationActivationPolicyRegular);

            let status_item =
                NSStatusBar::systemStatusBar(nil).statusItemWithLength_(NSVariableStatusItemLength);
            let _: () = msg_send![status_item, retain];

            let button = status_item.button();
            let _: () = msg_send![button, setImage: nil];
            let _: () = msg_send![button, setImagePosition: 0i64];
            let _: () = msg_send![button, setTitle: nsstring("")];

            let container_view = NSView::initWithFrame_(
                NSView::alloc(nil),
                NSRect::new(
                    NSPoint::new(0.0, 0.0),
                    NSSize::new(STATUS_VIEW_WIDTH, STATUS_VIEW_HEIGHT),
                ),
            );

            let top_left_label = make_status_label(TOP_LABEL_Y);
            let top_right_label = make_status_label(TOP_LABEL_Y);
            let bottom_left_label = make_status_label(BOTTOM_LABEL_Y);
            let bottom_right_label = make_status_label(BOTTOM_LABEL_Y);

            container_view.addSubview_(top_left_label);
            container_view.addSubview_(top_right_label);
            container_view.addSubview_(bottom_left_label);
            container_view.addSubview_(bottom_right_label);
            let _: () = msg_send![button, addSubview: container_view];

            let this = Self {
                status_item,
                container_view,
                top_left_label,
                top_right_label,
                bottom_left_label,
                bottom_right_label,
            };
            this.set_title(INITIAL_TITLE);
            this
        }
    }

    pub fn set_title(&self, title: &str) {
        let (line1, line2) = split_title_lines(title);
        let (line1_left, line1_right) = split_line_columns(line1);
        let (line2_left, line2_right) = split_line_columns(line2);
        unsafe {
            self.top_left_label.setStringValue_(nsstring(line1_left));
            self.top_right_label.setStringValue_(nsstring(line1_right));
            self.bottom_left_label.setStringValue_(nsstring(line2_left));
            self.bottom_right_label
                .setStringValue_(nsstring(line2_right));

            let left_width = title_visual_width(line1_left)
                .max(title_visual_width(line2_left))
                .max(STATUS_MIN_WIDTH);
            let right_width = title_visual_width(line1_right)
                .max(title_visual_width(line2_right))
                .max(26.0)
                + STATUS_RIGHT_COLUMN_EXTRA;
            let content_width = left_width + STATUS_COLUMN_GAP + right_width;
            let right_x = STATUS_HORIZONTAL_PADDING + left_width + STATUS_COLUMN_GAP;
            let width = content_width + STATUS_HORIZONTAL_PADDING * 2.0;
            let _: () = msg_send![self.container_view, setFrame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(width, STATUS_VIEW_HEIGHT))];
            let _: () = msg_send![self.top_left_label, setFrame: NSRect::new(NSPoint::new(STATUS_HORIZONTAL_PADDING, TOP_LABEL_Y), NSSize::new(left_width, LABEL_HEIGHT))];
            let _: () = msg_send![self.top_right_label, setFrame: NSRect::new(NSPoint::new(right_x, TOP_LABEL_Y), NSSize::new(right_width, LABEL_HEIGHT))];
            let _: () = msg_send![self.bottom_left_label, setFrame: NSRect::new(NSPoint::new(STATUS_HORIZONTAL_PADDING, BOTTOM_LABEL_Y), NSSize::new(left_width, LABEL_HEIGHT))];
            let _: () = msg_send![self.bottom_right_label, setFrame: NSRect::new(NSPoint::new(right_x, BOTTOM_LABEL_Y), NSSize::new(right_width, LABEL_HEIGHT))];
            self.status_item.setLength_(width);
            let tooltip = title.replace('\t', "  ");
            let _: () = msg_send![self.status_item.button(), setToolTip: nsstring(&tooltip)];
        }
    }

    pub fn set_menu(&self, menu: id) {
        unsafe {
            self.status_item.setMenu_(menu);
        }
    }
}

impl Drop for NativeStatusItem {
    fn drop(&mut self) {
        unsafe {
            self.container_view.removeFromSuperview();
            NSStatusBar::systemStatusBar(nil).removeStatusItem_(self.status_item);
            let _: () = msg_send![self.status_item, release];
        }
    }
}

fn make_status_label(y: f64) -> id {
    unsafe {
        let label = NSTextField::initWithFrame_(
            NSTextField::alloc(nil),
            NSRect::new(
                NSPoint::new(0.0, y),
                NSSize::new(STATUS_VIEW_WIDTH, LABEL_HEIGHT),
            ),
        );
        let font: id = msg_send![class!(NSFont), menuBarFontOfSize: STATUS_FONT_SIZE];
        let cell: id = msg_send![label, cell];
        let _: () = msg_send![cell, setFont: font];
        let _: () = msg_send![label, setEditable: NO];
        let _: () = msg_send![label, setBezeled: NO];
        let _: () = msg_send![label, setBordered: NO];
        let _: () = msg_send![label, setDrawsBackground: NO];
        let _: () = msg_send![label, setSelectable: NO];
        let _: () = msg_send![label, setAlignment: 0usize];
        label
    }
}

fn split_title_lines(title: &str) -> (&str, &str) {
    let mut lines = title.splitn(2, '\n');
    (lines.next().unwrap_or(""), lines.next().unwrap_or(""))
}

fn split_line_columns(line: &str) -> (&str, &str) {
    let mut columns = line.splitn(2, '\t');
    (columns.next().unwrap_or(""), columns.next().unwrap_or(""))
}

fn title_visual_width(line: &str) -> f64 {
    8.0 + line.chars().count() as f64 * 5.2
}
