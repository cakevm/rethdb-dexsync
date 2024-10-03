use alloy::providers::ProviderBuilder;
use amms::amm::factory::Factory;
use amms::amm::uniswap_v2::factory::UniswapV2Factory;
use amms::sync;
use eyre::Result;
use rethdb_dexsync::univ2::UNI_V2_FACTORY;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let rpc_endpoint = std::env::var("ETH_MAINNET_HTTP")?;
    let provider = Arc::new(ProviderBuilder::new().on_http(rpc_endpoint.parse()?));

    let factories = vec![Factory::UniswapV2Factory(UniswapV2Factory::new(UNI_V2_FACTORY, 2638438, 300))];

    sync::sync_amms(factories, provider, None, 500).await?;

    Ok(())
}
