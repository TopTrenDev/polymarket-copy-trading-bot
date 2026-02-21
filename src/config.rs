use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub private_key: String,
    pub target_wallet: String,
    pub size_multiplier: f64,
    pub max_order_amount: Option<f64>,
    pub order_type_fok: bool,
    pub tick_size: TickSize,
    pub neg_risk: bool,
    pub enable_copy_trading: bool,
    pub redeem_duration_minutes: Option<u64>,
    pub chain_id: u64,
    pub clob_api_url: String,
    pub user_real_time_data_url: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TickSize {
    Tick01,
    Tick001,
    Tick0001,
    Tick00001,
}

impl TickSize {
    pub fn as_str(&self) -> &'static str {
        match self {
            TickSize::Tick01 => "0.1",
            TickSize::Tick001 => "0.01",
            TickSize::Tick0001 => "0.001",
            TickSize::Tick00001 => "0.0001",
        }
    }
}

impl Default for TickSize {
    fn default() -> Self {
        TickSize::Tick001
    }
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let private_key = env::var("PRIVATE_KEY").map_err(|_| "PRIVATE_KEY not set")?;
        let target_wallet =
            env::var("TARGET_WALLET").map_err(|_| "TARGET_WALLET environment variable is not set")?;

        let size_multiplier = env::var("SIZE_MULTIPLIER")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1.0);

        let max_order_amount = env::var("MAX_ORDER_AMOUNT").ok().and_then(|s| s.parse().ok());

        let order_type = env::var("ORDER_TYPE").unwrap_or_else(|_| "FAK".into());
        let order_type_fok = order_type.to_uppercase() == "FOK";

        let tick_str = env::var("TICK_SIZE").unwrap_or_else(|_| "0.01".into());
        let tick_size = match tick_str.as_str() {
            "0.1" => TickSize::Tick01,
            "0.001" => TickSize::Tick0001,
            "0.0001" => TickSize::Tick00001,
            _ => TickSize::Tick001,
        };

        let neg_risk = env::var("NEG_RISK").as_deref() == Ok("true");
        let enable_copy_trading = env::var("ENABLE_COPY_TRADING").as_deref() != Some("false");

        let redeem_duration_minutes = env::var("REDEEM_DURATION")
            .ok()
            .and_then(|s| s.parse().ok());

        let chain_id = env::var("CHAIN_ID")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(137);

        let clob_api_url =
            env::var("CLOB_API_URL").unwrap_or_else(|_| "https://clob.polymarket.com".into());
        let user_real_time_data_url = env::var("USER_REAL_TIME_DATA_URL")
            .unwrap_or_else(|_| "wss://ws-live-data.polymarket.com".into());

        Ok(Config {
            private_key,
            target_wallet,
            size_multiplier,
            max_order_amount,
            order_type_fok,
            tick_size,
            neg_risk,
            enable_copy_trading,
            redeem_duration_minutes,
            chain_id,
            clob_api_url,
            user_real_time_data_url,
        })
    }
}
