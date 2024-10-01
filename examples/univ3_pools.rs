use reth_direct_db_uniswap_storage::univ3::read_univ3_pools;
use reth_direct_db_uniswap_storage::utils::init_db_from_env;

fn main() -> eyre::Result<()> {
    let factory = init_db_from_env()?;
    let provider = factory.provider()?;

    // Read all pools from UniswapV3Factory
    let pools = read_univ3_pools(&provider)?;
    for pool in pools.iter().take(3) {
        println!("Pool: {:#?}", pool);
    }
    println!("Total pools: {}", pools.len());

    Ok(())
}
