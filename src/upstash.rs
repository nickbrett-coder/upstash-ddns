use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RedisResponse {
    result: Option<String>,
    error: Option<String>,
}

pub struct Upstash {
    client: Client,
    url: String,
    token: String,
}

impl Upstash {
    pub fn new(url: String, token: String) -> Self {
        Self {
            client: Client::new(),
            url: url.trim_end_matches('/').to_owned(),
            token,
        }
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let encoded_key = urlencoding::encode(key);

        let url = format!("{}/get/{}", self.url, encoded_key);

        let response = self
            .client
            .get(url)
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?;

        let redis: RedisResponse = response.json().await?;

        if let Some(error) = redis.error {
            anyhow::bail!("Upstash error: {error}");
        }

        Ok(redis.result)
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        let encoded_key = urlencoding::encode(key);

        let url = format!("{}/set/{}", self.url, encoded_key);

        let response = self
            .client
            .post(url)
            .bearer_auth(&self.token)
            .body(value.to_owned())
            .send()
            .await?
            .error_for_status()?;

        let redis: RedisResponse = response.json().await?;

        if let Some(error) = redis.error {
            anyhow::bail!("Upstash error: {error}");
        }

        Ok(())
    }
}

