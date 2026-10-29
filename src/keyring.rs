use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::env;

use crate::error::Result;

const CREDENTIAL_NAME: &str = "upstash-token";

fn credentials_dir() -> Result<PathBuf> {
    if let Some(path) = env::var_os("CREDENTIALS_DIRECTORY") {
        return Ok(PathBuf::from(path));
    }

    if let Some(project_dirs) = directories::ProjectDirs::from("io", "upstash", "upstash-ddns") {
        let dir = project_dirs.config_local_dir().join("credentials");
        return Ok(dir.to_path_buf());
    }

    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/var/lib/upstash-ddns"));

    Ok(home.join(".config").join("upstash-ddns").join("credentials"))
}

fn token_path() -> Result<PathBuf> {
    Ok(credentials_dir()?.join(CREDENTIAL_NAME))
}

pub async fn set_token(token: &str) -> Result<()> {
    let token = token.trim();
    if token.is_empty() {
        anyhow::bail!("token cannot be empty");
    }

    let dir = credentials_dir()?;
    fs::create_dir_all(&dir)?;

    let path = dir.join(CREDENTIAL_NAME);
    fs::write(&path, token.as_bytes())?;

    let mut permissions = fs::metadata(&path)?.permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(&path, permissions)?;

    Ok(())
}

pub async fn get_token() -> Result<String> {
    if let Ok(token) = env::var("UPSTASH_DDNS_TOKEN") {
        return Ok(token);
    }

    if let Ok(token) = env::var("UPSTASH_TOKEN") {
        return Ok(token);
    }

    if let Some(path) = env::var_os("CREDENTIALS_DIRECTORY") {
        let token = fs::read_to_string(PathBuf::from(path).join(CREDENTIAL_NAME))?;
        return Ok(token.trim().to_owned());
    }

    let path = token_path()?;
    let token = fs::read_to_string(&path)
        .map_err(|err| {
            anyhow::anyhow!(
                "Upstash token not found in systemd credentials; run 'upstash-ddns credentials set' ({err})"
            )
        })?;

    Ok(token.trim().to_owned())
}

