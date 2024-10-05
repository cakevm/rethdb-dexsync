use rethdb_dexsync::univ2::{UniV2Factory, UNI_V2_FACTORY};
use rethdb_dexsync::utils::init_db_read_only_from_env;
use std::path::Path;
use std::time::Instant;

fn main() -> eyre::Result<()> {
    let factory = init_db_read_only_from_env()?;

    // Read all pairs from UniswapV2Factory
    let now = Instant::now();
    let cache_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("cache");
    let univ2_factory = UniV2Factory::load_pairs(factory.latest()?, UNI_V2_FACTORY, Some(cache_path))?;
    println!("Loaded UniswapV2Factory in {:?} sec", now.elapsed());

    for (pair, reserve) in univ2_factory.pairs.iter().take(3) {
        println!("Pair: {:#?}, Reserve: {:#?}", pair, reserve);
    }
    println!("Total pairs: {}", univ2_factory.pairs.len());

    Ok(())
}
