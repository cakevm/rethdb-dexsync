use eyre::Result;
use pool_sync::{Chain, PoolInfo, PoolSync, PoolType};

#[tokio::main]
async fn main() -> Result<()> {
    // Configure and build the PoolSync instance
    let pool_sync = PoolSync::builder().add_pool(PoolType::UniswapV3).chain(Chain::Ethereum).build()?;

    // Synchronize pools
    let (pools, _last_synced_block) = pool_sync.sync_pools().await?;

    // Common Info
    for pool in pools.iter().take(3) {
        println!("Pool Address {:?}, Token 0: {:?}, Token 1: {:?}", pool.address(), pool.token0_address(), pool.token1_address());
    }

    println!("Synced {} pools!", pools.len());
    Ok(())
}
