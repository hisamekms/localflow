use std::collections::HashMap;

use anyhow::{Context, Result};
use async_trait::async_trait;
use aws_sdk_secretsmanager::Client;
use tokio::sync::{OnceCell, RwLock};

#[async_trait]
pub(crate) trait SecretFetcher: Send + Sync {
    async fn fetch_secret(&self, arn: &str) -> Result<String>;
}

struct AwsSecretFetcher {
    region: Option<String>,
    client: OnceCell<Client>,
}

impl AwsSecretFetcher {
    fn new(region: Option<String>) -> Self {
        Self {
            region,
            client: OnceCell::new(),
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
}

#[async_trait]
impl SecretFetcher for AwsSecretFetcher {
    async fn fetch_secret(&self, arn: &str) -> Result<String> {
        let client = self.client().await?;
        let response = client
            .get_secret_value()
            .secret_id(arn)
            .send()
            .await
            .context("failed to get secret value from Secrets Manager")?;

        response
            .secret_string()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "secret {arn} has no string value (binary secrets are not supported)"
                )
            })
            .map(|s| s.to_string())
    }
}

pub struct SecretsManagerClient {
    fetcher: Box<dyn SecretFetcher>,
    cache: RwLock<HashMap<String, String>>,
}

impl SecretsManagerClient {
    pub fn new(region: Option<String>) -> Self {
        Self {
            fetcher: Box::new(AwsSecretFetcher::new(region)),
            cache: RwLock::new(HashMap::new()),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_fetcher(fetcher: Box<dyn SecretFetcher>) -> Self {
        Self {
            fetcher,
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get_secret(&self, arn: &str) -> Result<String> {
        // Check cache first (read lock)
        {
            let cache = self.cache.read().await;
            if let Some(value) = cache.get(arn) {
                return Ok(value.clone());
            }
        }

        // Cache miss — fetch via fetcher
        let secret_string = self.fetcher.fetch_secret(arn).await?;

        // Store in cache (write lock)
        {
            let mut cache = self.cache.write().await;
            cache.insert(arn.to_string(), secret_string.clone());
        }

        Ok(secret_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct FakeSecretFetcher {
        secrets: Arc<RwLock<HashMap<String, String>>>,
        call_count: Arc<AtomicUsize>,
    }

    impl FakeSecretFetcher {
        fn new(secrets: HashMap<String, String>) -> (Self, Arc<AtomicUsize>) {
            let call_count = Arc::new(AtomicUsize::new(0));
            let fetcher = Self {
                secrets: Arc::new(RwLock::new(secrets)),
                call_count: call_count.clone(),
            };
            (fetcher, call_count)
        }

        fn new_with_shared_secrets(
            secrets: Arc<RwLock<HashMap<String, String>>>,
        ) -> (Self, Arc<AtomicUsize>) {
            let call_count = Arc::new(AtomicUsize::new(0));
            let fetcher = Self {
                secrets,
                call_count: call_count.clone(),
            };
            (fetcher, call_count)
        }
    }

    #[async_trait]
    impl SecretFetcher for FakeSecretFetcher {
        async fn fetch_secret(&self, arn: &str) -> Result<String> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            let secrets = self.secrets.read().await;
            secrets
                .get(arn)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("secret not found: {arn}"))
        }
    }

    #[tokio::test]
    async fn cache_hit_fetcher_called_once() {
        let (fetcher, call_count) = FakeSecretFetcher::new(HashMap::from([
            ("arn:a".to_string(), "value-a".to_string()),
        ]));
        let client = SecretsManagerClient::with_fetcher(Box::new(fetcher));

        let v1 = client.get_secret("arn:a").await.unwrap();
        let v2 = client.get_secret("arn:a").await.unwrap();

        assert_eq!(v1, "value-a");
        assert_eq!(v2, "value-a");
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn cache_independent_keys() {
        let (fetcher, call_count) = FakeSecretFetcher::new(HashMap::from([
            ("arn:a".to_string(), "value-a".to_string()),
            ("arn:b".to_string(), "value-b".to_string()),
        ]));
        let client = SecretsManagerClient::with_fetcher(Box::new(fetcher));

        let va = client.get_secret("arn:a").await.unwrap();
        let vb = client.get_secret("arn:b").await.unwrap();

        assert_eq!(va, "value-a");
        assert_eq!(vb, "value-b");
        assert_eq!(call_count.load(Ordering::SeqCst), 2);

        // Fetching again uses cache
        let va2 = client.get_secret("arn:a").await.unwrap();
        assert_eq!(va2, "value-a");
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn cache_no_store_on_error() {
        let secrets = Arc::new(RwLock::new(HashMap::new()));
        let (fetcher, call_count) =
            FakeSecretFetcher::new_with_shared_secrets(secrets.clone());
        let client = SecretsManagerClient::with_fetcher(Box::new(fetcher));

        // First attempt: secret not found -> error
        assert!(client.get_secret("arn:a").await.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // Add the secret to the fetcher
        {
            let mut s = secrets.write().await;
            s.insert("arn:a".to_string(), "value-a".to_string());
        }

        // Second attempt: fetcher called again (error was not cached)
        let val = client.get_secret("arn:a").await.unwrap();
        assert_eq!(val, "value-a");
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }
}
