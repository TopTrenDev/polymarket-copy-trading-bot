use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::logger;
use serde::{Deserialize, Serialize};

pub type TokenHoldings = HashMap<String, HashMap<String, f64>>;

fn holdings_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("src/data/token-holding.json")
}

pub fn load_holdings() -> TokenHoldings {
    let path = holdings_path();
    if !path.exists() {
        return TokenHoldings::new();
    }
    match fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_else(|_| TokenHoldings::new()),
        Err(e) => {
            logger::error_msg("Failed to load holdings", Some(&e));
            TokenHoldings::new()
        }
    }
}

pub fn save_holdings(holdings: &TokenHoldings) {
    let path = holdings_path();
    if let Err(e) = fs::create_dir_all(path.parent().unwrap_or(&PathBuf::from("."))) {
        logger::error_msg("Failed to create data dir", Some(&e));
        return;
    }
    if let Err(e) = fs::write(path, serde_json::to_string_pretty(holdings).unwrap_or_default()) {
        logger::error_msg("Failed to save holdings", Some(&e));
    }
}

pub fn add_holdings(market_id: &str, token_id: &str, amount: f64) {
    let mut holdings = load_holdings();
    holdings
        .entry(market_id.to_string())
        .or_default()
        .entry(token_id.to_string())
        .and_modify(|a| *a += amount)
        .or_insert(amount);
    save_holdings(&holdings);
    logger::info(&format!(
        "Added {} tokens to holdings: {} -> {}",
        amount, market_id, token_id
    ));
}

pub fn get_holdings(market_id: &str, token_id: &str) -> f64 {
    let holdings = load_holdings();
    holdings
        .get(market_id)
        .and_then(|m| m.get(token_id).copied())
        .unwrap_or(0.0)
}

pub fn remove_holdings(market_id: &str, token_id: &str, amount: f64) {
    let mut holdings = load_holdings();
    let Some(market) = holdings.get_mut(market_id) else {
        logger::warning(&format!("No holdings found for {} -> {}", market_id, token_id));
        return;
    };
    let Some(current) = market.get_mut(token_id) else {
        logger::warning(&format!("No holdings found for {} -> {}", market_id, token_id));
        return;
    };
    let new_amount = (*current - amount).max(0.0);
    if new_amount == 0.0 {
        market.remove(token_id);
        if market.is_empty() {
            holdings.remove(market_id);
        }
    } else {
        *current = new_amount;
    }
    save_holdings(&holdings);
    logger::info(&format!(
        "Removed {} tokens from holdings: {} -> {} (remaining: {})",
        amount, market_id, token_id, new_amount
    ));
}

pub fn get_market_holdings(market_id: &str) -> HashMap<String, f64> {
    load_holdings()
        .get(market_id)
        .cloned()
        .unwrap_or_default()
}

pub fn get_all_holdings() -> TokenHoldings {
    load_holdings()
}

pub fn clear_market_holdings(market_id: &str) {
    let mut holdings = load_holdings();
    if holdings.remove(market_id).is_some() {
        save_holdings(&holdings);
        logger::info(&format!("Cleared holdings for market: {}", market_id));
    } else {
        logger::warning(&format!("No holdings found for market: {}", market_id));
    }
}

pub fn clear_holdings() {
    save_holdings(&TokenHoldings::new());
    logger::info("All holdings cleared");
}
