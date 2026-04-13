use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};

fn cache_path() -> Option<PathBuf> {
    let cache_dir = std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .ok()
        .filter(|p| p.is_absolute())
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".cache"))
        })?;
    Some(cache_dir.join("senko").join("auth_mode.json"))
}

fn read_cache() -> HashMap<String, String> {
    let path = match cache_path() {
        Some(p) => p,
        None => return HashMap::new(),
    };
    let data = match std::fs::read_to_string(&path) {
        Ok(d) => d,
        Err(_) => return HashMap::new(),
    };
    serde_json::from_str(&data).unwrap_or_default()
}

fn write_cache(map: &HashMap<String, String>) -> Result<()> {
    let path = cache_path().context("could not determine XDG cache path")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create cache directory: {}", parent.display()))?;
    }
    let data = serde_json::to_string_pretty(map).context("failed to serialize auth_mode cache")?;
    std::fs::write(&path, data)
        .with_context(|| format!("failed to write auth_mode cache: {}", path.display()))?;
    Ok(())
}

pub fn cache_auth_mode(api_url: &str, auth_mode: &str) -> Result<()> {
    let mut map = read_cache();
    map.insert(api_url.to_string(), auth_mode.to_string());
    write_cache(&map)
}

pub fn get_cached_auth_mode(api_url: &str) -> Option<String> {
    read_cache().get(api_url).cloned()
}

pub fn delete_cached_auth_mode(api_url: &str) -> Result<()> {
    let mut map = read_cache();
    map.remove(api_url);
    write_cache(&map)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_temp_cache<F: FnOnce()>(f: F) {
        let dir = tempfile::tempdir().unwrap();
        let prev = std::env::var("XDG_CACHE_HOME").ok();
        // SAFETY: tests are run serially (single-threaded test runner or #[serial])
        unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };
        f();
        match prev {
            Some(v) => unsafe { std::env::set_var("XDG_CACHE_HOME", v) },
            None => unsafe { std::env::remove_var("XDG_CACHE_HOME") },
        }
    }

    #[test]
    fn test_cache_roundtrip() {
        with_temp_cache(|| {
            cache_auth_mode("http://localhost:8080", "trusted_headers").unwrap();
            assert_eq!(
                get_cached_auth_mode("http://localhost:8080"),
                Some("trusted_headers".to_string())
            );
        });
    }

    #[test]
    fn test_get_missing_key() {
        with_temp_cache(|| {
            assert_eq!(get_cached_auth_mode("http://nonexistent:9999"), None);
        });
    }

    #[test]
    fn test_delete_cleans_up() {
        with_temp_cache(|| {
            cache_auth_mode("http://localhost:8080", "oidc").unwrap();
            delete_cached_auth_mode("http://localhost:8080").unwrap();
            assert_eq!(get_cached_auth_mode("http://localhost:8080"), None);
        });
    }

    #[test]
    fn test_multiple_urls() {
        with_temp_cache(|| {
            cache_auth_mode("http://a:1000", "trusted_headers").unwrap();
            cache_auth_mode("http://b:2000", "oidc").unwrap();
            assert_eq!(
                get_cached_auth_mode("http://a:1000"),
                Some("trusted_headers".to_string())
            );
            assert_eq!(
                get_cached_auth_mode("http://b:2000"),
                Some("oidc".to_string())
            );
        });
    }
}
