use crate::cache::CachedPool;
use crate::PoolsCache;
use alloy::primitives::aliases::U112;
use alloy::primitives::{address, b256, keccak256, Address, B256, U160, U256, U32};
use alloy_sol_types::SolValue;
use eyre::eyre;
use lazy_static::lazy_static;
use reth_provider::StateProvider;
use tracing::debug;

const ALL_PAIRS_SLOT: B256 = b256!("0000000000000000000000000000000000000000000000000000000000000003");

const PAIR_TOKEN0: B256 = b256!("0000000000000000000000000000000000000000000000000000000000000006");
const PAIR_TOKEN1: B256 = b256!("0000000000000000000000000000000000000000000000000000000000000007");
const PAIR_RESERVE: B256 = b256!("0000000000000000000000000000000000000000000000000000000000000008");

pub const UNI_V2_FACTORY: Address = address!("5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f");

lazy_static! {
    static ref ALL_PAIRS_START_SLOT: B256 = keccak256(ALL_PAIRS_SLOT.abi_encode());
}

#[derive(Debug)]
pub struct UniV2Pair {
    pub idx: usize,
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
}

impl From<&CachedPool> for UniV2Pair {
    fn from(pool: &CachedPool) -> Self {
        Self { idx: pool.idx, address: Address::from(pool.address), token0: Address::from(pool.token0), token1: Address::from(pool.token1) }
    }
}

#[derive(Debug)]
pub struct UniV2PairReserve {
    pub block_timestamp_last: U32,
    pub reserve0: U112,
    pub reserve1: U112,
}

pub struct UniV2Factory {
    pub pairs: Vec<(UniV2Pair, UniV2PairReserve)>,
}

impl UniV2Factory {
    pub fn load_pairs<T: StateProvider>(provider: T, factory_address: Address) -> eyre::Result<Self> {
        // Read cached pairs if exists
        let mut pools_cache = match PoolsCache::load(factory_address) {
            Ok(pools_cache) => pools_cache,
            Err(e) => {
                debug!("Failed to load pools cache: {}", e);
                PoolsCache::new(factory_address)
            }
        };
        debug!("Loaded pools cache: {}", pools_cache.pools.len());
        // Convert cached pools to pairs
        let mut pairs = pools_cache.pools.iter().map(UniV2Pair::from).collect::<Vec<_>>();

        // Add new pairs since last cache write from pair index
        let start_idx = if !pairs.is_empty() { pairs.len() - 1 } else { 0 };
        println!("Loaded new pools: {}", start_idx);
        let (new_pairs, _) = read_univ2_pairs(&provider, factory_address, start_idx)?;
        pairs.extend(new_pairs);

        // populate reserves for pairs
        let reserves = read_univ2_pairs_reserves(&provider, pairs)?;

        pools_cache.pools = reserves.iter().map(|(pair, _)| CachedPool::from(pair)).collect();
        pools_cache.save()?;

        Ok(Self { pairs: reserves })
    }
}

/// Reads all Uniswap V2 pair from the factory contract. The result is not sorted.
pub fn read_univ2_pairs<T: StateProvider>(
    provider: T,
    factory_address: Address,
    start_idx: usize,
) -> eyre::Result<(Vec<UniV2Pair>, usize)> {
    let pairs_length = match provider.storage(factory_address, ALL_PAIRS_SLOT)? {
        None => return Err(eyre!("Invalid pair length")),
        Some(l) => l.to::<usize>(),
    };

    let chunk_size: usize = 5000;
    let mut pairs = Vec::new();

    // Reading in chunks to avoid long transaction error.
    for start in (start_idx..pairs_length).step_by(chunk_size) {
        let end = std::cmp::min(start + chunk_size, pairs_length);
        let pairs_chunk = read_pairs_interval(&provider, factory_address, start, end)?;
        pairs.extend(pairs_chunk);
    }

    Ok((pairs, pairs_length))
}

