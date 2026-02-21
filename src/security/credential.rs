use crate::logger;
use crate::providers::ClobClient;
use std::path::PathBuf;

fn credential_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("src/data/credential.json")
}

pub async fn create_credential() -> Result<Option<crate::providers::clob::ApiKeyCreds>, String> {
    let private_key = std::env::var("PRIVATE_KEY").map_err(|_| "PRIVATE_KEY not found".to_string())?;
    let chain_id = std::env::var("CHAIN_ID")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(137u64);
    let host = std::env::var("CLOB_API_URL").unwrap_or_else(|_| "https://clob.polymarket.com".into());

    let mut client = ClobClient::new(&host, chain_id, &private_key).await?;
    match client.create_or_derive_api_key().await {
        Ok(creds) => Ok(Some(creds)),
        Err(e) => {
            logger::info(&format!("Credential: {}", e));
            Ok(None)
        }
    }
}
