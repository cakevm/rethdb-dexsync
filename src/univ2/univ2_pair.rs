use crate::utils::CachedPool;
use alloy::primitives::aliases::U112;
use alloy::primitives::{b256, Address, B256, U160, U32};
use eyre::eyre;
use reth_provider::StateProvider;

const PAIR_TOKEN0: B256 = b256!("0000000000000000000000000000000000000000000000000000000000000006");
const PAIR_TOKEN1: B256 = b256!("0000000000000000000000000000000000000000000000000000000000000007");
const PAIR_RESERVE: B256 = b256!("0000000000000000000000000000000000000000000000000000000000000008");

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

pub fn read_pair<T: StateProvider>(provider: T, pair_address: Address, idx: usize) -> eyre::Result<UniV2Pair> {
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

pub fn read_pair_reserves<T: StateProvider>(provider: T, pair_address: Address) -> eyre::Result<UniV2PairReserve> {
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
