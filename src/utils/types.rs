use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradePayload {
    pub asset: String,
    #[serde(alias = "conditionId")]
    pub condition_id: String,
    pub event_slug: Option<String>,
    pub outcome: String,
    pub outcome_index: Option<u32>,
    pub price: f64,
    pub proxy_wallet: Option<String>,
    pub wallet: Option<String>,
    pub user: Option<String>,
    pub address: Option<String>,
    pub user_address: Option<String>,
    pub pseudonym: Option<String>,
    pub side: String,
    pub size: f64,
    pub slug: Option<String>,
    pub timestamp: u64,
    pub title: Option<String>,
    pub transaction_hash: Option<String>,
}

impl TradePayload {
    pub fn proxy_wallet_lower(&self) -> Option<String> {
        self.proxy_wallet.as_ref().map(|s| s.to_lowercase())
    }
}
