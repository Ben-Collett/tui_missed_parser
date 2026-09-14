#[cfg(unix)]
use std::env;
use std::fs;
use std::path::PathBuf;

use crate::parse::DEFAULT_MESSAGE;
use crate::platform;

const PROJECT_NAME: &str = "missed_chord";
const CONFIG_FILE_NAME: &str = "config.toml";

pub fn load_message() -> Result<String, String> {
    let path = resolve_config_path()?;
    if !path.exists() {
        return Ok(DEFAULT_MESSAGE.to_string());
    }
    let text =
        fs::read_to_string(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    match message_from_config(&text)? {
        Some(msg) => Ok(msg),
        None => Ok(DEFAULT_MESSAGE.to_string()),
    }
}

pub fn message_from_config(text: &str) -> Result<Option<String>, String> {
    let table: toml::Table = toml::from_str(text)
        .map_err(|e| format!("failed to parse {PROJECT_NAME} config: {e}"))?;
    match table.get("notification") {
        None => Ok(None),
        Some(section) => match section.get("message") {
            None => Ok(None),
            Some(toml::Value::String(s)) => Ok(Some(s.clone())),
            Some(_) => Err("[notification].message must be a string".to_string()),
        },
    }
}

pub fn resolve_config_path() -> Result<PathBuf, String> {
    let home = sudo_aware_home()?;
    Ok(platform::config_dir_for_home(&home)
        .join(PROJECT_NAME)
        .join(CONFIG_FILE_NAME))
}

fn sudo_aware_home() -> Result<PathBuf, String> {
    #[cfg(unix)]
    {
        if let Ok(uid) = env::var("SUDO_UID") {
            if let Some(home) = home_from_passwd(&uid) {
                return Ok(PathBuf::from(home));
            }
        }
    }
    platform::home_dir().ok_or_else(home_unset_msg)
}

#[cfg(windows)]
fn home_unset_msg() -> String {
    "cannot locate missed_chord config: USERPROFILE is unset".to_string()
}

#[cfg(not(windows))]
fn home_unset_msg() -> String {
    "cannot locate missed_chord config: HOME is unset".to_string()
}

#[cfg(unix)]
fn home_from_passwd(uid: &str) -> Option<String> {
    let text = fs::read_to_string("/etc/passwd").ok()?;
    for line in text.lines() {
        let fields: Vec<&str> = line.split(':').collect();
        if fields.get(2).copied() == Some(uid) {
            return fields.get(5).map(|s| s.to_string());
        }
    }
    None
}

#[cfg(test)]
fn expand_path(p: &str, home: &str) -> PathBuf {
    if p == "~" {
        return PathBuf::from(home);
    }
    if let Some(rest) = p.strip_prefix("~/") {
        return PathBuf::from(home).join(rest);
    }
    PathBuf::from(p)
}

#[cfg(test)]
#[path = "tests/config.rs"]
mod tests;