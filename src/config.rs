use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub data_dir: String,
    pub http_port: u16,
    pub api_token: Option<String>,
    pub log_level: String,
    pub go2rtc_url: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_dir: "./data".into(),
            http_port: 8080,
            api_token: None,
            log_level: "info".into(),
            go2rtc_url: None,
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let candidates = ["irondom.toml", "/etc/irondom.toml", "/root/irondom/irondom.toml", "/overlay/upper/etc/irondom.toml"];
        for p in candidates {
            let path = std::path::Path::new(p);
            if path.exists() {
                let s = std::fs::read_to_string(path)?;
                let c: Config = toml::from_str(&s)?;
                return Ok(c);
            }
        }
        Ok(Config::default())
    }

    pub fn data_path(&self) -> PathBuf {
        PathBuf::from(&self.data_dir)
    }
}
