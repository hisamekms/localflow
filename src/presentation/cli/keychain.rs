use anyhow::{Context, Result};

const SERVICE: &str = "senko";

pub fn save(api_url: &str, api_key: &str) -> Result<()> {
    let entry =
        keyring::Entry::new(SERVICE, api_url).context("failed to create keychain entry")?;
    entry
        .set_password(api_key)
        .context("failed to save API key to keychain")?;
    Ok(())
}

pub fn load(api_url: &str) -> Result<String> {
    let entry =
        keyring::Entry::new(SERVICE, api_url).context("failed to create keychain entry")?;
    entry
        .get_password()
        .context("failed to load API key from keychain")
}

pub fn delete(api_url: &str) -> Result<()> {
    let entry =
        keyring::Entry::new(SERVICE, api_url).context("failed to create keychain entry")?;
    entry
        .delete_credential()
        .context("failed to delete API key from keychain")?;
    Ok(())
}
