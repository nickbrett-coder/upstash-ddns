//~/.config/upstash-ddns/config.toml
use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::error::Result;

const QUALIFIER: &str = "";
const ORGANIZATION: &str = "";
const APPLICATION: &str = "upstash-ddns";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Upstash Redis REST endpoint.
    pub url: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path()?;

        let contents = fs::read_to_string(&path).map_err(|e| {
            anyhow::anyhow!(
                "could not read config file {}: {}",
                path.display(),
                e
            )
        })?;

        let config: Self = toml::from_str(&contents)
            .map_err(|e| anyhow::anyhow!("invalid config file: {e}"))?;

        validate(&config)?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        validate(self)?;

        let path = config_path()?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;

        fs::write(&path, contents)?;

        Ok(())
    }
}

fn validate(config: &Config) -> Result<()> {
    let url = reqwest::Url::parse(&config.url)
        .map_err(|e| anyhow::anyhow!("invalid Upstash URL: {e}"))?;

    if url.scheme() != "https" {
        anyhow::bail!("Upstash URL must use HTTPS");
    }

    Ok(())
}

pub fn config_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .ok_or_else(|| anyhow::anyhow!("could not determine configuration directory"))?;

    Ok(dirs.config_dir().join("config.toml"))
}

