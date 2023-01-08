use leaf::app::router;
use tauri::api::path::home_dir;
use tauri::api::process::Command;
use tauri::async_runtime::block_on;
use tauri::{AppHandle, Manager, SystemTrayEvent};
use tauri::{CustomMenuItem, SystemTrayMenu, SystemTrayMenuItem, SystemTraySubmenu};

use crate::setting::UserSettings;
use crate::states::{switch_mode_proxy, toggle_system_proxy, toggle_tun_mode};

pub(crate) fn build(settings: UserSettings) -> SystemTrayMenu {
    let mut menu = SystemTrayMenu::new()
        .add_submenu(SystemTraySubmenu::new(
            "出站模式",
            SystemTrayMenu::new()
                .add_item({
                    let item = CustomMenuItem::new("mode_proxy", "全局代理");
                    if settings.outbound_mode == router::Mode::Global.to_string() {
                        item.selected()
                    } else {
                        item
                    }
                })
                .add_item({
                    let item = CustomMenuItem::new("mode_match", "规则匹配");
                    if settings.outbound_mode == router::Mode::Match.to_string() {
                        item.selected()
                    } else {
                        item
                    }
                })
                .add_item({
                    let item = CustomMenuItem::new("mode_direct", "直接连接");
                    if settings.outbound_mode == router::Mode::Direct.to_string() {
                        item.selected()
                    } else {
                        item
                    }
                }),
        ))
        .add_native_item(SystemTrayMenuItem::Separator);

    for out in &settings.inner.proxy_group {
        let mut sub = SystemTrayMenu::new();
        if let Some(actors) = out.actors.as_ref() {
            for act in actors {
                sub = sub.add_item(CustomMenuItem::new(
                    format!("{}_{}", out.tag.clone(), act.clone()),
                    act,
                ));
            }
        }

        menu = menu.add_submenu(SystemTraySubmenu::new(out.tag.clone(), sub))
    }

    menu.add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("set_system_proxy", "设置为系统代理"))
        .add_item(CustomMenuItem::new("copy_proxy_cmd", "复制终端代理命令"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("configure", "打开配置目录"))
        .add_item(CustomMenuItem::new("set_auto_startup", "开机启动"))
        .add_item(CustomMenuItem::new("tun_mode", "TUN 模式"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit", "退出"))
}

pub(crate) fn handle_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        tauri::SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "quit" => app.exit(0),
            "mode_proxy" => {
                block_on(switch_mode_proxy(app.to_owned(), router::Mode::Global))
                    .expect("failed to switch routing mode");
            }
            "mode_match" => {
                block_on(switch_mode_proxy(app.to_owned(), router::Mode::Match))
                    .expect("failed to switch routing mode");
            }
            "mode_direct" => {
                block_on(switch_mode_proxy(app.to_owned(), router::Mode::Direct))
                    .expect("failed to switch routing mode");
            }
            "tun_mode" => toggle_tun_mode(app.to_owned()).expect("failed to toggle tun mode"),
            "set_system_proxy" => {
                toggle_system_proxy(app.to_owned()).expect("failed to toggle system proxy")
            }
            "configure" => {
                let s = home_dir().unwrap();
                let s = s.join(".config").join("leaf-bud");
                app.shell_scope().open(s.to_str().unwrap(), None).unwrap();
            }
            _ => {
                // TODO: switch proxy group by clicking.
            }
        },
        tauri::SystemTrayEvent::LeftClick { .. } => {}
        _ => {}
    }
}
