use crate::univ2::UniV2Pair;
use alloy::primitives::Address;
use rkyv::rancor::Error;
use rkyv::{deserialize, rancor, Archive, Deserialize, Serialize};
use std::io::Write;
use std::path::Path;
use std::{env, fs};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Rkyv error: {0}")]
    Rkyv(#[from] rancor::Error),
}

#[derive(Archive, Deserialize, Serialize, PartialEq)]
pub struct CachedPool {
    pub idx: usize,
    pub address: [u8; 20],
    pub token0: [u8; 20],
    pub token1: [u8; 20],
}

impl From<&UniV2Pair> for CachedPool {
    fn from(pair: &UniV2Pair) -> Self {
        Self { idx: pair.idx, address: pair.address.0 .0, token0: pair.token0.0 .0, token1: pair.token1.0 .0 }
    }
}

#[derive(Archive, Deserialize, Serialize, PartialEq)]
pub struct PoolsCache {
    pub factory_address: [u8; 20],
    // factory address -> pool address
    pub pools: Vec<CachedPool>,
}

impl PoolsCache {
    pub fn new(factory_address: Address) -> Self {
        Self { factory_address: factory_address.0 .0, pools: Vec::new() }
    }

    pub fn save(&self) -> eyre::Result<()> {
        let bytes = rkyv::to_bytes::<Error>(self)?;

        let path = env::current_dir()?.join("../../cache");
        if !Path::new(&path).exists() {
            fs::create_dir_all(&path)?;
        }
        let mut file = fs::File::create(path.join(format!("factory_{:#?}.rkyv", Address::from(self.factory_address))))?;
        file.write_all(bytes.as_slice())?;
        Ok(())
    }

    pub fn load(factory_address: Address) -> Result<PoolsCache, CacheError> {
        let path = env::current_dir()?.join("../../cache");
        let file_path = path.join(format!("factory_{:#?}.rkyv", factory_address));
        if !file_path.exists() {
            return Ok(PoolsCache::new(factory_address));
        }
        let bytes = fs::read(file_path)?;
        let archived = rkyv::access::<ArchivedPoolsCache, Error>(bytes.as_slice())?;
        let deserialized = deserialize::<PoolsCache, Error>(archived)?;
        Ok(deserialized)
    }
}
