use std::{
    fs::File,
    io::{self, BufRead},
    path::PathBuf,
};

use leaf::config::{
    self,
    conf::{from_lines, to_internal, Config},
};

#[derive(Debug, Default, Clone)]
pub struct UserSettings {
    pub outbound_mode: String,
    pub system_proxy: bool,
    pub inner: Config,
}

impl UserSettings {
    pub fn from_file(src: PathBuf) -> Result<Self, anyhow::Error> {
        let f = File::open(src)?;
        let rows = io::BufReader::new(f).lines().collect();
        let inner = from_lines(rows)?;
        Ok(Self {
            inner,
            outbound_mode: "MATCH".into(),
            system_proxy: false,
        })
    }
}

impl TryFrom<UserSettings> for config::Config {
    type Error = anyhow::Error;

    fn try_from(mut settings: UserSettings) -> Result<Self, Self::Error> {
        let inner = to_internal(&mut settings.inner)?;
        Ok(inner)
    }
}
