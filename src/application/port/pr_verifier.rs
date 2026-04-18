use anyhow::Result;

/// Port trait for verifying PR status before task completion.
/// The implementation decides how to check (e.g., gh CLI, HTTP API, no-op).
pub trait PrVerifier: Send + Sync {
    fn verify_pr_status(&self, pr_url: &str) -> Result<()>;
}
