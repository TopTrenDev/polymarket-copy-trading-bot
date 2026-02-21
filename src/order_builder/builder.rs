use crate::config::TickSize;
use crate::logger;
use crate::order_builder::helpers::trade_to_market_order;
use crate::order_builder::{CopyTradeOptions, CopyTradeResult};
use crate::providers::ClobClient;
use crate::security::approve_tokens_after_buy;
use crate::utils::balance::{display_wallet_balance, validate_buy_order_balance};
use crate::utils::{add_holdings, get_holdings, remove_holdings};

pub struct TradeOrderBuilder {
    client: ClobClient,
}

impl TradeOrderBuilder {
    pub fn new(client: ClobClient) -> Self {
        TradeOrderBuilder { client }
    }

    pub async fn copy_trade(&mut self, options: CopyTradeOptions) -> CopyTradeResult {
        let tick_size = options.tick_size;
        let neg_risk = options.neg_risk;
        let order_type = if options.order_type_fok { "FOK" } else { "FAK" };
        let market_id = options.trade.condition_id.clone();
        let token_id = options.trade.asset.clone();

        if options.trade.side.to_uppercase() == "SELL" {
            let holdings_amount = get_holdings(&market_id, &token_id);
            if holdings_amount <= 0.0 {
                logger::warning(&format!(
                    "No holdings for token {} in market {}. Skipping SELL.",
                    token_id, market_id
                ));
                return CopyTradeResult {
                    success: false,
                    order_id: None,
                    error: Some("No holdings available to sell".into()),
                    transaction_hashes: None,
                };
            }
            let sell_amount = holdings_amount;
            logger::info(&format!(
                "Placing SELL market order: {} shares (type: {})",
                sell_amount, order_type
            ));
            match self
                .client
                .create_and_post_market_order(
                    &token_id,
                    "SELL",
                    sell_amount,
                    order_type,
                    tick_size,
                    neg_risk,
                    None,
                )
                .await
            {
                Ok(resp) => {
                    let tokens_sold = resp
                        .making_amount
                        .as_ref()
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(sell_amount);
                    if tokens_sold > 0.0 {
                        remove_holdings(&market_id, &token_id, tokens_sold);
                    }
                    logger::success(&format!(
                        "SELL order executed! OrderID: {:?}, Tokens sold: {}",
                        resp.order_id(),
                        tokens_sold
                    ));
                    return CopyTradeResult {
                        success: true,
                        order_id: resp.order_id.map(String::from),
                        error: None,
                        transaction_hashes: resp.transactions_hashes.clone(),
                    };
                }
                Err(e) => {
                    logger::error_msg("SELL order failed", None);
                    return CopyTradeResult {
                        success: false,
                        order_id: None,
                        error: Some(e),
                        transaction_hashes: None,
                    };
                }
            }
        }

        logger::info(&format!(
            "Building order: {} {} @ {} for token {}...",
            options.trade.side,
            options.trade.size,
            options.trade.price,
            token_id.chars().take(20).collect::<String>()
        ));

        let (token_id, side, amount, price) = trade_to_market_order(&options);
        let mut amount = amount;

        if let Err(e) = self
            .client
            .update_balance_allowance("COLLATERAL")
            .await
        {
            logger::warning(&format!("Failed to update balance allowance: {}", e));
        }
        let _ = display_wallet_balance(&mut self.client).await;
        let balance_check = match validate_buy_order_balance(&mut self.client, amount).await {
            Ok(b) => b,
            Err(e) => {
                return CopyTradeResult {
                    success: false,
                    order_id: None,
                    error: Some(e),
                    transaction_hashes: None,
                };
            }
        };
        if !balance_check.valid {
            if balance_check.available <= 0.0 {
                return CopyTradeResult {
                    success: false,
                    order_id: None,
                    error: Some(format!(
                        "Insufficient USDC. Available: {}",
                        balance_check.available
                    )),
                    transaction_hashes: None,
                };
            }
            amount = balance_check.available;
            logger::info(&format!("Adjusted order amount to available: {}", amount));
        }

        logger::info(&format!(
            "Placing {} market order: {} (type: {})",
            side, amount, order_type
        ));
        match self
            .client
            .create_and_post_market_order(
                &token_id,
                &side,
                amount,
                order_type,
                tick_size,
                neg_risk,
                price,
            )
            .await
        {
            Ok(resp) => {
                let tokens_received = resp
                    .taking_amount
                    .as_ref()
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                if tokens_received > 0.0 {
                    add_holdings(&market_id, &token_id, tokens_received);
                } else {
                    let estimated = amount / options.trade.price.max(1.0);
                    if estimated > 0.0 {
                        add_holdings(&market_id, &token_id, estimated);
                        logger::warning(&format!(
                            "Using estimated token amount: {} (actual not in response)",
                            estimated
                        ));
                    }
                }
                let _ = approve_tokens_after_buy().await;
                logger::success(&format!(
                    "BUY order executed! OrderID: {:?}, Tokens: {}",
                    resp.order_id(),
                    tokens_received
                ));
                CopyTradeResult {
                    success: true,
                    order_id: resp.order_id.map(String::from),
                    error: None,
                    transaction_hashes: resp.transactions_hashes.clone(),
                }
            }
            Err(e) => {
                logger::error_msg("Copy trade failed", None);
                CopyTradeResult {
                    success: false,
                    order_id: None,
                    error: Some(e),
                    transaction_hashes: None,
                }
            }
        }
    }
}
