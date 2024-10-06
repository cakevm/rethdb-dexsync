use alloy::primitives::aliases::{I24, U24};
use alloy::primitives::{b256, Address, B256, U128, U16, U160};
use eyre::eyre;
use reth_provider::StateProvider;

const LIQUIDITY_SLOT: B256 = b256!("0000000000000000000000000000000000000000000000000000000000000004");

#[allow(dead_code)]
#[derive(Debug)]
pub struct Univ3Pool {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub fee: U24,
}

#[derive(Debug)]
pub struct Univ3Slot0 {
    pub sqrt_price_x96: U160,
    pub tick: I24,
    pub observation_index: U16,
    pub observation_cardinality: U16,
    pub observation_cardinality_next: U16,
    pub fee_protocol: u8,
    pub unlocked: bool,
}

pub fn read_liquidity<T: StateProvider>(provider: T, pool_address: Address) -> eyre::Result<U128> {
    match provider.storage(pool_address, LIQUIDITY_SLOT) {
        Ok(storage_value) => match storage_value {
            None => Ok(U128::ZERO), // the pool has no liquidity
            Some(value) => {
                let bytes: [u8; 32] = value.to_be_bytes();
                Ok(U128::from_be_slice(&bytes[16..32]))
            }
        },
        Err(e) => Err(eyre!(e)),
    }
}

pub fn read_slot0<T: StateProvider>(provider: T, pool_address: Address) -> eyre::Result<Option<Univ3Slot0>> {
    match provider.storage(pool_address, B256::ZERO) {
        Ok(storage_value) => match storage_value {
            None => Ok(None), // pool not found
            Some(value) => {
                let bytes: [u8; 32] = value.to_be_bytes();
                let unlocked = bytes[1] != 0;
                let fee_protocol = bytes[2];
                let observation_cardinality_next = U16::from_be_slice(&bytes[3..5]);
                let observation_cardinality = U16::from_be_slice(&bytes[5..7]);
                let observation_index = U16::from_be_slice(&bytes[7..9]);
                let tick = I24::try_from_be_slice(&bytes[9..12]).unwrap();
                let sqrt_price_x96 = U160::from_be_slice(&bytes[12..32]);
                Ok(Some(Univ3Slot0 {
                    unlocked,
                    fee_protocol,
                    observation_cardinality_next,
                    observation_cardinality,
                    observation_index,
                    tick,
                    sqrt_price_x96,
                }))
            }
        },
        Err(e) => Err(eyre!(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{address, U256};
    use reth_primitives::{Account, StorageEntry};
    use reth_stages::test_utils::TestStageDB;

    #[test]
    fn test_read_liquidity() -> eyre::Result<()> {
        let test_db = TestStageDB::default();

        let pool_weth_usdc = address!("88e6a0c2ddd26feeb64f039a2c41296fcb3f5640");
        let pool_weth_mkr = address!("886072a44bdd944495eff38ace8ce75c1eacdaf6"); // no liquidity
        let pool_storages = vec![
            (
                pool_weth_usdc,
                (
                    Account::default(),
                    vec![StorageEntry::new(
                        LIQUIDITY_SLOT,
                        U256::from_be_slice(b256!("000000000000000000000000000000000000000000000000028a48a2ae28f10f").as_slice()),
                    )],
                ),
            ),
            (pool_weth_mkr, (Account::default(), vec![StorageEntry::new(LIQUIDITY_SLOT, U256::ZERO)])),
        ];
        test_db.insert_accounts_and_storages(pool_storages)?;

        let liquidity = read_liquidity(test_db.factory.latest()?, pool_weth_usdc)?;
        assert_eq!(liquidity, U128::from(183038598405746959u128));

        let liquidity = read_liquidity(test_db.factory.latest()?, pool_weth_mkr)?;
        assert_eq!(liquidity, U128::ZERO);

        Ok(())
    }

    #[test]
    fn test_read_slot0() -> eyre::Result<()> {
        let test_db = TestStageDB::default();

        let pool_weth_usdc = address!("88e6a0c2ddd26feeb64f039a2c41296fcb3f5640");
        let pool_storage = (
            pool_weth_usdc,
            (
                Account::default(),
                vec![StorageEntry::new(
                    B256::ZERO,
                    U256::from_be_slice(b256!("00010002d302d301800307320000000000004f96a4fc64ac43f93680a947bbda").as_slice()),
                )],
            ),
        );
        test_db.insert_accounts_and_storages(vec![pool_storage])?;

        let slot0 = read_slot0(test_db.factory.latest()?, pool_weth_usdc)?;
        assert!(slot0.is_some());
        let slot0 = slot0.unwrap();
        assert_eq!(slot0.sqrt_price_x96, U160::from(1614245643731953243882325864332250u128));
        assert_eq!(slot0.tick, I24::from_dec_str("198450")?);
        assert_eq!(slot0.observation_index, U16::from(384));
        assert_eq!(slot0.observation_cardinality, U16::from(723));
        assert_eq!(slot0.observation_cardinality_next, U16::from(723));
        assert_eq!(slot0.fee_protocol, 0);
        assert_eq!(slot0.unlocked, true);
        Ok(())
    }
}
