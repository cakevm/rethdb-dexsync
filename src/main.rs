use alloy::primitives::{address, b256, U32};
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

const PAIR_TOKEN0: B256 = b256!("0000000000000000000000000000000000000000000000000000000000000006");
const PAIR_TOKEN1: B256 = b256!("0000000000000000000000000000000000000000000000000000000000000007");
const PAIR_RESERVE: B256 = b256!("0000000000000000000000000000000000000000000000000000000000000008");

#[allow(dead_code)]
#[derive(Debug)]
struct Univ2Pair {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub block_timestamp_last: U32,
    pub reserve0: U160,
    pub reserve1: U160,
}

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

fn read_univ2_pairs<T: StateProvider + AccountReader>(provider: T) -> eyre::Result<Vec<Univ2Pair>> {
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
            let pair = read_pair(&provider, pair_address)?;
            pairs.push(pair);
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

fn read_pair<T: StateProvider + AccountReader>(provider: T, pair_address: Address) -> eyre::Result<Univ2Pair> {
    let token0 = match provider.storage(pair_address, PAIR_TOKEN0) {
        Ok(storage_value) => match storage_value {
            None => return Err(eyre!("STORAGE_SLOT_NOT_FOUND token0, {:#?}", pair_address)),
            Some(value) => Address::from(U160::from(value)),
        },
        Err(e) => return Err(eyre!(e)),
    };

    let token1 = match provider.storage(pair_address, PAIR_TOKEN1) {
        Ok(storage_value) => match storage_value {
            None => return Err(eyre!("STORAGE_SLOT_NOT_FOUND token1, {:#?}", pair_address)),
            Some(value) => Address::from(U160::from(value)),
        },
        Err(e) => return Err(eyre!(e)),
    };

    let (block_timestamp_last, reserve1, reserve0) = match provider.storage(pair_address, PAIR_RESERVE) {
        Ok(storage_value) => match storage_value {
            None => (U32::ZERO, U160::ZERO, U160::ZERO), // pair not initialized
            Some(value) => {
                let bytes = value.to_be_bytes_vec();
                let block_timestamp_last = U32::from_be_slice(&bytes[0..4]);
                let reserve1 = U160::from_be_slice(&bytes[4..18]);
                let reserve0 = U160::from_be_slice(&bytes[18..32]);
                (block_timestamp_last, reserve1, reserve0)
            }
        },
        Err(e) => return Err(eyre!(e)),
    };

    Ok(Univ2Pair { address: pair_address, token0, token1, block_timestamp_last, reserve0, reserve1 })
}
