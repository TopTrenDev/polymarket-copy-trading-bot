use crate::utils::types::TradePayload;
use crate::config::TickSize;

#[derive(Clone)]
pub struct CopyTradeOptions {
    pub trade: TradePayload,
    pub size_multiplier: f64,
    pub max_amount: Option<f64>,
    pub order_type_fok: bool,
    pub tick_size: TickSize,
    pub neg_risk: bool,
}

pub struct CopyTradeResult {
    pub success: bool,
    pub order_id: Option<String>,
    pub error: Option<String>,
    pub transaction_hashes: Option<Vec<String>>,
}
