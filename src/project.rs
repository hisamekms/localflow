use std::path::{Path, PathBuf};

use anyhow::Result;

pub fn resolve_project_root(explicit: Option<&Path>) -> Result<PathBuf> {
    let start_dir = std::env::current_dir()?;
    resolve_from(explicit, &start_dir)
}

fn resolve_from(explicit: Option<&Path>, start_dir: &Path) -> Result<PathBuf> {
    // 1. Explicit path takes priority
    if let Some(path) = explicit {
        return Ok(path.to_path_buf());
    }

    // 2. Search upward for .localflow/
    if let Some(root) = search_upward(start_dir, ".localflow") {
        return Ok(root);
    }

    // 3. Search upward for .git
    if let Some(root) = search_upward(start_dir, ".git") {
        return Ok(root);
    }

    // 4. Fallback to current directory
    Ok(start_dir.to_path_buf())
}

fn search_upward(start: &Path, marker: &str) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join(marker).exists() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_path_takes_priority() {
        let explicit = PathBuf::from("/tmp/explicit");
        let result = resolve_from(Some(&explicit), Path::new("/tmp/other")).unwrap();
        assert_eq!(result, explicit);
    }

    #[test]
    fn detects_localflow_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let sub = tmp.path().join("a/b/c");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::create_dir_all(tmp.path().join("a/.localflow")).unwrap();

        let result = resolve_from(None, &sub).unwrap();
        assert_eq!(result, tmp.path().join("a"));
    }

    #[test]
    fn detects_git_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let sub = tmp.path().join("x/y");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::create_dir_all(tmp.path().join(".git")).unwrap();

        let result = resolve_from(None, &sub).unwrap();
        assert_eq!(result, tmp.path().to_path_buf());
    }

    #[test]
    fn localflow_takes_priority_over_git() {
        let tmp = tempfile::tempdir().unwrap();
        let sub = tmp.path().join("proj/src");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::create_dir_all(tmp.path().join(".git")).unwrap();
        std::fs::create_dir_all(tmp.path().join("proj/.localflow")).unwrap();

        let result = resolve_from(None, &sub).unwrap();
        assert_eq!(result, tmp.path().join("proj"));
    }

    #[test]
    fn fallback_to_start_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let result = resolve_from(None, tmp.path()).unwrap();
        assert_eq!(result, tmp.path().to_path_buf());
    }
}
