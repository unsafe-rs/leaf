use tauri::{AppHandle, SystemTrayEvent};
use tauri::{CustomMenuItem, SystemTrayMenu, SystemTrayMenuItem, SystemTraySubmenu};

use crate::states::{switch_mode_proxy, toggle_system_proxy, toggle_tun_mode};

pub(crate) fn default() -> SystemTrayMenu {
    SystemTrayMenu::new()
        .add_submenu(SystemTraySubmenu::new(
            "出站模式",
            SystemTrayMenu::new()
                .add_item(CustomMenuItem::new("mode_proxy", "全局代理").selected())
                .add_item(CustomMenuItem::new("mode_match", "规则匹配"))
                .add_item(CustomMenuItem::new("mode_direct", "直接连接")),
        ))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_submenu(SystemTraySubmenu::new(
            "Proxy",
            SystemTrayMenu::new().add_item(CustomMenuItem::new("latency", "延迟测速")),
        ))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("set_system_proxy", "设置为系统代理"))
        .add_item(CustomMenuItem::new("copy_proxy_cmd", "复制终端代理命令"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("set_auto_startup", "开机启动"))
        .add_item(CustomMenuItem::new("tun_mode", "TUN 模式"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit", "退出"))
}

pub(crate) fn handle_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        tauri::SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "quit" => app.exit(0),
            "mode_proxy" | "mode_match" | "mode_direct" => {
                switch_mode_proxy(
                    app.to_owned(),
                    match id.as_str() {
                        "mode_match" => crate::states::OutMode::Match,
                        "mode_direct" => crate::states::OutMode::Direct,
                        _ => crate::states::OutMode::Proxy,
                    },
                )
                .expect("failed to switch mode");
            }
            "tun_mode" => toggle_tun_mode(app.to_owned()).expect("failed to toggle tun mode"),
            "set_system_proxy" => {
                toggle_system_proxy(app.to_owned()).expect("failed to toggle system proxy")
            }
            _ => {}
        },
        tauri::SystemTrayEvent::LeftClick { .. } => {}
        _ => {}
    }
}
