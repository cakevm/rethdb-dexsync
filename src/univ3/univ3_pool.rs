use alloy::primitives::aliases::{I24, U24};
use alloy::primitives::{Address, B256, U16, U160};
use eyre::eyre;
use reth_provider::StateProvider;

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
