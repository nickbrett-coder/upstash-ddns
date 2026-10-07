use std::collections::HashMap;

use secret_service::{EncryptionType, SecretService};

use crate::error::Result;

const SERVICE: &str = "upstash-ddns";
const ACCOUNT: &str = "upstash";
const LABEL: &str = "Upstash DDNS";

pub async fn set_token(token: &str) -> Result<()> {
    let secret_service = SecretService::connect(EncryptionType::Dh).await?;

    let collection = secret_service.get_default_collection().await?;

    let attributes = HashMap::from([
        ("service", SERVICE),
        ("account", ACCOUNT),
    ]);

    collection
        .create_item(
            LABEL,
            attributes,
            token.as_bytes(),
            true,
            "text/plain",
        )
        .await?;

    Ok(())
}

pub async fn get_token() -> Result<String> {
    let secret_service = SecretService::connect(EncryptionType::Dh).await?;

    let attributes = HashMap::from([
        ("service", SERVICE),
        ("account", ACCOUNT),
    ]);

    let results = secret_service.search_items(attributes).await?;

    let item = if let Some(item) = results.unlocked.first() {
        item
    } else if let Some(item) = results.locked.first() {
        item.unlock().await?;
        item
    } else {
        anyhow::bail!(
            "Upstash token not found in GNOME Keyring; \
             run 'upstash-ddns credentials set'"
        );
    };

    let secret = item.get_secret().await?;

    let token = String::from_utf8(secret)
        .map_err(|_| anyhow::anyhow!("Upstash token in keyring is not valid UTF-8"))?;

    Ok(token)
}

