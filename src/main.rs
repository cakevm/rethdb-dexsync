use alloy::primitives::{address, b256};
use alloy_sol_types::private::primitives::U160;
use alloy_sol_types::SolValue;
use eyre::eyre;
use reth_chainspec::ChainSpecBuilder;
use reth_node_ethereum::EthereumNode;
use reth_node_types::NodeTypesWithDBAdapter;
use reth_primitives::{keccak256, Address, B256, U256};
use reth_provider::{providers::StaticFileProvider, AccountReader, ProviderFactory, StateProvider};
use std::path::Path;

const ALL_PAIRS_SLOT: B256 = b256!("0000000000000000000000000000000000000000000000000000000000000003");

fn main() -> eyre::Result<()> {
    let db_path = std::env::var("RETH_DB_PATH")?;
    let db_path = Path::new(&db_path);

    let spec = ChainSpecBuilder::mainnet().build();
    let factory = ProviderFactory::<NodeTypesWithDBAdapter<EthereumNode, _>>::new_with_database_path(
        db_path,
        spec.into(),
        Default::default(),
        StaticFileProvider::read_only(db_path.join("static_files"), false)?,
    )?;

    // Read all pairs from UniswapV2Factory
    let pairs = read_univ2_pairs(factory.latest()?)?;
    for pair in pairs.iter().take(3) {
        println!("Pair: {:#?}", pair);
    }
    println!("Total pairs: {}", pairs.len());

    Ok(())
}

fn read_univ2_pairs<T: StateProvider + AccountReader>(provider: T) -> eyre::Result<Vec<Address>> {
    let univ2_factory = address!("5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f");
    let all_pairs_start = keccak256(ALL_PAIRS_SLOT.abi_encode());
    let pairs_length = match provider.storage(univ2_factory, ALL_PAIRS_SLOT)? {
        None => {
            return Err(eyre!("Invalid pair length"));
        }
        Some(l) => {
            println!("{:#?}", l);
            l.to::<u128>()
        }
    };

    let mut pairs = Vec::with_capacity(pairs_length as usize);
    for i in 0..pairs_length {
        if let Some(pair_address) = read_array_item_address(&provider, univ2_factory, all_pairs_start, i)? {
            pairs.push(pair_address);
        }
    }

    Ok(pairs)
}

fn read_array_item_address<T: StateProvider + AccountReader>(
    provider: T,
    contract_address: Address,
    slot: B256,
    idx: u128,
) -> eyre::Result<Option<Address>> {
    let storage_key = B256::from(U256::from_be_slice(slot.as_slice()) + U256::from(idx));

    match provider.storage(contract_address, storage_key) {
        Ok(storage_value) => match storage_value {
            None => Ok(None),
            Some(value) => Ok(Some(Address::from(U160::from(value)))),
        },
        Err(e) => Err(eyre!(e)),
    }
}
