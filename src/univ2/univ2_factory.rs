use crate::univ2::univ2_pair::UniV2Pair;
use crate::univ2::{univ2_pair, UniV2PairReserve};
use crate::utils::{read_array_item, state_provider, CacheError, DexSyncCache};
use alloy::eips::BlockNumberOrTag;
use alloy::primitives::{b256, keccak256, Address, B256, U160};
use alloy_sol_types::SolValue;
use eyre::eyre;
use lazy_static::lazy_static;
use reth_provider::{StateProvider, StateProviderFactory};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::debug;

const ALL_PAIRS_SLOT: B256 = b256!("0000000000000000000000000000000000000000000000000000000000000003");

lazy_static! {
    static ref ALL_PAIRS_START_SLOT: B256 = keccak256(ALL_PAIRS_SLOT.abi_encode());
}

// Smart caching all pairs with address, token0 and token1. Only new pairs will be loaded.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UniV2FactoryCache {
    pub pairs: Vec<UniV2Pair>,
}

impl UniV2FactoryCache {
    pub fn new() -> Self {
        Self { pairs: Vec::new() }
    }
}

#[derive(Debug, Default)]
pub struct PoolFilter {
    // Exclusive filter all pairs with block timestamp after this value.
    block_timestamp_after: u32,
}

impl PoolFilter {
    pub fn new() -> Self {
        Self { block_timestamp_after: 0 }
    }
    pub fn block_timestamp_after(&mut self, block_timestamp_after: u32) -> &mut Self {
        self.block_timestamp_after = block_timestamp_after;
        self
    }
}

#[derive(Debug, Default)]
pub struct UniV2Factory {
    pub pairs: Vec<(UniV2Pair, UniV2PairReserve)>,
}

impl UniV2Factory {
    pub fn load_pairs<P: StateProviderFactory>(
        provider_factory: &P,
        block_number_or_tag: &BlockNumberOrTag,
        factory_address: Address,
        filter: &PoolFilter,
        cache_path: Option<PathBuf>,
    ) -> eyre::Result<Self> {
        let cached = Self::read_cached_pairs_if_exists(&cache_path, factory_address)?;
        // Convert cached pools to pairs
        let mut pairs = cached.pairs;

        // Add new pairs since last cache write from pair index
        let start_idx = if !pairs.is_empty() { pairs.len() - 1 } else { 0 };
        debug!("Loaded new pools: {}", start_idx);
        let (new_pairs, _) = read_univ2_pairs(provider_factory, block_number_or_tag, factory_address, start_idx)?;
        pairs.extend(new_pairs);

        // populate reserves for pairs
        let pairs_and_reserves = read_univ2_pairs_reserves(provider_factory, block_number_or_tag, pairs, filter)?;

        if cache_path.is_some() {
            let pairs = pairs_and_reserves.iter().map(|(pair, _)| pair.clone()).collect();
            let cache = UniV2FactoryCache { pairs };
            DexSyncCache::save(&cache_path.unwrap(), factory_address, cache)?;
        }

        Ok(Self { pairs: pairs_and_reserves })
    }

    fn read_cached_pairs_if_exists(cache_path: &Option<PathBuf>, factory_address: Address) -> eyre::Result<UniV2FactoryCache> {
        let factory = match &cache_path {
            Some(cache_path) => {
                let factory = match DexSyncCache::load::<UniV2FactoryCache>(cache_path, factory_address) {
                    Ok(univ2_factory) => univ2_factory,
                    Err(cache_error) => match cache_error {
                        CacheError::Io(e) => return Err(eyre!(e)),
                        CacheError::Bincode(e) => return Err(eyre!(e)),
                        CacheError::FileNotFound => UniV2FactoryCache::new(),
                    },
                };
                debug!("Loaded pools cache: {}", factory.pairs.len());
                factory
            }
            None => UniV2FactoryCache::new(),
        };
        Ok(factory)
    }
}

/// Reads all Uniswap V2 pair from the factory contract. The result is not sorted.
pub fn read_univ2_pairs<P: StateProviderFactory>(
    provider_factory: &P,
    block_number_or_tag: &BlockNumberOrTag,
    factory_address: Address,
    start_idx: usize,
) -> eyre::Result<(Vec<UniV2Pair>, usize)> {
    let provider = state_provider(provider_factory, block_number_or_tag)?;
    let pairs_length = match provider.storage(factory_address, ALL_PAIRS_SLOT)? {
        None => return Err(eyre!("Invalid pair length")),
        Some(l) => l.to::<usize>(),
    };

    let chunk_size: usize = 1000;
    let mut pairs = Vec::new();

    // Reading in chunks to avoid long transaction error.
    for start in (start_idx..pairs_length).step_by(chunk_size) {
        let end = std::cmp::min(start + chunk_size, pairs_length);
        // To avoid long-running transactions we create a now provider for each chunk.
        let provider = state_provider(provider_factory, block_number_or_tag)?;
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
pub fn read_univ2_pairs_reserves<P: StateProviderFactory>(
    provider_factory: &P,
    block_number_or_tag: &BlockNumberOrTag,
    pairs: Vec<UniV2Pair>,
    filter: &PoolFilter,
) -> eyre::Result<Vec<(UniV2Pair, UniV2PairReserve)>> {
    let mut pairs_with_reserves = Vec::new();

    let provider = state_provider(provider_factory, block_number_or_tag)?;
    for pair in pairs {
        let pair_reserves = univ2_pair::read_pair_reserves(&provider, pair.address)?;
        if pair_reserves.block_timestamp_last <= filter.block_timestamp_after {
            continue;
        }
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

    for idx in start..end {
        let pair_address = read_pair_address(&provider, factory_address, idx)?;
        let pair = univ2_pair::read_pair(&provider, pair_address)?;
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

    for idx in start..end {
        let pair_address = read_pair_address(&provider, factory_address, idx)?;
        let pair = univ2_pair::read_pair(&provider, pair_address)?;
        let pair_reserves = univ2_pair::read_pair_reserves(&provider, pair_address)?;
        pairs.push((pair, pair_reserves));
    }

    Ok(pairs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::univ2::UNI_V2_FACTORY;
    use alloy::primitives::{address, U256};
    use reth_primitives::{Account, StorageEntry};
    use reth_stages::test_utils::TestStageDB;

    #[test]
    fn test_read_pair_address() -> eyre::Result<()> {
        let test_db = TestStageDB::default();

        let pair_address_0 = address!("b4e16d0168e52d35cacd2c6185b44281ec28c9dc");
        let pair_address_18 = address!("5d27df1a6e03254e4f1218607d8e073667ffae2f");
        let fist_pair = (
            UNI_V2_FACTORY,
            (
                Account::default(),
                vec![
                    StorageEntry::new(
                        b256!("c2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85b"),
                        U256::from_be_slice(pair_address_0.as_slice()),
                    ),
                    StorageEntry::new(
                        b256!("c2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f86d"),
                        U256::from_be_slice(pair_address_18.as_slice()),
                    ),
                ],
            ),
        );
        test_db.insert_accounts_and_storages(vec![fist_pair])?;

        let pair_address = read_pair_address(test_db.factory.latest()?, UNI_V2_FACTORY, 0)?;
        assert_eq!(pair_address, pair_address_0);

        let pair_address = read_pair_address(test_db.factory.latest()?, UNI_V2_FACTORY, 18)?;
        assert_eq!(pair_address, pair_address_18);
        Ok(())
    }
}
