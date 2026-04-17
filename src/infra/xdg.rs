//! Centralized XDG Base Directory resolution.
//!
//! This is the *only* module that reads `XDG_*` / `HOME` environment variables.
//! Callers obtain an [`XdgDirs`] instance at startup (typically via
//! [`XdgDirs::from_env`]) and then pass it by reference through the call chain.
//! Tests may construct [`XdgDirs`] directly with `tempdir` paths to avoid
//! mutating process-global env vars.

use std::path::{Path, PathBuf};

/// Resolved XDG Base Directory paths (or `HOME`-based fallbacks).
///
/// Each field is `Option<PathBuf>` because, for a given XDG variable, both the
/// explicit env var and `HOME` may be absent. Callers that currently return
/// `Option<PathBuf>` in that situation preserve the same behavior.
#[derive(Debug, Clone, Default)]
pub struct XdgDirs {
    pub cache_home: Option<PathBuf>,
    pub config_home: Option<PathBuf>,
    pub data_home: Option<PathBuf>,
    pub state_home: Option<PathBuf>,
}

impl XdgDirs {
    /// Resolve all four XDG directories from the current process environment.
    ///
    /// For each entry, the priority is:
    /// 1. `$XDG_<NAME>_HOME` if set and absolute.
    /// 2. `$HOME/<fallback>` if `HOME` is set.
    /// 3. `None` otherwise.
    pub fn from_env() -> Self {
        let home = std::env::var("HOME").ok().map(PathBuf::from);
        Self {
            cache_home: resolve("XDG_CACHE_HOME", home.as_deref(), ".cache"),
            config_home: resolve("XDG_CONFIG_HOME", home.as_deref(), ".config"),
            data_home: resolve("XDG_DATA_HOME", home.as_deref(), ".local/share"),
            state_home: resolve("XDG_STATE_HOME", home.as_deref(), ".local/state"),
        }
    }
}

fn resolve(env_name: &str, home: Option<&Path>, home_fallback: &str) -> Option<PathBuf> {
    std::env::var(env_name)
        .map(PathBuf::from)
        .ok()
        .filter(|p| p.is_absolute())
        .or_else(|| home.map(|h| h.join(home_fallback)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_prefers_absolute_env_var() {
        // Use a synthetic env var name to avoid touching shared XDG state.
        // SAFETY: tests are single-threaded or serialized per test-thread config,
        // and this name is unique to this test.
        unsafe {
            std::env::set_var("SENKO_TEST_XDG_ABSOLUTE", "/abs/path");
        }
        let got = resolve(
            "SENKO_TEST_XDG_ABSOLUTE",
            Some(Path::new("/home/user")),
            ".cache",
        );
        assert_eq!(got, Some(PathBuf::from("/abs/path")));
        unsafe {
            std::env::remove_var("SENKO_TEST_XDG_ABSOLUTE");
        }
    }

    #[test]
    fn resolve_falls_back_to_home_when_env_relative() {
        unsafe {
            std::env::set_var("SENKO_TEST_XDG_RELATIVE", "rel/path");
        }
        let got = resolve(
            "SENKO_TEST_XDG_RELATIVE",
            Some(Path::new("/home/user")),
            ".cache",
        );
        assert_eq!(got, Some(PathBuf::from("/home/user/.cache")));
        unsafe {
            std::env::remove_var("SENKO_TEST_XDG_RELATIVE");
        }
    }

    #[test]
    fn resolve_returns_none_when_no_env_and_no_home() {
        let got = resolve("SENKO_TEST_XDG_UNSET_ABSENT", None, ".cache");
        assert_eq!(got, None);
    }
}
