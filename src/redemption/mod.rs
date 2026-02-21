use crate::logger;
use std::collections::HashMap;

#[derive(Debug)]
pub struct AutoRedeemResult {
    pub total: usize,
    pub resolved: usize,
    pub redeemed: usize,
    pub failed: usize,
    pub results: Vec<AutoRedeemItem>,
}

#[derive(Debug)]
pub struct AutoRedeemItem {
    pub condition_id: String,
    pub is_resolved: bool,
    pub redeemed: bool,
    pub error: Option<String>,
}

pub async fn redeem_positions(
    _condition_id: &str,
    _index_sets: Option<&[u64]>,
    _chain_id: Option<u64>,
) -> Result<serde_json::Value, String> {
    logger::warning("Rust redeem_positions is a stub. Use: npm run redeem -- <conditionId>");
    Err("Use TypeScript redeem CLI for on-chain redemption".into())
}

pub async fn check_condition_resolution(
    _condition_id: &str,
    _chain_id: Option<u64>,
) -> Result<ConditionResolution, String> {
    Ok(ConditionResolution {
        is_resolved: false,
        winning_index_sets: vec![],
        reason: Some("Rust stub".into()),
    })
}

pub struct ConditionResolution {
    pub is_resolved: bool,
    pub winning_index_sets: Vec<u64>,
    pub reason: Option<String>,
}

pub async fn get_user_token_balances(
    _condition_id: &str,
    _wallet_address: &str,
    _chain_id: Option<u64>,
) -> Result<HashMap<u64, String>, String> {
    Ok(HashMap::new())
}

pub async fn redeem_market(
    condition_id: &str,
    _chain_id: Option<u64>,
    _max_retries: u32,
) -> Result<serde_json::Value, String> {
    redeem_positions(condition_id, None, _chain_id).await
}

pub async fn is_market_resolved(
    _condition_id: &str,
) -> Result<IsMarketResolvedResult, String> {
    Ok(IsMarketResolvedResult {
        is_resolved: false,
        reason: Some("Rust stub".into()),
        winning_index_sets: None,
    })
}

pub struct IsMarketResolvedResult {
    pub is_resolved: bool,
    pub reason: Option<String>,
    pub winning_index_sets: Option<Vec<u64>>,
}

pub async fn auto_redeem_resolved_markets(
    options: Option<AutoRedeemOptions>,
) -> Result<AutoRedeemResult, String> {
    let _ = options;
    let holdings = crate::utils::holdings::get_all_holdings();
    let market_ids: Vec<String> = holdings.keys().cloned().collect();
    let mut resolved = 0usize;
    let mut redeemed = 0usize;
    let mut failed = 0usize;
    let mut results = Vec::new();

    logger::info(&format!(
        "=== AUTO-REDEEM: Checking {} markets ===",
        market_ids.len()
    ));

    for condition_id in &market_ids {
        match is_market_resolved(condition_id).await {
            Ok(r) => {
                if r.is_resolved {
                    resolved += 1;
                    match redeem_market(condition_id, None, 3).await {
                        Ok(_) => {
                            redeemed += 1;
                            crate::utils::clear_market_holdings(condition_id);
                            results.push(AutoRedeemItem {
                                condition_id: condition_id.clone(),
                                is_resolved: true,
                                redeemed: true,
                                error: None,
                            });
                        }
                        Err(e) => {
                            failed += 1;
                            results.push(AutoRedeemItem {
                                condition_id: condition_id.clone(),
                                is_resolved: true,
                                redeemed: false,
                                error: Some(e),
                            });
                        }
                    }
                } else {
                    results.push(AutoRedeemItem {
                        condition_id: condition_id.clone(),
                        is_resolved: false,
                        redeemed: false,
                        error: r.reason,
                    });
                }
            }
            Err(e) => {
                failed += 1;
                results.push(AutoRedeemItem {
                    condition_id: condition_id.clone(),
                    is_resolved: false,
                    redeemed: false,
                    error: Some(e),
                });
            }
        }
    }

    logger::info("=== AUTO-REDEEM SUMMARY ===");
    logger::info(&format!("Total: {}, Resolved: {}, Redeemed: {}, Failed: {}", market_ids.len(), resolved, redeemed, failed));
    Ok(AutoRedeemResult {
        total: market_ids.len(),
        resolved,
        redeemed,
        failed,
        results,
    })
}

pub struct AutoRedeemOptions {
    pub clear_holdings_after_redeem: bool,
    pub dry_run: bool,
    pub max_retries: u32,
}
