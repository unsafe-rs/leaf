use std::fs;
use std::{error::Error, fs::create_dir_all, path::Path, sync::Mutex};

use leaf::config::conf::{to_internal, Proxy, ProxyGroup, Rule};
use leaf::config::json::{self, Log};
use leaf::config::{self, conf::Config};
use leaf::relay;
use tauri::{api::path::home_dir, App, Manager};

use crate::states::UserSetting;

pub(crate) fn setup(app: &mut App) -> Result<(), Box<dyn Error>> {
    let dst = Path::new(&home_dir().unwrap())
        .join(".config")
        .join("leafapp");
    if !dst.is_dir() {
        create_dir_all(dst.clone())?;
    }

    if let Some(src) = app.path_resolver().resolve_resource("resources/geo.mmdb") {
        let dst = dst.join("geo.mmdb");
        if !dst.exists() {
            fs::copy(src, dst)?;
        }
    }

    std::env::set_var("ASSET_LOCATION", dst);

    let defaults = app
        .path_resolver()
        .resolve_resource("resources/default.json")
        .unwrap_or_default();
    let mut rmc = json::json_from_string(&fs::read_to_string(defaults)?)?;
    let rm = relay::create(json::to_internal(&mut rmc)?)?;

    app.manage(UserSetting {
        mode: Default::default(),
        tun: Default::default(),
        rm,
        rmc: Mutex::new(rmc),
        system_proxy: Default::default(),
    });
    Ok(())
}
