use reth_direct_db_uniswap_storage::univ2::{UniV2Factory, UNI_V2_FACTORY};
use reth_direct_db_uniswap_storage::utils::init_db_from_env;
use std::time::Instant;

fn main() -> eyre::Result<()> {
    let factory = init_db_from_env()?;

    // Read all pairs from UniswapV2Factory
    let now = Instant::now();
    let univ2_factory = UniV2Factory::load_pairs(factory.latest()?, UNI_V2_FACTORY)?;
    println!("Loaded UniswapV2Factory in {:?} sec", now.elapsed());

    for (pair, reserve) in univ2_factory.pairs.iter().take(3) {
        println!("Pair: {:#?}, Reserve: {:#?}", pair, reserve);
    }
    println!("Total pairs: {}", univ2_factory.pairs.len());

    Ok(())
}
