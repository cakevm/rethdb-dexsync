use alloy::primitives::{Address, StorageValue, B256, U256};
use eyre::eyre;
use reth_db::cursor::DbCursorRO;
use reth_db::{tables, Database, DatabaseEnv};
use reth_primitives::StorageEntry;
use reth_provider::StateProvider;
use std::sync::Arc;

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

pub fn read_all_storage_entries(db_ref: Arc<DatabaseEnv>, address: Address) -> eyre::Result<Vec<StorageEntry>> {
    let tx = db_ref.tx()?;
    let mut cursor = tx.new_cursor::<tables::PlainStorageState>()?;
    let walker = cursor.walk(Some(address))?;

    let mut slots = vec![];
    for table_row_result in walker {
        let table_row = table_row_result?;
        if table_row.0 != address {
            break;
        }
        slots.push(table_row.1);
    }
    Ok(slots)
}
