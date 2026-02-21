mod builder;
mod helpers;
mod types;

pub use builder::TradeOrderBuilder;
pub use helpers::{calculate_market_order_amount, parse_trade_side, trade_to_market_order};
pub use types::{CopyTradeOptions, CopyTradeResult};
