use crate::logger;
use crate::providers::clob::{ClobClient, BalanceAllowanceResponse, OpenOrder};

const ASSET_TYPE_COLLATERAL: &str = "COLLATERAL";
const ASSET_TYPE_CONDITIONAL: &str = "CONDITIONAL";

pub async fn get_available_balance(
    client: &mut ClobClient,
    asset_type: &str,
    token_id: Option<&str>,
) -> Result<f64, String> {
    let balance_response = client
        .get_balance_allowance(asset_type, token_id)
        .await?;
    let total_balance: f64 = balance_response
        .balance
        .as_deref()
        .unwrap_or("0")
        .parse()
        .unwrap_or(0.0);

    let open_orders = client.get_open_orders(token_id).await?;
    let mut reserved_amount = 0.0f64;
    for order in &open_orders {
        let order_side = order.side.to_uppercase();
        let is_buy = order_side == "BUY";
        let is_sell = order_side == "SELL";
        if (asset_type == ASSET_TYPE_COLLATERAL && is_buy)
            || (asset_type == ASSET_TYPE_CONDITIONAL && is_sell)
        {
            let order_size: f64 = order
                .original_size
                .as_deref()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0.0);
            let size_matched: f64 = order
                .size_matched
                .as_deref()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0.0);
            reserved_amount += order_size - size_matched;
        }
    }
    let available = (total_balance - reserved_amount).max(0.0);
    logger::debug(&format!(
        "Balance check: Total={}, Reserved={}, Available={}",
        total_balance, reserved_amount, available
    ));
    Ok(available)
}

pub async fn display_wallet_balance(client: &mut ClobClient) -> Result<(), String> {
    let r = client
        .get_balance_allowance(ASSET_TYPE_COLLATERAL, None)
        .await?;
    let balance: f64 = r.balance.as_deref().unwrap_or("0").parse().unwrap_or(0.0);
    let allowance: f64 = r.allowance.as_deref().unwrap_or("0").parse().unwrap_or(0.0);
    logger::info("═══════════════════════════════════════");
    logger::info("💰 WALLET BALANCE & ALLOWANCE");
    logger::info("═══════════════════════════════════════");
    logger::info(&format!("USDC Balance: {:.6}", balance));
    logger::info(&format!("USDC Allowance: {:.6}", allowance));
    logger::info(&format!(
        "Available: {:.6} (Balance: {:.6}, Allowance: {:.6})",
        balance.min(allowance),
        balance,
        allowance
    ));
    logger::info("═══════════════════════════════════════");
    Ok(())
}

pub struct ValidateBalanceResult {
    pub valid: bool,
    pub available: f64,
    pub required: f64,
    pub balance: Option<f64>,
    pub allowance: Option<f64>,
}

pub async fn validate_buy_order_balance(
    client: &mut ClobClient,
    required_amount: f64,
) -> Result<ValidateBalanceResult, String> {
    let r = client
        .get_balance_allowance(ASSET_TYPE_COLLATERAL, None)
        .await?;
    let balance: f64 = r.balance.as_deref().unwrap_or("0").parse().unwrap_or(0.0);
    let allowance: f64 = r.allowance.as_deref().unwrap_or("0").parse().unwrap_or(0.0);
    let available = get_available_balance(client, ASSET_TYPE_COLLATERAL, None).await?;
    let valid = available >= required_amount;
    if !valid {
        logger::warning("═══════════════════════════════════════");
        logger::warning("⚠️  INSUFFICIENT BALANCE/ALLOWANCE");
        logger::warning("═══════════════════════════════════════");
        logger::warning(&format!("Required: {:.6} USDC", required_amount));
        logger::warning(&format!("Available: {:.6} USDC", available));
        logger::warning(&format!("Balance: {:.6} USDC", balance));
        logger::warning(&format!("Allowance: {:.6} USDC", allowance));
        logger::warning("═══════════════════════════════════════");
    }
    Ok(ValidateBalanceResult {
        valid,
        available,
        required: required_amount,
        balance: Some(balance),
        allowance: Some(allowance),
    })
}

pub async fn validate_sell_order_balance(
    client: &mut ClobClient,
    token_id: &str,
    required_amount: f64,
) -> Result<ValidateBalanceResult, String> {
    let available =
        get_available_balance(client, ASSET_TYPE_CONDITIONAL, Some(token_id)).await?;
    let valid = available >= required_amount;
    if !valid {
        logger::warning(&format!(
            "Insufficient token balance: Token={}..., Required={}, Available={}",
            &token_id[..token_id.len().min(20)],
            required_amount,
            available
        ));
    }
    Ok(ValidateBalanceResult {
        valid,
        available,
        required: required_amount,
        balance: None,
        allowance: None,
    })
}
