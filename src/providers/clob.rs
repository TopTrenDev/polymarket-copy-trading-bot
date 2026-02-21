use crate::config::TickSize;
use crate::logger;
use ethers::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyCreds {
    pub api_key: String,
    pub secret: String,
    pub passphrase: String,
}

impl ApiKeyCreds {
    fn from_json_key_names(api_key: String, secret: String, passphrase: String) -> Self {
        ApiKeyCreds {
            api_key,
            secret,
            passphrase,
        }
    }
}

#[derive(Debug, Deserialize)]
struct CredentialFile {
    key: String,
    secret: String,
    passphrase: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BalanceAllowanceRequest {
    asset_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BalanceAllowanceResponse {
    pub balance: Option<String>,
    pub allowance: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OpenOrder {
    pub side: String,
    #[serde(alias = "originalSize")]
    pub original_size: Option<String>,
    #[serde(alias = "sizeMatched")]
    pub size_matched: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateBalanceAllowanceRequest {
    asset_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderResponse {
    #[serde(alias = "orderID")]
    pub order_id: Option<String>,
    pub status: Option<String>,
    pub making_amount: Option<String>,
    pub taking_amount: Option<String>,
    #[serde(alias = "transactionsHashes")]
    pub transactions_hashes: Option<Vec<String>>,
    pub transaction_hash: Option<String>,
}

impl CreateOrderResponse {
    pub fn order_id(&self) -> Option<&str> {
        self.order_id.as_deref()
    }
    pub fn transactions_hashes(&self) -> Option<&Vec<String>> {
        self.transactions_hashes.as_ref()
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MarketOrderRequest {
    token_id: String,
    side: String,
    amount: f64,
    order_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<f64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OrderOptions {
    tick_size: String,
    neg_risk: bool,
}

pub struct ClobClient {
    client: Client,
    base_url: String,
    chain_id: u64,
    wallet: LocalWallet,
    creds: Option<ApiKeyCreds>,
}

fn credential_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("src/data/credential.json")
}

impl ClobClient {
    pub async fn new(base_url: &str, chain_id: u64, private_key: &str) -> Result<Self, String> {
        let pk = private_key
            .strip_prefix("0x")
            .unwrap_or(private_key);
        let wallet = pk
            .parse::<LocalWallet>()
            .map_err(|e| format!("Invalid private key: {}", e))?;
        let client = Client::builder()
            .build()
            .map_err(|e| e.to_string())?;
        let creds = Self::load_creds().ok();
        Ok(ClobClient {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            chain_id,
            wallet,
            creds,
        })
    }

    fn load_creds() -> Result<ApiKeyCreds, String> {
        let path = credential_path();
        let s = std::fs::read_to_string(&path).map_err(|e| format!("Credential file not found: {}", e))?;
        let c: CredentialFile = serde_json::from_str(&s).map_err(|e| format!("Invalid credential.json: {}", e))?;
        let secret = c.secret.replace('-', "+").replace('_', "/");
        Ok(ApiKeyCreds::from_json_key_names(c.key, secret, c.passphrase))
    }

    pub fn has_creds(&self) -> bool {
        self.creds.is_some()
    }

    pub async fn create_or_derive_api_key(&mut self) -> Result<ApiKeyCreds, String> {
        if let Ok(creds) = Self::load_creds() {
            self.creds = Some(creds.clone());
            return Ok(creds);
        }
        Err("Credential file not found. Run the TypeScript bot once to create src/data/credential.json, or use createCredential().".to_string())
    }

    fn l2_headers(&self, method: &str, path: &str, body: Option<&str>) -> Result<HashMap<String, String>, String> {
        let creds = self.creds.as_ref().ok_or("No API credentials. Run create_or_derive_api_key first.")?;
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let timestamp = ts.to_string();
        let address = format!("{:?}", self.wallet.address());

        let to_sign = format!("{}{}{}", timestamp, method, path);
        let to_sign = if let Some(b) = body { format!("{}{}", to_sign, b) } else { to_sign };

        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<sha2::Sha256>;
        use base64::Engine;
        let secret_decoded = base64::engine::general_purpose::STANDARD
            .decode(creds.secret.as_bytes())
            .map_err(|e| format!("Base64 decode secret: {}", e))?;
        let mut mac = HmacSha256::new_from_slice(&secret_decoded).map_err(|e| e.to_string())?;
        mac.update(to_sign.as_bytes());
        let result = mac.finalize();
        let sig = base64::engine::general_purpose::STANDARD.encode(result.into_bytes());

        let mut h = HashMap::new();
        h.insert("POLY_ADDRESS".into(), address);
        h.insert("POLY_SIGNATURE".into(), sig);
        h.insert("POLY_TIMESTAMP".into(), timestamp);
        h.insert("POLY_API_KEY".into(), creds.api_key.clone());
        h.insert("POLY_PASSPHRASE".into(), creds.passphrase.clone());
        Ok(h)
    }

    async fn ensure_creds(&mut self) -> Result<(), String> {
        if self.creds.is_none() {
            if let Ok(c) = Self::load_creds() {
                self.creds = Some(c);
            } else {
                self.create_or_derive_api_key().await?;
            }
        }
        Ok(())
    }

    pub async fn get_balance_allowance(
        &mut self,
        asset_type: &str,
        token_id: Option<&str>,
    ) -> Result<BalanceAllowanceResponse, String> {
        self.ensure_creds().await?;
        let path = "/balance-allowance";
        let url = format!("{}{}", self.base_url, path);
        let headers = self.l2_headers("GET", path, None)?;
        let mut req = self.client.get(&url);
        for (k, v) in headers {
            req = req.header(k, v);
        }
        let res = req
            .query(&[("asset_type", asset_type)])
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            let t = res.text().await.unwrap_or_default();
            return Err(format!("get_balance_allowance failed: {}", t));
        }
        let out: BalanceAllowanceResponse = res.json().await.map_err(|e| e.to_string())?;
        Ok(out)
    }

    pub async fn get_open_orders(&mut self, asset_id: Option<&str>) -> Result<Vec<OpenOrder>, String> {
        self.ensure_creds().await?;
        let path = "/orders";
        let url = format!("{}{}", self.base_url, path);
        let headers = self.l2_headers("GET", path, None)?;
        let mut req = self.client.get(&url);
        for (k, v) in headers {
            req = req.header(k, v);
        }
        let mut res = req;
        if let Some(id) = asset_id {
            res = res.query(&[("asset_id", id)]);
        }
        let res = res.send().await.map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            let t = res.text().await.unwrap_or_default();
            return Err(format!("get_open_orders failed: {}", t));
        }
        let out: Vec<OpenOrder> = res.json().await.map_err(|e| e.to_string())?;
        Ok(out)
    }

    pub async fn update_balance_allowance(&mut self, asset_type: &str) -> Result<(), String> {
        self.ensure_creds().await?;
        let path = "/balance-allowance";
        let body = UpdateBalanceAllowanceRequest {
            asset_type: asset_type.to_string(),
        };
        let body_str = serde_json::to_string(&body).map_err(|e| e.to_string())?;
        let url = format!("{}{}", self.base_url, path);
        let headers = self.l2_headers("POST", path, Some(&body_str))?;
        let mut req = self.client.post(&url).body(body_str).header("Content-Type", "application/json");
        for (k, v) in headers {
            req = req.header(k, v);
        }
        let res = req.send().await.map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            let t = res.text().await.unwrap_or_default();
            return Err(format!("update_balance_allowance failed: {}", t));
        }
        logger::success("CLOB API balance allowance updated for USDC");
        Ok(())
    }

    pub async fn create_and_post_market_order(
        &mut self,
        token_id: &str,
        side: &str,
        amount: f64,
        order_type: &str,
        tick_size: TickSize,
        neg_risk: bool,
        price: Option<f64>,
    ) -> Result<CreateOrderResponse, String> {
        self.ensure_creds().await?;
        let path = "/order";
        let order = MarketOrderRequest {
            token_id: token_id.to_string(),
            side: side.to_uppercase(),
            amount,
            order_type: order_type.to_uppercase(),
            price,
        };
        let options = OrderOptions {
            tick_size: tick_size.as_str().to_string(),
            neg_risk,
        };
        let body = serde_json::json!({
            "order": order,
            "options": options
        });
        let body_str = body.to_string();
        let url = format!("{}/order", self.base_url);
        let headers = self.l2_headers("POST", path, Some(&body_str))?;
        let mut req = self
            .client
            .post(&url)
            .body(body_str)
            .header("Content-Type", "application/json");
        for (k, v) in headers {
            req = req.header(k, v);
        }
        let res = req.send().await.map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            let t = res.text().await.unwrap_or_default();
            return Err(format!("create_and_post_market_order failed: {}", t));
        }
        let out: CreateOrderResponse = res.json().await.map_err(|e| e.to_string())?;
        Ok(out)
    }

    pub async fn get_market(&mut self, condition_id: &str) -> Result<serde_json::Value, String> {
        self.ensure_creds().await?;
        let path = format!("/markets/{}", condition_id);
        let url = format!("{}{}", self.base_url, path);
        let headers = self.l2_headers("GET", &path, None)?;
        let mut req = self.client.get(&url);
        for (k, v) in headers {
            req = req.header(k, v);
        }
        let res = req.send().await.map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            let t = res.text().await.unwrap_or_default();
            return Err(format!("get_market failed: {}", t));
        }
        let out: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
        Ok(out)
    }

    pub fn address(&self) -> Address {
        self.wallet.address()
    }
}
