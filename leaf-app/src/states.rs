use std::sync::Arc;

use leaf::{config::json, relay::RelayManager};
use serde::{Deserialize, Serialize};
use tauri::{Manager, Result, Runtime};

use crate::macos;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
pub enum OutMode {
    Proxy,
    Match,
    Direct,
}

impl Default for OutMode {
    fn default() -> Self {
        OutMode::Proxy
    }
}

pub struct UserSetting {
    pub mode: std::sync::Mutex<OutMode>,
    pub tun: std::sync::Mutex<bool>,
    pub system_proxy: std::sync::Mutex<bool>,
    pub rm: RelayManager,
    pub rmc: std::sync::Mutex<json::Config>,
}

#[tauri::command]
pub fn switch_mode_proxy<R: Runtime>(app: tauri::AppHandle<R>, mode: OutMode) -> Result<()> {
    let state = app.state::<UserSetting>();
    *state.mode.lock().unwrap() = mode;

    let h = app.tray_handle();
    h.get_item("mode_proxy")
        .set_selected(mode == OutMode::Proxy)?;
    h.get_item("mode_match")
        .set_selected(mode == OutMode::Match)?;
    h.get_item("mode_direct")
        .set_selected(mode == OutMode::Direct)?;
    Ok(())
}

#[tauri::command]
pub fn toggle_tun_mode<R: Runtime>(app: tauri::AppHandle<R>) -> Result<()> {
    let state = app.state::<UserSetting>();
    let v = *state.tun.lock().unwrap();
    *state.tun.lock().unwrap() = !v;

    let h = app.tray_handle();
    h.get_item("tun_mode").set_selected(v)?;
    Ok(())
}

#[tauri::command]
pub fn toggle_system_proxy<R: Runtime>(app: tauri::AppHandle<R>) -> Result<()> {
    let state = app.state::<UserSetting>();
    let is = (*state.system_proxy.lock().unwrap()).clone();
    *state.system_proxy.lock().unwrap() = !is;

    // let general = (&state.rmc.lock().unwrap()).inbounds.unwrap_or_default().iter().find(|x| x.);

    if !is {
        macos::set_system_proxy(1087, 1086).expect("set system proxy failed");
    } else {
        macos::unset_system_proxy().expect("unset system proxy failed");
    }

    app.tray_handle()
        .get_item("set_system_proxy")
        .set_selected(!is)?;
    Ok(())
}
