use alloy_primitives::aliases::U112;
use alloy_primitives::{b256, Address, B256, U160, U32};
use eyre::eyre;
use reth_provider::StateProvider;
use serde::{Deserialize, Serialize};

const PAIR_TOKEN0: B256 = b256!("0000000000000000000000000000000000000000000000000000000000000006");
const PAIR_TOKEN1: B256 = b256!("0000000000000000000000000000000000000000000000000000000000000007");
const PAIR_RESERVE: B256 = b256!("0000000000000000000000000000000000000000000000000000000000000008");

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UniV2Pair {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UniV2PairReserve {
    pub block_timestamp_last: u32,
    pub reserve0: U112,
    pub reserve1: U112,
}

pub fn read_pair<T: StateProvider>(provider: T, pair_address: Address) -> eyre::Result<UniV2Pair> {
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
    Ok(UniV2Pair { address: pair_address, token0, token1 })
}

pub fn read_pair_reserves<T: StateProvider>(provider: T, pair_address: Address) -> eyre::Result<UniV2PairReserve> {
    let (block_timestamp_last, reserve1, reserve0) = match provider.storage(pair_address, PAIR_RESERVE) {
        Ok(storage_value) => match storage_value {
            None => (0, U112::ZERO, U112::ZERO), // pair not initialized
            Some(value) => {
                let bytes = value.to_be_bytes_vec();
                let block_timestamp_last = U32::from_be_slice(&bytes[0..4]);
                let reserve1 = U112::from_be_slice(&bytes[4..18]);
                let reserve0 = U112::from_be_slice(&bytes[18..32]);
                (block_timestamp_last.to::<u32>(), reserve1, reserve0)
            }
        },
        Err(e) => return Err(eyre!(e)),
    };

    Ok(UniV2PairReserve { block_timestamp_last, reserve0, reserve1 })
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{address, U256};
    use reth_primitives::{Account, StorageEntry};
    use reth_stages::test_utils::TestStageDB;

    #[test]
    fn test_read_pair() -> eyre::Result<()> {
        let test_db = TestStageDB::default();

        let pair_address = address!("b4e16d0168e52d35cacd2c6185b44281ec28c9dc");
        let usdc_address = address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
        let weth_address = address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");

        let fist_pair = (
            pair_address,
            (
                Account::default(),
                vec![
                    StorageEntry::new(PAIR_TOKEN0, U256::from_be_slice(usdc_address.as_slice())),
                    StorageEntry::new(PAIR_TOKEN1, U256::from_be_slice(weth_address.as_slice())),
                ],
            ),
        );
        test_db.insert_accounts_and_storages(vec![fist_pair])?;

        let pair = read_pair(test_db.factory.latest()?, pair_address)?;
        assert_eq!(pair.token0, usdc_address);
        assert_eq!(pair.token1, weth_address);

        Ok(())
    }

    #[test]
    fn test_read_pair_reserves() -> eyre::Result<()> {
        let test_db = TestStageDB::default();

        let pair_address = address!("b4e16d0168e52d35cacd2c6185b44281ec28c9dc");

        let fist_pair = (
            pair_address,
            (
                Account::default(),
                vec![StorageEntry::new(
                    PAIR_RESERVE,
                    U256::from_be_slice(b256!("6700f0af0000000003c5512b85fc28d1721e0000000000000000272e698defb8").as_slice()),
                )],
            ),
        );
        test_db.insert_accounts_and_storages(vec![fist_pair])?;

        let reserves = read_pair_reserves(test_db.factory.latest()?, pair_address)?;
        assert_eq!(reserves.reserve0, U112::from(43080292888504u128));
        assert_eq!(reserves.reserve1, U112::from(17806956946988851098142u128));

        Ok(())
    }
}
