use crate::logger;
use crate::providers::{chain_id_from_env, get_rpc_url, ClobClient};

pub async fn approve_usdc_allowance() -> Result<(), String> {
    let _pk = std::env::var("PRIVATE_KEY").map_err(|_| "PRIVATE_KEY not found")?;
    let chain_id = chain_id_from_env();
    let _rpc = get_rpc_url(chain_id);
    logger::info(&format!(
        "Approving pUSD/CTF allowances on chain {} (run TypeScript bot once if you need on-chain approvals)",
        chain_id
    ));
    logger::warning("Rust allowance module is a stub. Use the TypeScript bot to approve allowances, or implement ethers contract calls here.");
    Ok(())
}

pub async fn update_clob_balance_allowance(
    client: &mut ClobClient,
) -> Result<(), String> {
    logger::info("Updating CLOB API balance allowance for pUSD...");
    client.update_balance_allowance("COLLATERAL").await?;
    Ok(())
}

pub async fn approve_tokens_after_buy() -> Result<(), String> {
    logger::debug("approve_tokens_after_buy (stub)");
    Ok(())
}
