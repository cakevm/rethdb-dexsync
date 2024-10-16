use crate::univ3::ticks::TICKS_SLOT;
use alloy_primitives::aliases::{I24, I56, U24};
use alloy_primitives::{b256, keccak256, Address, B256, I128, U128, U16, U160, U256};
use alloy_sol_types::SolValue;
use eyre::eyre;
use reth_provider::StateProvider;

const LIQUIDITY_SLOT: B256 = b256!("0000000000000000000000000000000000000000000000000000000000000004");

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

#[derive(Debug)]
pub struct TickInfo {
    pub liquidity_gross: U128,
    pub liquidity_net: I128,
    pub fee_growth_outside_0x128: U256,
    pub fee_growth_outside_1x128: U256,
    pub tick_cumulative_outside: I56,
    pub seconds_per_liquidity_outside_x128: U160,
    pub seconds_outside: u32,
    pub initialized: bool,
}

pub struct UniswapV3Pool;

impl UniswapV3Pool {
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

    pub fn read_tick<T: StateProvider>(provider: T, pool_address: Address, tick: I24) -> eyre::Result<Option<TickInfo>> {
        let storage_key0 = keccak256((tick, TICKS_SLOT).abi_encode());
        let storage_key1 = B256::from(U256::from_be_slice(storage_key0.0.as_slice()) + U256::from(1));
        let storage_key2 = B256::from(U256::from_be_slice(storage_key0.0.as_slice()) + U256::from(2));
        let storage_key3 = B256::from(U256::from_be_slice(storage_key0.0.as_slice()) + U256::from(3));

        let Some(storage_value0) = provider.storage(pool_address, storage_key0)? else {
            return Ok(None);
        };
        let Some(storage_value1) = provider.storage(pool_address, storage_key1)? else {
            return Err(eyre!("STORAGE_SLOT_1_NOT_FOUND"));
        };
        let Some(storage_value2) = provider.storage(pool_address, storage_key2)? else {
            return Err(eyre!("STORAGE_SLOT_2_NOT_FOUND"));
        };
        let Some(storage_value3) = provider.storage(pool_address, storage_key3)? else {
            return Err(eyre!("STORAGE_SLOT_3_NOT_FOUND"));
        };
        // slot0
        let bytes: [u8; 32] = storage_value0.to_be_bytes();
        let liquidity_net = I128::try_from_be_slice(&bytes[0..16]).unwrap();
        let liquidity_gross = U128::from_be_slice(&bytes[16..32]);
        // slot1
        let fee_growth_outside_0x128 = U256::from(storage_value1);
        // slot2
        let fee_growth_outside_1x128 = U256::from(storage_value2);
        // slot3
        let bytes: [u8; 32] = storage_value3.to_be_bytes();
        let initialized: bool = bytes[0] != 0;
        let seconds_outside: u32 = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
        let seconds_per_liquidity_outside_x128 = U160::from_be_slice(&bytes[5..25]);
        let tick_cumulative_outside = I56::try_from_be_slice(&bytes[25..32]).unwrap();

        Ok(Some(TickInfo {
            liquidity_gross,
            liquidity_net,
            fee_growth_outside_0x128,
            fee_growth_outside_1x128,
            tick_cumulative_outside,
            seconds_per_liquidity_outside_x128,
            seconds_outside,
            initialized,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{address, U256};
    use reth_primitives::{Account, StorageEntry};
    use reth_stages::test_utils::TestStageDB;
    use std::str::FromStr;

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

        let liquidity = UniswapV3Pool::read_liquidity(test_db.factory.latest()?, pool_weth_usdc)?;
        assert_eq!(liquidity, U128::from(183038598405746959u128));

        let liquidity = UniswapV3Pool::read_liquidity(test_db.factory.latest()?, pool_weth_mkr)?;
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

        let slot0 = UniswapV3Pool::read_slot0(test_db.factory.latest()?, pool_weth_usdc)?;
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

    #[test]
    pub fn test_read_tick() -> eyre::Result<()> {
        let test_db = TestStageDB::default();
        let pool_weth_usdc = address!("88e6a0c2ddd26feeb64f039a2c41296fcb3f5640");
        let pool_storage = (
            pool_weth_usdc,
            (
                Account::default(),
                vec![
                    StorageEntry::new(
                        b256!("ad66b8e7ab72f450ddfdaf1c5bc10e3a3fabf9f63ad8aa07b8743b93722f0a45"),
                        U256::from_be_slice(b256!("0000000000000000000000000000006400000000000000000000000000000064").as_slice()),
                    ),
                    StorageEntry::new(
                        b256!("ad66b8e7ab72f450ddfdaf1c5bc10e3a3fabf9f63ad8aa07b8743b93722f0a46"),
                        U256::from_be_slice(b256!("00000000000000000000000000000000000014c5771d23059d2718a51e7b0788").as_slice()),
                    ),
                    StorageEntry::new(
                        b256!("ad66b8e7ab72f450ddfdaf1c5bc10e3a3fabf9f63ad8aa07b8743b93722f0a47"),
                        U256::from_be_slice(b256!("000000000000000000000000000001ffe4d52ef99b7ca0682ff8350300adffbf").as_slice()),
                    ),
                    StorageEntry::new(
                        b256!("ad66b8e7ab72f450ddfdaf1c5bc10e3a3fabf9f63ad8aa07b8743b93722f0a48"),
                        U256::from_be_slice(b256!("0161058d820000000000000001e64843cf94d43eefe288859700015a733ba9cd").as_slice()),
                    ),
                ],
            ),
        );
        test_db.insert_accounts_and_storages(vec![pool_storage])?;

        let tick = UniswapV3Pool::read_tick(test_db.factory.latest()?, pool_weth_usdc, I24::try_from(100).unwrap())?.unwrap();
        assert_eq!(tick.liquidity_gross, U128::from(100));
        assert_eq!(tick.liquidity_net, I128::try_from(100)?);
        assert_eq!(tick.fee_growth_outside_0x128, U256::from(421293004028747376539989136050056u128));
        assert_eq!(tick.fee_growth_outside_1x128, U256::from_str("174188460391636158641944524878941652910015")?);
        assert_eq!(tick.tick_cumulative_outside, I56::try_from(1487991974349u64)?);
        assert_eq!(tick.seconds_per_liquidity_outside_x128, U160::from(150497077659940598037623637399u128));
        assert_eq!(tick.seconds_outside, 1627753858u32);
        assert_eq!(tick.initialized, true);

        Ok(())
    }
}
