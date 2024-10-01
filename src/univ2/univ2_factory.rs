use crate::univ2::univ2_pair::UniV2Pair;
use crate::univ2::{univ2_pair, UniV2PairReserve};
use crate::utils::read_array_item;
use crate::utils::{CachedPool, PoolsCache};
use alloy::primitives::{address, b256, keccak256, Address, B256, U160};
use alloy_sol_types::SolValue;
use eyre::eyre;
use lazy_static::lazy_static;
use reth_provider::StateProvider;
use tracing::debug;

const ALL_PAIRS_SLOT: B256 = b256!("0000000000000000000000000000000000000000000000000000000000000003");

pub const UNI_V2_FACTORY: Address = address!("5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f");

lazy_static! {
    static ref ALL_PAIRS_START_SLOT: B256 = keccak256(ALL_PAIRS_SLOT.abi_encode());
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
        debug!("Loaded new pools: {}", start_idx);
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

    let chunk_size: usize = 1000;
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
        let pair_reserves = univ2_pair::read_pair_reserves(&provider, pair.address)?;
        pairs_with_reserves.push((pair, pair_reserves));
    }

    Ok(pairs_with_reserves)
}

fn read_pair_address<T: StateProvider>(provider: T, factory_address: Address, idx: usize) -> eyre::Result<Address> {
    match read_array_item(&provider, factory_address, *ALL_PAIRS_START_SLOT, idx)? {
        None => Err(eyre!("PAIR_NOT_FOUND")),
        Some(storage_value) => Ok(Address::from(U160::from(storage_value))),
    }
}

fn read_pairs_interval<T: StateProvider>(provider: T, factory_address: Address, start: usize, end: usize) -> eyre::Result<Vec<UniV2Pair>> {
    let mut pairs = Vec::new();

    for i in start..end {
        let pair_address = read_pair_address(&provider, factory_address, i)?;
        let pair = univ2_pair::read_pair(&provider, pair_address, i)?;
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
        let pair = univ2_pair::read_pair(&provider, pair_address, i)?;
        let pair_reserves = univ2_pair::read_pair_reserves(&provider, pair_address)?;
        pairs.push((pair, pair_reserves));
    }

    Ok(pairs)
}
