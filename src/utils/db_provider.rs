use reth_chainspec::ChainSpecBuilder;
use reth_db::mdbx::DatabaseArguments;
use reth_db::{open_db_read_only, ClientVersion, DatabaseEnv};
use reth_node_ethereum::EthereumNode;
use reth_node_types::NodeTypesWithDBAdapter;
use reth_provider::providers::StaticFileProvider;
use reth_provider::ProviderFactory;
use std::path::Path;
use std::sync::Arc;

pub fn init_db_read_only_from_env() -> eyre::Result<ProviderFactory<NodeTypesWithDBAdapter<EthereumNode, Arc<DatabaseEnv>>>> {
    let db_path = std::env::var("RETH_DB_PATH")?;
    let db_path = Path::new(&db_path);
    init_db_read_only(db_path)
}

pub fn init_db_read_only(db_path: &Path) -> eyre::Result<ProviderFactory<NodeTypesWithDBAdapter<EthereumNode, Arc<DatabaseEnv>>>> {
    let db = Arc::new(open_db_read_only(db_path.join("db").as_path(), DatabaseArguments::new(ClientVersion::default()))?);
    let spec = Arc::new(ChainSpecBuilder::mainnet().build());

    let factory = ProviderFactory::<NodeTypesWithDBAdapter<EthereumNode, Arc<DatabaseEnv>>>::new(
        db.clone(),
        spec.clone(),
        StaticFileProvider::read_only(db_path.join("static_files"), true)?,
    );

    Ok(factory)
}
