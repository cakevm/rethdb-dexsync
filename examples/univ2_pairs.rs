use reth_chainspec::ChainSpecBuilder;
use reth_db::mdbx::DatabaseArguments;
use reth_db::{open_db_read_only, ClientVersion, DatabaseEnv};
use reth_direct_db_uniswap_storage::{UniV2Factory, UNI_V2_FACTORY};
use reth_node_ethereum::EthereumNode;
use reth_node_types::NodeTypesWithDBAdapter;
use reth_provider::{providers::StaticFileProvider, ProviderFactory};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

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
