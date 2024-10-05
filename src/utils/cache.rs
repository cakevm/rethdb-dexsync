use alloy::primitives::Address;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("File not found")]
    FileNotFound,
    #[error("Bincode error: {0}")]
    Bincode(#[from] bincode::Error),
}

pub struct DexSyncCache {}

impl DexSyncCache {
    pub fn save<T: Serialize>(path: &Path, address: Address, data: T) -> eyre::Result<()> {
        if !Path::new(&path).exists() {
            fs::create_dir_all(path)?;
        }
        let mut file = fs::File::create(path.join(format!("factory_{:#?}.bincode", address)))?;
        let encoded: Vec<u8> = bincode::serialize(&data)?;
        file.write_all(&encoded)?;
        Ok(())
    }

    pub fn load<T>(path: &Path, address: Address) -> Result<T, CacheError>
    where
        T: DeserializeOwned,
    {
        let file_path = path.join(format!("factory_{:#?}.bincode", address));
        if !path.exists() {
            return Err(CacheError::FileNotFound);
        }
        let encoded = fs::read(file_path)?;
        let decoded: T = bincode::deserialize(&encoded[..])?;
        Ok(decoded)
    }
}
