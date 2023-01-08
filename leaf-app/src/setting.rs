use std::{
    fs::File,
    io::{self, BufRead},
    path::PathBuf,
};

use leaf::config::{
    self,
    conf::{from_lines, to_internal, Config, Proxy, ProxyGroup},
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
        let mut inner = from_lines(rows)?;

        let mut g = Vec::new();
        for item in &inner.proxy {
            g.push(item.tag.clone());
        }
        for item in &inner.proxy_group {
            g.push(item.tag.clone());
        }

        inner.proxy_group.push(ProxyGroup {
            tag: "GLOBAL".into(),
            protocol: "select".into(),
            actors: Some(g),
            ..Default::default()
        });
        inner.proxy.push(Proxy {
            tag: "DIRECT".into(),
            protocol: "direct".into(),
            ..Default::default()
        });
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
