use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::infra::xdg::XdgDirs;

fn cache_path(xdg: &XdgDirs) -> Option<PathBuf> {
    xdg.cache_home
        .as_ref()
        .map(|dir| dir.join("senko").join("auth_mode.json"))
}

fn read_cache(xdg: &XdgDirs) -> HashMap<String, String> {
    let path = match cache_path(xdg) {
        Some(p) => p,
        None => return HashMap::new(),
    };
    let data = match std::fs::read_to_string(&path) {
        Ok(d) => d,
        Err(_) => return HashMap::new(),
    };
    serde_json::from_str(&data).unwrap_or_default()
}

fn write_cache(xdg: &XdgDirs, map: &HashMap<String, String>) -> Result<()> {
    let path = cache_path(xdg).context("could not determine XDG cache path")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create cache directory: {}", parent.display()))?;
    }
    let data = serde_json::to_string_pretty(map).context("failed to serialize auth_mode cache")?;
    std::fs::write(&path, data)
        .with_context(|| format!("failed to write auth_mode cache: {}", path.display()))?;
    Ok(())
}

pub fn cache_auth_mode(xdg: &XdgDirs, api_url: &str, auth_mode: &str) -> Result<()> {
    let mut map = read_cache(xdg);
    map.insert(api_url.to_string(), auth_mode.to_string());
    write_cache(xdg, &map)
}

pub fn get_cached_auth_mode(xdg: &XdgDirs, api_url: &str) -> Option<String> {
    read_cache(xdg).get(api_url).cloned()
}

pub fn delete_cached_auth_mode(xdg: &XdgDirs, api_url: &str) -> Result<()> {
    let mut map = read_cache(xdg);
    map.remove(api_url);
    write_cache(xdg, &map)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn xdg_with_cache(path: &std::path::Path) -> XdgDirs {
        XdgDirs {
            cache_home: Some(path.to_path_buf()),
            ..Default::default()
        }
    }

    #[test]
    fn test_cache_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let xdg = xdg_with_cache(dir.path());
        cache_auth_mode(&xdg, "http://localhost:8080", "trusted_headers").unwrap();
        assert_eq!(
            get_cached_auth_mode(&xdg, "http://localhost:8080"),
            Some("trusted_headers".to_string())
        );
    }

    #[test]
    fn test_get_missing_key() {
        let dir = tempfile::tempdir().unwrap();
        let xdg = xdg_with_cache(dir.path());
        assert_eq!(get_cached_auth_mode(&xdg, "http://nonexistent:9999"), None);
    }

    #[test]
    fn test_delete_cleans_up() {
        let dir = tempfile::tempdir().unwrap();
        let xdg = xdg_with_cache(dir.path());
        cache_auth_mode(&xdg, "http://localhost:8080", "oidc").unwrap();
        delete_cached_auth_mode(&xdg, "http://localhost:8080").unwrap();
        assert_eq!(get_cached_auth_mode(&xdg, "http://localhost:8080"), None);
    }

    #[test]
    fn test_multiple_urls() {
        let dir = tempfile::tempdir().unwrap();
        let xdg = xdg_with_cache(dir.path());
        cache_auth_mode(&xdg, "http://a:1000", "trusted_headers").unwrap();
        cache_auth_mode(&xdg, "http://b:2000", "oidc").unwrap();
        assert_eq!(
            get_cached_auth_mode(&xdg, "http://a:1000"),
            Some("trusted_headers".to_string())
        );
        assert_eq!(
            get_cached_auth_mode(&xdg, "http://b:2000"),
            Some("oidc".to_string())
        );
    }
}
