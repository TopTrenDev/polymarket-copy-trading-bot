use polymarket_copy_trading_bot::config::Config;
use polymarket_copy_trading_bot::logger;
use polymarket_copy_trading_bot::order_builder::{CopyTradeOptions, TradeOrderBuilder};
use polymarket_copy_trading_bot::providers::wss::parse_trade_payload;
use polymarket_copy_trading_bot::providers::{ClobClient, RealTimeWsClient};
use polymarket_copy_trading_bot::redemption::auto_redeem_resolved_markets;
use polymarket_copy_trading_bot::security::{
    approve_usdc_allowance, create_credential, update_clob_balance_allowance,
};
use polymarket_copy_trading_bot::utils::balance::display_wallet_balance;
use polymarket_copy_trading_bot::utils::types::TradePayload;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    logger::info("Starting the bot...");

    let config = Config::from_env().map_err(|e| e.to_string())?;

    logger::info("Configuration:");
    logger::info(&format!("  Target Wallet: {}", config.target_wallet));
    logger::info(&format!("  Size Multiplier: {}x", config.size_multiplier));
    logger::info(&format!(
        "  Max Order Amount: {}",
        config
            .max_order_amount
            .map(|a| a.to_string())
            .unwrap_or_else(|| "unlimited".into())
    ));
    logger::info(&format!(
        "  Order Type: {}",
        if config.order_type_fok { "FOK" } else { "FAK" }
    ));
    logger::info(&format!("  Tick Size: {}", config.tick_size.as_str()));
    logger::info(&format!("  Neg Risk: {}", config.neg_risk));
    logger::info(&format!(
        "  Copy Trading: {}",
        if config.enable_copy_trading {
            "enabled"
        } else {
            "disabled"
        }
    ));

    let _ = create_credential().await;
    logger::info("Credentials ready");

    let mut clob_client: Option<ClobClient> = None;
    if config.enable_copy_trading {
        match ClobClient::new(
            &config.clob_api_url,
            config.chain_id,
            &config.private_key,
        )
        .await
        {
            Ok(c) => clob_client = Some(c),
            Err(e) => {
                logger::info(&format!("ClobClient init: {}", e));
                logger::warning("Continuing without ClobClient - orders may fail");
            }
        }
    }

    if config.enable_copy_trading {
        if let Some(ref mut client) = clob_client {
            let _ = approve_usdc_allowance().await;
            let _ = update_clob_balance_allowance(client).await;
            let _ = display_wallet_balance(client).await;
        }
    }

    let (tx, mut rx) = mpsc::unbounded_channel::<CopyTradeOptions>();
    let builder = clob_client.map(TradeOrderBuilder::new);
    let copy_trading_paused = std::sync::Arc::new(AtomicBool::new(false));
    let paused_clone = std::sync::Arc::clone(&copy_trading_paused);

    if let Some(mut order_builder) = builder {
        tokio::spawn(async move {
            while let Some(opts) = rx.recv().await {
                if !paused_clone.load(Ordering::SeqCst) {
                    let result = order_builder.copy_trade(opts).await;
                    if result.success {
                        logger::success(&format!("✅ Trade copied! OrderID: {:?}", result.order_id));
                    } else {
                        logger::info(&format!("❌ Copy failed: {:?}", result.error));
                    }
                }
            }
        });
    }

    let target_wallet = config.target_wallet.to_lowercase();
    let size_multiplier = config.size_multiplier;
    let max_amount = config.max_order_amount;
    let order_type_fok = config.order_type_fok;
    let tick_size = config.tick_size;
    let neg_risk = config.neg_risk;
    let enable_copy_trading = config.enable_copy_trading;

    let on_message = move |msg: polymarket_copy_trading_bot::providers::wss::WsMessage| {
        if msg.topic.as_deref() != Some("activity") || msg.msg_type.as_deref() != Some("trades") {
            return;
        }
        let payload = match &msg.payload {
            Some(p) => p,
            None => return,
        };
        let trade: TradePayload = match parse_trade_payload(payload) {
            Some(t) => t,
            None => return,
        };
        let proxy = match trade.proxy_wallet_lower() {
            Some(p) => p,
            None => return,
        };
        if proxy != target_wallet {
            return;
        }

        logger::warning(&format!(
            "🎯 Trade detected! Side: {}, Price: {}, Size: {}, Market: {:?}",
            trade.side,
            trade.price,
            trade.size,
            trade.title.as_deref().or(trade.slug.as_deref())
        ));

        if enable_copy_trading {
            let opts = CopyTradeOptions {
                trade,
                size_multiplier,
                max_amount,
                order_type_fok,
                tick_size,
                neg_risk,
            };
            let _ = tx.send(opts);
        }
    };

    let on_connect = || {
        logger::success("Connected to the server");
    };

    let client = RealTimeWsClient::new(on_message, on_connect);
    client.connect();
    logger::success("Bot started successfully");

    if let Some(mins) = config.redeem_duration_minutes {
        if mins > 0 {
            logger::info(&format!(
                "\n⏰ Auto-redemption scheduled: Every {} minutes",
                mins
            ));
            let paused = std::sync::Arc::clone(&copy_trading_paused);
            tokio::spawn(async move {
                let mut interval =
                    tokio::time::interval(Duration::from_millis(mins * 60 * 1000));
                loop {
                    interval.tick().await;
                    paused.store(true, Ordering::SeqCst);
                    logger::info("\n========== AUTOMATIC REDEMPTION ==========");
                    let _ = auto_redeem_resolved_markets(None).await;
                    paused.store(false, Ordering::SeqCst);
                }
            });
        }
    }

    tokio::signal::ctrl_c().await?;
    Ok(())
}
