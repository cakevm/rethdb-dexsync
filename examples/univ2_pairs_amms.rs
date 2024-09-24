use std::sync::Arc;
use alloy::primitives::address;
use alloy::providers::ProviderBuilder;
use amms::amm::factory::Factory;
use amms::amm::uniswap_v2::factory::UniswapV2Factory;
use amms::sync;
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Add rpc endpoint here:
    let rpc_endpoint = std::env::var("ETH_MAINNET_HTTP")?;
    let provider = Arc::new(ProviderBuilder::new().on_http(rpc_endpoint.parse()?));

    let factories = vec![
        // Add UniswapV2
        Factory::UniswapV2Factory(UniswapV2Factory::new(
            address!("5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"),
            2638438,
            300,
        )),
    ];

    // Sync pairs
    sync::sync_amms(factories, provider, None, 500).await?;

    Ok(())
}
