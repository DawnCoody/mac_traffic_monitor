use cocoa::{
    appkit::{NSApp, NSImage},
    base::{id, nil},
};
use objc::{class, msg_send, sel, sel_impl};

pub fn try_set_application_icon() {
    static ICON_BYTES: &[u8] = include_bytes!("../../assets/icons/rocket.icns");

    unsafe {
        let data: id = msg_send![
            class!(NSData),
            dataWithBytes: ICON_BYTES.as_ptr() as *const std::ffi::c_void
            length: ICON_BYTES.len()
        ];
        if data == nil {
            return;
        }

        let image: id = msg_send![NSImage::alloc(nil), initWithData: data];
        if image != nil {
            let _: () = msg_send![NSApp(), setApplicationIconImage: image];
            let _: () = msg_send![image, release];
        }
    }
}
