//! `~/.config/kite/directory/credentials.toml` reader/writer.

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CredentialsFile {
    #[serde(default = "default_profile_name")]
    pub default_profile: String,
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
}

fn default_profile_name() -> String {
    "default".into()
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub session_token: Option<String>,
    #[serde(default)]
    pub agent_token: Option<String>,
    #[serde(default)]
    pub stepup_token: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
}

pub fn credentials_path() -> PathBuf {
    if let Ok(p) = std::env::var("KITEDIR_CREDENTIALS") {
        return PathBuf::from(p);
    }
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".config")
        .join("kite")
        .join("directory")
        .join("credentials.toml")
}

pub fn load() -> anyhow::Result<CredentialsFile> {
    let p = credentials_path();
    if !p.exists() {
        return Ok(CredentialsFile {
            default_profile: "default".into(),
            profiles: BTreeMap::new(),
        });
    }
    let body = std::fs::read_to_string(&p).with_context(|| format!("read {}", p.display()))?;
    toml::from_str(&body).with_context(|| format!("parse {}", p.display()))
}

pub fn save(file: &CredentialsFile) -> anyhow::Result<()> {
    let p = credentials_path();
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let body = toml::to_string_pretty(file)?;
    std::fs::write(&p, body)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = std::fs::metadata(&p)?.permissions();
        perm.set_mode(0o600);
        std::fs::set_permissions(&p, perm)?;
    }
    Ok(())
}

pub fn get_profile<'a>(file: &'a CredentialsFile, name: &str) -> Option<&'a Profile> {
    file.profiles.get(name)
}

pub fn set_profile(file: &mut CredentialsFile, name: &str, profile: Profile) {
    file.profiles.insert(name.to_string(), profile);
}
