use polymarket_copy_trading_bot::logger;
use polymarket_copy_trading_bot::redemption::auto_redeem_resolved_markets;
use polymarket_copy_trading_bot::utils::get_all_holdings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    let args: Vec<String> = std::env::args().collect();
    let dry_run = args.iter().any(|a| a == "--dry-run");
    let use_api = args.iter().any(|a| a == "--api");

    if use_api {
        logger::info("=== USING POLYMARKET API METHOD ===");
        logger::warning("Rust binary does not implement --api yet. Use: npm run auto-redeem -- --api");
        return Ok(());
    }

    let holdings = get_all_holdings();
    if holdings.is_empty() {
        logger::warning("No holdings in token-holding.json. Use --api with TypeScript for API-based redemption.");
        return Ok(());
    }

    logger::info(&format!("Found {} market(s) in holdings", holdings.len()));
    let options = Some(polymarket_copy_trading_bot::redemption::AutoRedeemOptions {
        clear_holdings_after_redeem: false,
        dry_run,
        max_retries: 3,
    });
    match auto_redeem_resolved_markets(options).await {
        Ok(r) => {
            logger::info(&format!("Total: {}, Resolved: {}, Redeemed: {}, Failed: {}", r.total, r.resolved, r.redeemed, r.failed));
        }
        Err(e) => {
            logger::info(&format!("Error: {}", e));
            std::process::exit(1);
        }
    }
    Ok(())
}
