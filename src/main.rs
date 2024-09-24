use reth_chainspec::ChainSpecBuilder;
use reth_node_ethereum::EthereumNode;
use reth_node_types::NodeTypesWithDBAdapter;
use reth_primitives::{Address, B256};
use reth_provider::{
    providers::StaticFileProvider, AccountReader, ProviderFactory, StateProvider
};
use std::path::Path;

fn main() -> eyre::Result<()> {
    // Opens a RO handle to the database file.
    let db_path = std::env::var("RETH_DB_PATH")?;
    let db_path = Path::new(&db_path);

    let spec = ChainSpecBuilder::mainnet().build();
    let factory =
        ProviderFactory::<NodeTypesWithDBAdapter<EthereumNode, _>>::new_with_database_path(
            db_path,
            spec.into(),
            Default::default(),
            StaticFileProvider::read_only(db_path.join("static_files"), false)?,
        )?;

    read_univ2_pairs(factory.latest()?)?;

    Ok(())
}

fn read_univ2_pairs<T: StateProvider + AccountReader>(provider: T) -> eyre::Result<Vec<Address>> {
    let address = Address::random();
    let storage_key = B256::random();

    // Can get account / storage state with simple point queries
    let _account = provider.basic_account(address)?;
    let _code = provider.account_code(address)?;
    let _storage = provider.storage(address, storage_key)?;


    Ok(vec![])
}
