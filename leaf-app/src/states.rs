use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use leaf::{app::router, relay::RelayManager};
use log::info;
use tauri::{api::path::home_dir, async_runtime::block_on, Manager, Result, Runtime};

use crate::{setting, system};

pub struct CoreState {
    pub rm: Arc<Mutex<RelayManager>>,
    pub settings: Arc<Mutex<setting::UserSettings>>,
}

#[tauri::command]
pub fn switch_mode_proxy<R: Runtime>(app: tauri::AppHandle<R>, mode: router::Mode) -> Result<()> {
    let state = app.state::<CoreState>();
    state.settings.lock().unwrap().outbound_mode = mode.to_string();
    let mut rm = state.rm.lock().unwrap();
    block_on(rm.route_manager().write()).set_mode(mode);

    let h = app.tray_handle();
    h.get_item("mode_proxy")
        .set_selected(mode == router::Mode::Global)?;
    h.get_item("mode_match")
        .set_selected(mode == router::Mode::Match)?;
    h.get_item("mode_direct")
        .set_selected(mode == router::Mode::Direct)?;

    Ok(())
}

#[tauri::command]
pub fn toggle_tun_mode<R: Runtime>(app: tauri::AppHandle<R>) -> Result<()> {
    let state = app.state::<CoreState>();
    let mut v = state.settings.lock().unwrap();
    let on = v.inner.general.tun_auto.or(Some(false));
    v.inner.general.tun_auto = Some(!on.unwrap_or(false));

    let h = app.tray_handle();
    h.get_item("tun_mode")
        .set_selected(v.inner.general.tun_auto.unwrap_or(false))?;
    Ok(())
}

#[tauri::command]
pub fn toggle_system_proxy<R: Runtime>(app: tauri::AppHandle<R>) -> Result<()> {
    let state = app.state::<CoreState>();
    let mut settings = state.settings.lock().unwrap();
    settings.system_proxy = !settings.system_proxy;

    let http_port = settings.inner.general.http_port;
    let socks_port = settings.inner.general.socks_port;

    if settings.system_proxy {
        system::set_system_proxy(http_port, socks_port).expect("set system proxy failed");
    } else {
        system::unset_system_proxy().expect("unset system proxy failed");
    }

    app.tray_handle()
        .get_item("set_system_proxy")
        .set_selected(settings.system_proxy)?;
    Ok(())
}

#[tauri::command]
pub fn reload_config<R: Runtime>(app: tauri::AppHandle<R>) -> Result<()> {
    let state = app.state::<CoreState>();
    let mut prev = state.settings.lock().unwrap();

    let dst = Path::new(&home_dir().unwrap())
        .join(".config")
        .join("leaf-bud")
        .join("bud.conf");
    let next = setting::UserSettings::from_file(dst).unwrap();

    prev.inner = next.inner;

    state
        .rm
        .lock()
        .unwrap()
        .update_config(prev.to_owned().try_into().unwrap());
    block_on(state.rm.lock().unwrap().reload()).unwrap();

    info!("reload_config done");

    Ok(())
}

#[tauri::command]
async fn switch_proxy<R: Runtime>(
    app: tauri::AppHandle<R>,
    outbound: &str,
    selected: &str,
) -> Result<()> {
    let state = app.state::<CoreState>();
    state
        .rm
        .lock()
        .unwrap()
        .set_outbound_selected(outbound, selected);
    Ok(())
}
