use std::sync::{Arc, Mutex};

use leaf::{app::router, relay::RelayManager};
use tauri::{Manager, Result, Runtime};

use crate::{setting, system};

pub struct CoreState {
    pub rm: Arc<RelayManager>,
    pub settings: Arc<Mutex<setting::UserSettings>>,
}

#[tauri::command]
pub async fn switch_mode_proxy<R: Runtime>(
    app: tauri::AppHandle<R>,
    mode: router::Mode,
) -> Result<()> {
    let state = app.state::<CoreState>();
    state.settings.lock().unwrap().outbound_mode = mode.to_string();
    state.rm.route_manager().write().await.set_mode(mode);

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
async fn switch_proxy<R: Runtime>(
    app: tauri::AppHandle<R>,
    outbound: &str,
    selected: &str,
) -> Result<()> {
    let state = app.state::<CoreState>();
    state.rm.set_outbound_selected(outbound, selected);
    Ok(())
}
