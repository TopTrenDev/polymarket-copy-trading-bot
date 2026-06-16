use crate::config::TickSize;
use crate::logger;
use ethers::prelude::*;
use ethers::types::transaction::eip712::Eip712;
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
    #[serde(default)]
    pub allowances: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct OpenOrder {
    pub id: Option<String>,
    pub status: Option<String>,
    pub market: Option<String>,
    #[serde(alias = "asset_id")]
    pub asset_id: Option<String>,
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

#[derive(Debug, Deserialize)]
struct LastTradePriceResponse {
    price: String,
    side: String,
}

fn now_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn now_unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

fn to_fixed_1e6(value: f64) -> Result<u128, String> {
    if !value.is_finite() || value < 0.0 {
        return Err(format!("Invalid numeric value: {}", value));
    }
    // Polymarket uses 6-decimal fixed math for both collateral and shares.
    Ok((value * 1_000_000.0).round() as u128)
}

fn zero_bytes32() -> [u8; 32] {
    [0u8; 32]
}

#[derive(Debug, Clone, Serialize, Eip712)]
#[eip712(
    name = "Polymarket CTF Exchange",
    version = "2",
    chain_id = 137,
    verifying_contract = "0xE111180000d2663C0091e4f400237545B87B996B"
)]
#[serde(rename_all = "camelCase")]
struct SignedOrderV2 {
    pub salt: U256,
    pub maker: Address,
    pub signer: Address,
    pub token_id: U256,
    pub maker_amount: U256,
    pub taker_amount: U256,
    pub side: u8,
    pub signature_type: u8,
    pub timestamp: U256,
    pub metadata: [u8; 32],
    pub builder: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Eip712)]
