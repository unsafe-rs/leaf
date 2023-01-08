#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod delegate;
mod menu;
mod setting;
mod states;
mod system;

use log::LevelFilter;
use tauri::SystemTray;
use tauri_plugin_log::{LogTarget, LoggerBuilder};

fn main() {
    let mut app = tauri::Builder::default()
        .plugin(
            LoggerBuilder::default()
                .targets([LogTarget::LogDir, LogTarget::Stdout, LogTarget::Webview])
                .level(LevelFilter::Info)
                .build(),
        )
        .system_tray(SystemTray::new().with_menu(menu::build(Default::default())))
        .on_system_tray_event(menu::handle_event)
        .setup(delegate::setup)
        .invoke_handler(tauri::generate_handler![states::switch_mode_proxy,])
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Prohibited);

    app.run(|_app_handle, event| {
        if let tauri::RunEvent::ExitRequested { api, .. } = event {
            api.prevent_exit();
        }
    });
}
