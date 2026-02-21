use polymarket_copy_trading_bot::logger;
use polymarket_copy_trading_bot::redemption::redeem_market;
use polymarket_copy_trading_bot::utils::clear_market_holdings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    let args: Vec<String> = std::env::args().collect();
    let condition_id: Option<String> = args.get(1).cloned().or_else(|| std::env::var("CONDITION_ID").ok());
    if let Some(ref cid) = condition_id {
        logger::info(&format!("Redeeming positions for condition: {}", cid));
        match redeem_market(cid, None, 3).await {
            Ok(_) => {
                logger::success("✅ Successfully redeemed positions!");
                let _ = clear_market_holdings(cid);
            }
            Err(e) => {
                logger::info(&format!("❌ Failed to redeem: {}", e));
                std::process::exit(1);
            }
        }
    } else {
        logger::info("Usage: redeem <conditionId> or set CONDITION_ID in .env");
        std::process::exit(1);
    }
    Ok(())
}
