//! The user's Documents and Desktop folders, for the Save file window's folder list.
//!
//! On Windows they come from the registry (`User Shell Folders`), so folders moved elsewhere
//! (to OneDrive, another drive) are found where they are; elsewhere, and as a fallback, they are
//! the home folder's `Documents` and `Desktop` (or the XDG user directories on Linux).

use std::path::PathBuf;

/// (English name, path) for each of Documents and Desktop that exists.
pub fn known_folders() -> Vec<(String, String)> {
    [("Documents", "Personal", "XDG_DOCUMENTS_DIR"), ("Desktop", "Desktop", "XDG_DESKTOP_DIR")]
        .into_iter()
        .filter_map(|(name, shell_value, xdg)| {
            let path = shell_folder(shell_value).or_else(|| xdg_folder(xdg)).or_else(|| home().map(|h| h.join(name)))?;
            path.is_dir().then(|| (name.to_string(), path.to_string_lossy().into_owned()))
        })
        .collect()
}

fn home() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")).map(PathBuf::from).filter(|p| !p.as_os_str().is_empty())
}

/// A folder from `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders`,
/// its `%VARIABLES%` expanded.
#[cfg(target_os = "windows")]
fn shell_folder(value: &str) -> Option<PathBuf> {
    let key = windows_registry::CURRENT_USER.open(r"Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders").ok()?;
    let raw = key.get_string(value).ok()?;
    let path = expand_variables(&raw)?;
    (!path.is_empty()).then(|| PathBuf::from(path))
}

#[cfg(not(target_os = "windows"))]
fn shell_folder(_value: &str) -> Option<PathBuf> {
    None
}

/// `%NAME%` replaced by the environment variable's value; `None` when one isn't set.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn expand_variables(raw: &str) -> Option<String> {
    let mut out = String::new();
    let mut rest = raw;
    while let Some(start) = rest.find('%') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let end = after.find('%')?;
        let name = &after[..end];
        if name.is_empty() {
            out.push('%');
        } else {
            out.push_str(&std::env::var(name).ok()?);
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Some(out)
}

/// An XDG user directory (`~/.config/user-dirs.dirs`, e.g. `XDG_DOCUMENTS_DIR="$HOME/Dokumente"`).
#[cfg(target_os = "linux")]
fn xdg_folder(key: &str) -> Option<PathBuf> {
    let home = home()?;
    let config = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|| home.join(".config"));
    let text = std::fs::read_to_string(config.join("user-dirs.dirs")).ok()?;
    let value = text.lines().filter_map(|line| line.trim().strip_prefix(key)?.trim_start().strip_prefix('=')).next_back()?;
    let value = value.trim().trim_matches('"');
    let path = match value.strip_prefix("$HOME") {
        Some(rest) => home.join(rest.trim_start_matches('/')),
        None => PathBuf::from(value),
    };
    // XDG sets an unused folder to the home folder itself.
    (path != home).then_some(path)
}

#[cfg(not(target_os = "linux"))]
fn xdg_folder(_key: &str) -> Option<PathBuf> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variables_in_a_registry_path_are_expanded() {
        let path = std::env::var("PATH").unwrap();
        assert_eq!(expand_variables("%PATH%\\Documents").unwrap(), format!("{path}\\Documents"));
        assert_eq!(expand_variables("D:\\Docs").unwrap(), "D:\\Docs");
        assert_eq!(expand_variables("100%% sure").unwrap(), "100% sure");
        assert!(expand_variables("%GRIDCRAFT_SURELY_UNSET_VARIABLE%\\x").is_none());
        assert!(expand_variables("%unterminated").is_none());
    }
}