#[allow(dead_code)]
/// Read all univ2 pairs from the factory contract. The result is not sorted.
pub fn read_univ2_pairs_full<T: StateProvider>(
    provider: T,
    factory_address: Address,
    start_idx: usize,
) -> eyre::Result<Vec<(UniV2Pair, UniV2PairReserve)>> {
    let pairs_length = match provider.storage(factory_address, ALL_PAIRS_SLOT)? {
        None => return Err(eyre!("Invalid pair length")),
        Some(l) => l.to::<usize>(),
    };

    let chunk_size: usize = 5000;
    let mut pairs = Vec::new();

    // Reading in chunks to avoid long transaction error.
    for start in (start_idx..pairs_length).step_by(chunk_size) {
        let end = std::cmp::min(start + chunk_size, pairs_length);
        let pairs_chunk = read_pairs_full_interval(&provider, factory_address, start, end)?;
        pairs.extend(pairs_chunk);
    }

    Ok(pairs)
}

/// Read all univ2 reserves for provides pairs from the factory contract.
pub fn read_univ2_pairs_reserves<T: StateProvider>(provider: T, pairs: Vec<UniV2Pair>) -> eyre::Result<Vec<(UniV2Pair, UniV2PairReserve)>> {
    let mut pairs_with_reserves = Vec::new();

    for pair in pairs {
        let pair_reserves = read_pair_reserves(&provider, pair.address)?;
        pairs_with_reserves.push((pair, pair_reserves));
    }

    Ok(pairs_with_reserves)
}

fn read_pair_address<T: StateProvider>(provider: T, factory_address: Address, idx: usize) -> eyre::Result<Address> {
    match read_array_item_address(&provider, factory_address, *ALL_PAIRS_START_SLOT, idx)? {
        None => Err(eyre!("PAIR_NOT_FOUND")),
        Some(address) => Ok(address),
    }
}

fn read_pairs_interval<T: StateProvider>(provider: T, factory_address: Address, start: usize, end: usize) -> eyre::Result<Vec<UniV2Pair>> {
    let mut pairs = Vec::new();

    for i in start..end {
        let pair_address = read_pair_address(&provider, factory_address, i)?;
        let pair = read_pair(&provider, pair_address, i)?;
        pairs.push(pair);
    }

    Ok(pairs)
}

#[allow(dead_code)]
fn read_pairs_full_interval<T: StateProvider>(
    provider: T,
    factory_address: Address,
    start: usize,
    end: usize,
) -> eyre::Result<Vec<(UniV2Pair, UniV2PairReserve)>> {
    let mut pairs = Vec::new();

    for i in start..end {
        let pair_address = read_pair_address(&provider, factory_address, i)?;
        let pair = read_pair(&provider, pair_address, i)?;
        let pair_reserves = read_pair_reserves(&provider, pair_address)?;
        pairs.push((pair, pair_reserves));
    }

    Ok(pairs)
}

fn read_array_item_address<T: StateProvider>(
    provider: &T,
    contract_address: Address,
    slot: B256,
    idx: usize,
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

fn read_pair<T: StateProvider>(provider: T, pair_address: Address, idx: usize) -> eyre::Result<UniV2Pair> {
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
    Ok(UniV2Pair { idx, address: pair_address, token0, token1 })
}

fn read_pair_reserves<T: StateProvider>(provider: T, pair_address: Address) -> eyre::Result<UniV2PairReserve> {
    let (block_timestamp_last, reserve1, reserve0) = match provider.storage(pair_address, PAIR_RESERVE) {
        Ok(storage_value) => match storage_value {
            None => (U32::ZERO, U112::ZERO, U112::ZERO), // pair not initialized
            Some(value) => {
                let bytes = value.to_be_bytes_vec();
                let block_timestamp_last = U32::from_be_slice(&bytes[0..4]);
                let reserve1 = U112::from_be_slice(&bytes[4..18]);
                let reserve0 = U112::from_be_slice(&bytes[18..32]);
                (block_timestamp_last, reserve1, reserve0)
            }
        },
        Err(e) => return Err(eyre!(e)),
    };

    Ok(UniV2PairReserve { block_timestamp_last, reserve0, reserve1 })
}
