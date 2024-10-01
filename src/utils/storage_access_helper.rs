use alloy::primitives::{Address, StorageValue, B256, U256};
use eyre::eyre;
use reth_provider::StateProvider;

pub fn read_array_item<T: StateProvider>(
    provider: &T,
    contract_address: Address,
    slot: B256,
    idx: usize,
) -> eyre::Result<Option<StorageValue>> {
    let storage_key = B256::from(U256::from_be_slice(slot.as_slice()) + U256::from(idx));

    match provider.storage(contract_address, storage_key) {
        Ok(storage_value) => match storage_value {
            None => Ok(None),
            Some(storage_value) => Ok(Some(storage_value)),
        },
        Err(e) => Err(eyre!(e)),
    }
}
