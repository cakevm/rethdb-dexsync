use reth_provider::StateProviderFactory;
use rethdb_dexsync::univ3::{UniV3PositionManager, UNI_V3_POSITION_MANAGER};
use rethdb_dexsync::utils::init_db_read_only_from_env;

fn main() -> eyre::Result<()> {
    let factory = init_db_read_only_from_env()?;

    // Read all positions from PositionManager
    let position_manager = UniV3PositionManager::load_pools(factory.latest()?, UNI_V3_POSITION_MANAGER)?;
    for (pool, slot0) in position_manager.pools.iter().take(3) {
        println!("Pool: {:#?}", pool);
        println!("Slot0: {:#?}", slot0);
    }
    println!("Total pools: {}", position_manager.pools.len());

    Ok(())
}
