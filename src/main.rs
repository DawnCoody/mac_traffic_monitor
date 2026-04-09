use cocoa::appkit::{NSApp, NSApplication};
use tao::{
    event::{Event, StartCause},
    event_loop::{ControlFlow, EventLoopBuilder},
};

mod app;

use app::icon::try_set_application_icon;
use app::monitor::spawn_monitor_worker;
use app::types::UserEvent;
use app::ui::{
    actions::{create_menu_action_target, force_refresh_ip_flag},
    menu::{build_native_menu, set_menu_item_title},
    status_item::NativeStatusItem,
};


fn main() {
    unsafe {
        let app = NSApp();
        app.setActivationPolicy_(cocoa::appkit::NSApplicationActivationPolicyRegular);
    }
    try_set_application_icon();

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let status_item = NativeStatusItem::new();
    let menu_action_target = create_menu_action_target();
    let (menu, menu_items, _quit_item) = build_native_menu(menu_action_target);
    status_item.set_menu(menu);

    spawn_monitor_worker(proxy, force_refresh_ip_flag());

    event_loop.run(move |event, _, control_flow| {
        let _keep_action_target_alive = &menu_action_target;
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {}
            Event::UserEvent(UserEvent::StatsUpdated(state)) => {
                status_item.set_title(&state.title);
                set_menu_item_title(menu_items.memory_item, &state.memory_summary);
                set_menu_item_title(menu_items.total_item, &state.total_summary);
                set_menu_item_title(menu_items.ipv4_item, &state.ipv4_summary);
                set_menu_item_title(menu_items.ipv6_item, &state.ipv6_summary);
                set_menu_item_title(menu_items.subnet_item, &state.subnet_summary);
                set_menu_item_title(menu_items.gateway_item, &state.gateway_summary);
                set_menu_item_title(menu_items.session_total_item, &state.session_total_summary);
                set_menu_item_title(menu_items.uptime_item, &state.uptime_summary);
                set_menu_item_title(menu_items.external_ipv4_item, &state.external_ipv4_summary);
                set_menu_item_title(menu_items.external_ipv6_item, &state.external_ipv6_summary);
            }
            Event::LoopDestroyed => {}
            _ => {}
        }
    });
}


