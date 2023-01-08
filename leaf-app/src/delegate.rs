use std::fs;
use std::sync::Arc;
use std::{error::Error, fs::create_dir_all, path::Path, sync::Mutex};

use leaf::relay;
use tauri::{api::path::home_dir, App, Manager};

use crate::states::CoreState;
use crate::{menu, setting};

pub(crate) fn setup(app: &mut App) -> Result<(), Box<dyn Error>> {
    let dst = Path::new(&home_dir().unwrap())
        .join(".config")
        .join("leaf-bud");
    if !dst.is_dir() {
        create_dir_all(dst.clone())?;
    }

    if let Some(src) = app.path_resolver().resolve_resource("resources/geo.mmdb") {
        let dst = dst.join("geo.mmdb");
        if !dst.exists() {
            fs::copy(src, dst)?;
        }
    }

    if let Some(src) = app
        .path_resolver()
        .resolve_resource("resources/default.conf")
    {
        let dst = dst.join("bud.conf");
        if !dst.exists() {
            fs::copy(src, dst)?;
        }
    }

    std::env::set_var("ASSET_LOCATION", dst.clone());

    let settings = setting::UserSettings::from_file(dst.join("bud.conf"))?;

    let rm = relay::create(settings.clone().try_into()?)?;
    app.tray_handle().set_menu(menu::build(settings.clone()))?;

    app.manage(CoreState {
        rm: Arc::new(Mutex::new(rm)),
        settings: Arc::new(Mutex::new(settings)),
    });
    Ok(())
}