#[eip712(
    name = "Polymarket CTF Exchange",
    version = "2",
    chain_id = 137,
    verifying_contract = "0xe2222d279d744050d28e00520010520000310F59"
)]
#[serde(rename_all = "camelCase")]
struct SignedOrderV2NegRisk {
    pub salt: U256,
    pub maker: Address,
    pub signer: Address,
    pub token_id: U256,
    pub maker_amount: U256,
    pub taker_amount: U256,
    pub side: u8,
    pub signature_type: u8,
    pub timestamp: U256,
    pub metadata: [u8; 32],
    pub builder: [u8; 32],
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OrderOptions {
    tick_size: String,
    neg_risk: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PostOrderBody {
    order: serde_json::Value,
    owner: String,
    #[serde(rename = "orderType")]
    order_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiration: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    post_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    defer_exec: Option<bool>,
}

pub struct ClobClient {
    client: Client,
    base_url: String,
    chain_id: u64,
    wallet: LocalWallet,
    creds: Option<ApiKeyCreds>,
    owner: Option<String>,
    signature_type: u8,
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
        let owner = std::env::var("CLOB_OWNER").ok().filter(|s| !s.trim().is_empty());
        let signature_type = std::env::var("CLOB_SIGNATURE_TYPE")
            .ok()
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(0);
        Ok(ClobClient {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            chain_id,
            wallet,
            creds,
            owner,
            signature_type,
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
        let timestamp = now_unix_seconds().to_string();
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
        // CLOB v2 user orders endpoint.
        let path = "/data/orders";
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
        let v: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
        // v2 response is paginated: { data: [...], ... }
        if let Some(arr) = v.get("data").and_then(|x| x.as_array()) {
            let orders: Vec<OpenOrder> =
                serde_json::from_value(serde_json::Value::Array(arr.clone()))
                    .map_err(|e| e.to_string())?;
            return Ok(orders);
        }
        // Some deployments may still return a plain array.
        if v.is_array() {
            let orders: Vec<OpenOrder> = serde_json::from_value(v).map_err(|e| e.to_string())?;
            return Ok(orders);
        }
        Ok(vec![])
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
        logger::success("CLOB API balance allowance updated");
        Ok(())
    }

    pub async fn get_last_trade_price(&self, token_id: &str) -> Result<f64, String> {
        let path = "/last-trade-price";
        let url = format!("{}{}", self.base_url, path);
        let res = self
            .client
            .get(&url)
            .query(&[("token_id", token_id)])
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            let t = res.text().await.unwrap_or_default();
            return Err(format!("get_last_trade_price failed: {}", t));
        }
        let out: LastTradePriceResponse = res.json().await.map_err(|e| e.to_string())?;
        out.price.parse::<f64>().map_err(|e| e.to_string())
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
        let side_upper = side.to_uppercase();
        let side_u8 = match side_upper.as_str() {
            "BUY" => 0u8,
            "SELL" => 1u8,
            _ => return Err(format!("Invalid side: {}", side)),
        };

        // We always need a price to compute maker/taker amounts for a signed order.
        let px = match price {
            Some(p) if p.is_finite() && p > 0.0 => p,
            _ => self.get_last_trade_price(token_id).await?,
        };

        // V2 signed order uses fixed-math ints (1e6).
        let (maker_amount, taker_amount) = if side_u8 == 0 {
            // BUY: caller's `amount` is collateral amount (pUSD). Shares = amount / price.
            let collateral = amount;
            let shares = collateral / px;
            (to_fixed_1e6(collateral)?, to_fixed_1e6(shares)?)
        } else {
            // SELL: caller's `amount` is shares. Collateral = shares * price.
            let shares = amount;
            let collateral = shares * px;
            (to_fixed_1e6(shares)?, to_fixed_1e6(collateral)?)
        };

        let maker_amount_u256 = U256::from(maker_amount);
        let taker_amount_u256 = U256::from(taker_amount);
        let token_u256 = U256::from_dec_str(token_id)
            .or_else(|_| U256::from_str_radix(token_id.trim_start_matches("0x"), 16))
            .map_err(|_| format!("Invalid token_id for v2 order (expected numeric/hex): {}", token_id))?;

        let ts_ms = now_unix_millis();
        let salt = U256::from(ts_ms);
        let timestamp = U256::from(ts_ms);
        let signature_type = self.signature_type;

        // Sign EIP-712 v2 order.
        let signature_hex = if neg_risk {
            let order = SignedOrderV2NegRisk {
                salt,
                maker: self.wallet.address(),
                signer: self.wallet.address(),
                token_id: token_u256,
                maker_amount: maker_amount_u256,
                taker_amount: taker_amount_u256,
                side: side_u8,
                signature_type,
                timestamp,
                metadata: zero_bytes32(),
                builder: zero_bytes32(),
            };
            self.wallet
                .sign_typed_data(&order)
                .await
                .map_err(|e| e.to_string())?
                .to_string()
        } else {
            let order = SignedOrderV2 {
                salt,
                maker: self.wallet.address(),
                signer: self.wallet.address(),
                token_id: token_u256,
                maker_amount: maker_amount_u256,
                taker_amount: taker_amount_u256,
                side: side_u8,
                signature_type,
                timestamp,
                metadata: zero_bytes32(),
                builder: zero_bytes32(),
            };
            self.wallet
                .sign_typed_data(&order)
                .await
                .map_err(|e| e.to_string())?
                .to_string()
        };

        let order_json = serde_json::json!({
            "salt": salt.to_string(),
            "maker": format!("{:?}", self.wallet.address()),
            "signer": format!("{:?}", self.wallet.address()),
            "tokenId": token_id,
            "makerAmount": maker_amount_u256.to_string(),
            "takerAmount": taker_amount_u256.to_string(),
            "side": side_upper,
            "expiration": "0",
            "timestamp": ts_ms.to_string(),
            "metadata": format!("0x{}", hex::encode(zero_bytes32())),
            "builder": format!("0x{}", hex::encode(zero_bytes32())),
            "signature": signature_hex,
            "signatureType": signature_type
        });

        let owner = self
            .owner
            .clone()
            .or_else(|| self.creds.as_ref().map(|c| c.api_key.clone()))
            .ok_or("Missing owner: set CLOB_OWNER or provide api_key creds")?;
        let body = PostOrderBody {
            order: order_json,
            owner,
            order_type: order_type.to_uppercase(),
            expiration: None,
            post_only: Some(false),
            defer_exec: Some(false),
        };
        let body_str = serde_json::to_string(&body).map_err(|e| e.to_string())?;

        let path = "/order";
        let url = format!("{}{}", self.base_url, path);
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
