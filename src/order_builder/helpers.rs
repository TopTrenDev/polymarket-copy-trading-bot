use crate::config::TickSize;
use crate::order_builder::CopyTradeOptions;
use crate::utils::types::TradePayload;

pub fn parse_trade_side(side: &str) -> Result<&'static str, String> {
    let u = side.to_uppercase();
    if u == "BUY" {
        Ok("BUY")
    } else if u == "SELL" {
        Ok("SELL")
    } else {
        Err(format!("Invalid trade side: {}", side))
    }
}

pub fn calculate_market_order_amount(
    trade: &TradePayload,
    size_multiplier: f64,
    max_amount: Option<f64>,
) -> f64 {
    let adjusted = trade.size * size_multiplier;
    if trade.side.to_uppercase() == "BUY" {
        let mut amount = trade.price * adjusted;
        if amount < 1.0 {
            return 1.0;
        }
        if let Some(max) = max_amount {
            if amount > max {
                return max;
            }
        }
        amount
    } else {
        adjusted
    }
}

pub fn trade_to_market_order(options: &CopyTradeOptions) -> (String, String, f64, Option<f64>) {
    let side = parse_trade_side(&options.trade.side).unwrap_or("BUY");
    let amount = calculate_market_order_amount(
        &options.trade,
        options.size_multiplier,
        options.max_amount,
    );
    let price = if options.trade.price > 0.0 {
        Some(options.trade.price)
    } else {
        None
    };
    (
        options.trade.asset.clone(),
        side.to_string(),
        amount,
        price,
    )
}
