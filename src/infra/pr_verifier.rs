use anyhow::{bail, Context, Result};

use crate::application::port::PrVerifier;

/// PR verifier that uses the `gh` CLI to check PR status.
pub struct GhCliPrVerifier;

impl PrVerifier for GhCliPrVerifier {
    fn verify_pr_status(&self, pr_url: &str, auto_merge: bool) -> Result<()> {
        let mut args = vec!["pr", "view", pr_url, "--json", "state"];
        if !auto_merge {
            args[4] = "state,reviewDecision";
        }

        let output = std::process::Command::new("gh")
            .args(&args)
            .output()
            .context(
                "failed to run 'gh' CLI. gh is required when merge_via = \"pr\". \
                 Install it from https://cli.github.com/",
            )?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("gh pr view failed: {}", stderr.trim());
        }

        let json: serde_json::Value =
            serde_json::from_slice(&output.stdout).context("failed to parse gh output")?;

        let state = json["state"].as_str().unwrap_or("");
        if state != "MERGED" {
            bail!(
                "cannot complete task: PR is not merged (current state: {}). \
                 Merge the PR first, then run complete again.",
                state
            );
        }

        if !auto_merge {
            let decision = json["reviewDecision"].as_str().unwrap_or("");
            if decision != "APPROVED" {
                bail!(
                    "cannot complete task: PR has not been approved (reviewDecision: {}). \
                     Get the PR reviewed and approved, then run complete again.",
                    if decision.is_empty() {
                        "none"
                    } else {
                        decision
                    }
                );
            }
        }

        Ok(())
    }
}
