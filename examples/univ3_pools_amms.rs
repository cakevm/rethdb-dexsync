use alloy::primitives::address;
use alloy::providers::ProviderBuilder;
use amms::amm::factory::Factory;
use amms::amm::uniswap_v3::factory::UniswapV3Factory;
use amms::sync;
use eyre::Result;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Add rpc endpoint here:
    let rpc_endpoint = std::env::var("ETH_MAINNET_HTTP")?;
    let provider = Arc::new(ProviderBuilder::new().on_http(rpc_endpoint.parse()?));

    let factories = vec![
        // Add UniswapV3
        Factory::UniswapV3Factory(UniswapV3Factory::new(address!("1F98431c8aD98523631AE4a59f267346ea31F984"), 185)),
    ];

    // Sync pairs
    sync::sync_amms(factories, provider, None, 500).await?;

    Ok(())
}
