use alloy::eips::BlockNumberOrTag;
use reth_db::cursor::DbCursorRW;
use reth_db::tables;
use reth_db::transaction::DbTxMut;
use rethdb_dexsync::test_utils::init_test_db_rw;
use rethdb_dexsync::univ2::{UniV2Factory, UNI_V2_FACTORY};
use rethdb_dexsync::utils::{init_db_read_only_from_env, read_all_storage_entries};
use std::path::{Path, PathBuf};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let provider_factory = init_db_read_only_from_env()?;
    let univ2_factory = UniV2Factory::load_pairs(&provider_factory, &BlockNumberOrTag::Latest, UNI_V2_FACTORY, None)?;

    let test_db_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata").join("univ2-test-db");
    create_path_if_not_exists(&test_db_path)?;
    create_path_if_not_exists(&test_db_path.join("db"))?;
    create_path_if_not_exists(&test_db_path.join("static_files"))?;

    let test_db_provider_factory = init_test_db_rw(&test_db_path)?;

    let mut test_db_provider = test_db_provider_factory.provider_rw()?;
    let tx_rw = test_db_provider.tx_mut();

    let mut cursor = tx_rw.cursor_dup_write::<tables::PlainStorageState>()?;

    let mut pair_addresses = vec![UNI_V2_FACTORY];
    for (pair, _) in univ2_factory.pairs {
        pair_addresses.push(pair.address);
    }
    // An inconsistency can happen if a new pair was added after reading the pools
    for address in pair_addresses {
        let db_ref = provider_factory.db_ref();
        let slots = read_all_storage_entries(db_ref.clone(), address)?;

        for slot in slots {
            cursor.upsert(address, slot)?
        }
    }
    test_db_provider.commit()?;

    Ok(())
}

fn create_path_if_not_exists(db_path: &PathBuf) -> eyre::Result<()> {
    if !db_path.exists() {
        std::fs::create_dir_all(db_path)?;
    }
    Ok(())
}
