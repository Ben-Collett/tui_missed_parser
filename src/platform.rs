use std::path::{Path, PathBuf};

pub fn home_dir() -> Option<PathBuf> {
    Some(PathBuf::from(candidate_home_var()?))
}

// Windows: prefer $HOME (set by Git-for-Windows etc.), fall back to $USERPROFILE.
#[cfg(windows)]
fn candidate_home_var() -> Option<String> {
    std::env::var("HOME")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| {
            std::env::var("USERPROFILE")
                .ok()
                .filter(|v| !v.trim().is_empty())
        })
}

#[cfg(not(windows))]
fn candidate_home_var() -> Option<String> {
    std::env::var("HOME").ok().filter(|v| !v.trim().is_empty())
}

pub fn expand_home(p: &str) -> PathBuf {
    let home = home_dir();
    if let Some(home) = &home {
        if p == "~" {
            return home.clone();
        }
        if let Some(rest) = p.strip_prefix("~/") {
            return home.join(rest);
        }
    }
    PathBuf::from(p)
}

pub fn config_dir_for_home(home: &Path) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| home.to_path_buf())
    }
    #[cfg(target_os = "macos")]
    {
        home.join("Library").join("Application Support")
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        if let Ok(v) = std::env::var("XDG_CONFIG_HOME") {
            if !v.trim().is_empty() {
                return expand_home(&v);
            }
        }
        home.join(".config")
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn expand_home_passes_absolute_and_relative_paths_through() {
        assert_eq!(expand_home("/abs/path"), PathBuf::from("/abs/path"));
        assert_eq!(expand_home("rel/path"), PathBuf::from("rel/path"));
    }

    #[cfg(unix)]
    #[test]
    fn expand_home_resolves_tilde_from_home_env() {
        env::set_var("HOME", "/home/test");
        assert_eq!(expand_home("~"), PathBuf::from("/home/test"));
        assert_eq!(expand_home("~/logs"), PathBuf::from("/home/test/logs"));
        assert_eq!(expand_home("~$x"), PathBuf::from("~$x"));
        assert_eq!(expand_home("/etc/passwd"), PathBuf::from("/etc/passwd"));
    }

    #[cfg(unix)]
    #[test]
    fn home_dir_reads_home_env() {
        env::set_var("HOME", "/home/test");
        assert_eq!(home_dir(), Some(PathBuf::from("/home/test")));
    }

    #[cfg(any(target_os = "linux", target_os = "android"))]
    #[test]
    fn config_dir_for_home_honors_xdg_then_home() {
        env::remove_var("XDG_CONFIG_HOME");
        assert_eq!(
            config_dir_for_home(&PathBuf::from("/home/u")),
            PathBuf::from("/home/u/.config")
        );
        env::set_var("XDG_CONFIG_HOME", "/xdg/cfg");
        assert_eq!(
            config_dir_for_home(&PathBuf::from("/home/u")),
            PathBuf::from("/xdg/cfg")
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn config_dir_for_home_is_application_support() {
        assert_eq!(
            config_dir_for_home(&PathBuf::from("/home/u")),
            PathBuf::from("/home/u/Library/Application Support")
        );
    }

    #[cfg(windows)]
    #[test]
    fn config_dir_for_home_honors_appdata() {
        env::remove_var("APPDATA");
        assert_eq!(
            config_dir_for_home(&PathBuf::from("C:\\Users\\u")),
            PathBuf::from("C:\\Users\\u")
        );
        env::set_var("APPDATA", "C:\\Users\\u\\AppData\\Roaming");
        assert_eq!(
            config_dir_for_home(&PathBuf::from("C:\\Users\\u")),
            PathBuf::from("C:\\Users\\u\\AppData\\Roaming")
        );
    }
}