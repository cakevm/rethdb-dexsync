use reth_chainspec::ChainSpecBuilder;
use reth_db::mdbx::DatabaseArguments;
use reth_db::{init_db, ClientVersion, DatabaseEnv};
use reth_node_ethereum::EthereumNode;
use reth_node_types::NodeTypesWithDBAdapter;
use reth_provider::providers::StaticFileProvider;
use reth_provider::ProviderFactory;
use std::path::Path;
use std::sync::Arc;

pub fn init_test_db_rw(db_path: &Path) -> eyre::Result<ProviderFactory<NodeTypesWithDBAdapter<EthereumNode, Arc<DatabaseEnv>>>> {
    let db = Arc::new(init_db(db_path.join("db").as_path(), DatabaseArguments::new(ClientVersion::default()))?);
    let spec = Arc::new(ChainSpecBuilder::mainnet().build());

    let factory = ProviderFactory::<NodeTypesWithDBAdapter<EthereumNode, Arc<DatabaseEnv>>>::new(
        db.clone(),
        spec.clone(),
        StaticFileProvider::read_write(db_path.join("static_files"))?,
    );

    Ok(factory)
}
