use alloy::primitives::aliases::{U176, U24, U80};
use alloy::primitives::{address, b256, keccak256, Address, B256, U256};
use alloy_sol_types::SolValue;
use eyre::eyre;
use reth_provider::StateProvider;

const POOL_INIT_CODE_HASH: B256 = b256!("e34f199b19b2b4f47f68442619d555527d244f78a3297ea89325f843f87b8b54");
const NEXT_POOL_ID: B256 = b256!("000000000000000000000000000000000000000000000000000000000000000d");
const POOL_ID_TO_POOL_KEY: B256 = b256!("000000000000000000000000000000000000000000000000000000000000000b");
pub const UNI_V3_POSITION_MANAGER: Address = address!("c36442b4a4522e871399cd717abdd847ab11fe88");

#[allow(dead_code)]
#[derive(Debug)]
pub struct Univ3Pool {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub fee: U24,
}

#[derive(Debug)]
pub struct PoolKey {
    token0: Address,
    token1: Address,
    fee: U24,
}

pub struct UniV3PositionManager {
    pub pools: Vec<Univ3Pool>,
}

impl UniV3PositionManager {
    pub fn load_pools<T: StateProvider>(provider: T, univ3_position_mng: Address) -> eyre::Result<Self> {
        let pools = read_univ3_position_pools(provider, univ3_position_mng)?;
        Ok(UniV3PositionManager { pools })
    }
}

pub fn read_univ3_position_pools<T: StateProvider>(provider: T, univ3_position_mng: Address) -> eyre::Result<Vec<Univ3Pool>> {
    let univ3_factory = address!("1F98431c8aD98523631AE4a59f267346ea31F984");

    let (next_pool_id, next_position_id) = match provider.storage(univ3_position_mng, NEXT_POOL_ID)? {
        None => return Err(eyre!("Invalid pair length")),
        Some(value) => {
            let bytes = value.to_be_bytes_vec();
            let next_pool_id = U80::from_be_slice(&bytes[0..10]);
            let next_position_id = U176::from_be_slice(&bytes[10..32]);
            (next_pool_id, next_position_id)
        }
    };
    println!("Next pool id: {}, Next position id: {}", next_pool_id, next_position_id);

    let mut pool_addresses = vec![];

    for pool_id in 1..next_pool_id.to::<u64>() {
        // mapping(uint80 => PoolAddress.PoolKey)
        let storage_key0 = keccak256((U80::from(pool_id), POOL_ID_TO_POOL_KEY).abi_encode());
        let storage_key1 = B256::from(U256::from_be_slice(storage_key0.0.as_slice()) + U256::from(1));

        let pool_key = match provider.storage(univ3_position_mng, storage_key0)? {
            None => return Err(eyre!("Invalid pool id")),
            Some(value) => {
                let bytes = value.to_be_bytes_vec();
                let token0 = Address::from_slice(&bytes[12..32]);

                // read second slot
                let (fee, token1) = match provider.storage(univ3_position_mng, storage_key1)? {
                    None => return Err(eyre!("Invalid pool id second slot")),
                    Some(value) => {
                        let bytes = value.to_be_bytes_vec();
                        let fee = U24::from_be_slice(&bytes[9..12]);
                        let token1 = Address::from_slice(&bytes[12..32]);
                        (fee, token1)
                    }
                };

                PoolKey { token0, token1, fee }
            }
        };

        let pool_address = compute_address(univ3_factory, &pool_key)?;
        pool_addresses.push(Univ3Pool { address: pool_address, token0: pool_key.token0, token1: pool_key.token1, fee: pool_key.fee });
    }

    Ok(pool_addresses)
}

pub fn compute_address(factory: Address, key: &PoolKey) -> eyre::Result<Address> {
    if key.token0 >= key.token1 {
        return Err(eyre!("token0 must be less than token1"));
    }
    let inner_hash = keccak256((key.token0, key.token1, key.fee).abi_encode());
    let pool_hash = keccak256((b"\xff", factory, inner_hash, POOL_INIT_CODE_HASH).abi_encode_packed());
    let pool_address = Address::from_slice(&pool_hash.as_slice()[12..32]);

    Ok(pool_address)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;

    #[test]
    fn test_compute_address() {
        let factory = address!("1F98431c8aD98523631AE4a59f267346ea31F984");
        let key = PoolKey {
            token0: address!("1f9840a85d5af5bf1d1762f925bdaddc4201f984"),
            token1: address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"),
            fee: U24::from(3000),
        };

        let expected_address = address!("1d42064Fc4Beb5F8aAF85F4617AE8b3b5B8Bd801");

        match compute_address(factory, &key) {
            Ok(pool_address) => assert_eq!(pool_address, expected_address),
            Err(e) => panic!("Failed to compute address: {:?}", e),
        }
    }
}
