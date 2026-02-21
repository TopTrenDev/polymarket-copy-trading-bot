use ethers::prelude::*;

pub fn get_rpc_url(chain_id: u64) -> String {
    let rpc_token = std::env::var("RPC_TOKEN").ok();
    match chain_id {
        137 => rpc_token
            .map(|t| format!("https://polygon-mainnet.g.alchemy.com/v2/{}", t))
            .unwrap_or_else(|| "https://polygon-rpc.com".into()),
        80002 => rpc_token
            .map(|t| format!("https://polygon-amoy.g.alchemy.com/v2/{}", t))
            .unwrap_or_else(|| "https://rpc-amoy.polygon.technology".into()),
        _ => panic!(
            "Unsupported chain ID: {}. Supported: 137 (Polygon), 80002 (Amoy)",
            chain_id
        ),
    }
}

pub fn chain_id_from_env() -> u64 {
    std::env::var("CHAIN_ID")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(137)
}

pub fn provider(chain_id: u64) -> Result<Provider<Http>, String> {
    let url = get_rpc_url(chain_id);
    Provider::<Http>::try_from(url.as_str()).map_err(|e| e.to_string())
}
