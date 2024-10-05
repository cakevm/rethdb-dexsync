use reth_db::cursor::DbCursorRW;
use reth_db::tables;
use reth_db::transaction::DbTxMut;
use reth_provider::StateProviderFactory;
use rethdb_dexsync::test_utils::init_test_db_rw;
use rethdb_dexsync::univ3::{UniV3PositionManager, UNI_V3_FACTORY, UNI_V3_POSITION_MANAGER};
use rethdb_dexsync::utils::{init_db_read_only_from_env, read_all_storage_entries};
use std::path::{Path, PathBuf};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let factory = init_db_read_only_from_env()?;

    /*
        let db_ref = factory.db_ref();
        let slots = read_all_storage_entries(db_ref.clone(), address!("88e6a0c2ddd26feeb64f039a2c41296fcb3f5640"))?;

        println!("{:#?}", slots);
        println!("{:#?}", slots.len());
    */

    let univ3_pos_mng = UniV3PositionManager::load_pools(factory.latest()?, UNI_V3_FACTORY)?;

    let test_db_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata").join("univ3-test-db");
    create_path_if_not_exists(&test_db_path)?;
    create_path_if_not_exists(&test_db_path.join("db"))?;
    create_path_if_not_exists(&test_db_path.join("static_files"))?;

    let test_db_provider_factory = init_test_db_rw(&test_db_path)?;

    let mut test_db_provider = test_db_provider_factory.provider_rw()?;
    let tx_rw = test_db_provider.tx_mut();

    let mut cursor = tx_rw.cursor_dup_write::<tables::PlainStorageState>()?;

    let mut pool_addresses = vec![UNI_V3_FACTORY, UNI_V3_POSITION_MANAGER];
    for (pool, _, _) in univ3_pos_mng.pools {
        pool_addresses.push(pool.address);
    }
    // An inconsistency can happen if a new pair was added after reading the pools
    for address in pool_addresses {
        let db_ref = factory.db_ref();
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
