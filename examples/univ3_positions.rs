use reth_chainspec::ChainSpecBuilder;
use reth_db::mdbx::DatabaseArguments;
use reth_db::{open_db_read_only, ClientVersion, DatabaseEnv};
use reth_direct_db_uniswap_storage::{UniV3PositionManager, UNI_V3_POSITION_MANAGER};
use reth_node_ethereum::EthereumNode;
use reth_node_types::NodeTypesWithDBAdapter;
use reth_provider::{providers::StaticFileProvider, ProviderFactory};
use std::path::Path;
use std::sync::Arc;

fn main() -> eyre::Result<()> {
    let db_path = std::env::var("RETH_DB_PATH")?;
    let db_path = Path::new(&db_path);

    let db = Arc::new(open_db_read_only(db_path.join("db").as_path(), DatabaseArguments::new(ClientVersion::default()))?);
    let spec = Arc::new(ChainSpecBuilder::mainnet().build());
    let factory = ProviderFactory::<NodeTypesWithDBAdapter<EthereumNode, Arc<DatabaseEnv>>>::new(
        db.clone(),
        spec.clone(),
        StaticFileProvider::read_only(db_path.join("static_files"), true)?,
    );

    // Read all positions from NonfungiblePositionManager
    let position_manager = UniV3PositionManager::load_pools(factory.latest()?, UNI_V3_POSITION_MANAGER)?;
    for pool in position_manager.pools.iter().take(3) {
        println!("Pool: {:#?}", pool);
    }
    println!("Total pools: {}", position_manager.pools.len());

    Ok(())
}
