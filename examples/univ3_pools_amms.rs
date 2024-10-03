use alloy::providers::ProviderBuilder;
use amms::amm::factory::Factory;
use amms::amm::uniswap_v3::factory::UniswapV3Factory;
use amms::sync;
use eyre::Result;
use rethdb_dexsync::univ3::UNI_V3_FACTORY;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let rpc_endpoint = std::env::var("ETH_MAINNET_HTTP")?;
    let provider = Arc::new(ProviderBuilder::new().on_http(rpc_endpoint.parse()?));

    let factories = vec![Factory::UniswapV3Factory(UniswapV3Factory::new(UNI_V3_FACTORY, 185))];
    sync::sync_amms(factories, provider, None, 500).await?;

    Ok(())
}
