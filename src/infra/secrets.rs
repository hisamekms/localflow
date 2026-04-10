use std::collections::HashMap;

use anyhow::{Context, Result};
use aws_sdk_secretsmanager::Client;
use tokio::sync::{OnceCell, RwLock};

pub struct SecretsManagerClient {
    region: Option<String>,
    client: OnceCell<Client>,
    cache: RwLock<HashMap<String, String>>,
}

impl SecretsManagerClient {
    pub fn new(region: Option<String>) -> Self {
        Self {
            region,
            client: OnceCell::new(),
            cache: RwLock::new(HashMap::new()),
        }
    }

    async fn client(&self) -> Result<&Client> {
        self.client
            .get_or_try_init(|| async {
                let mut config_loader =
                    aws_config::defaults(aws_config::BehaviorVersion::latest());
                if let Some(ref region) = self.region {
                    config_loader =
                        config_loader.region(aws_config::Region::new(region.clone()));
                }
                let sdk_config = config_loader.load().await;
                Ok(Client::new(&sdk_config))
            })
            .await
    }

    pub async fn get_secret(&self, arn: &str) -> Result<String> {
        // Check cache first (read lock)
        {
            let cache = self.cache.read().await;
            if let Some(value) = cache.get(arn) {
                return Ok(value.clone());
            }
        }

        // Cache miss — fetch from AWS
        let client = self.client().await?;
        let response = client
            .get_secret_value()
            .secret_id(arn)
            .send()
            .await
            .context("failed to get secret value from Secrets Manager")?;

        let secret_string = response
            .secret_string()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "secret {arn} has no string value (binary secrets are not supported)"
                )
            })?
            .to_string();

        // Store in cache (write lock)
        {
            let mut cache = self.cache.write().await;
            cache.insert(arn.to_string(), secret_string.clone());
        }

        Ok(secret_string)
    }
}
