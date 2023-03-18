pub mod auth_storage;

use crate::error::RPocketError;

pub trait Storage: Send + Sync {
    /// get the value of a key.
    fn get(&self, key: &str) -> Result<Option<String>, RPocketError>;
    /// set the value of a key.
    fn set(&self, key: &str, value: &str) -> Result<(), RPocketError>;
    // clear
    fn clear(&self) -> Result<(), RPocketError>;
}

/// MemoryStorage is a simple implementation of Storage.
pub struct MemoryStorage {
    data: std::sync::RwLock<std::collections::HashMap<String, String>>,
}

impl MemoryStorage {
    /// create a new MemoryStorage.
    pub fn new() -> Self {
        return MemoryStorage {
            data: std::sync::RwLock::new(std::collections::HashMap::new()),
        };
    }
}

impl Storage for MemoryStorage {
    /// get the value of a key.
    /// return: the value if it exists, otherwise return None.
    fn get(&self, key: &str) -> Result<Option<String>, RPocketError> {
        let data = self.data.read().map_err(|_| RPocketError::MutexError)?;
        return Ok(data.get(key).map(|v| v.to_string()));
    }

    /// set the value of a key.
    fn set(&self, key: &str, value: &str) -> Result<(), RPocketError> {
        let mut data = self.data.write().map_err(|_| RPocketError::MutexError)?;
        data.insert(key.to_string(), value.to_string());
        return Ok(());
    }

    /// clear the storage.
    fn clear(&self) -> Result<(), RPocketError> {
        let mut data = self.data.write().map_err(|_| RPocketError::MutexError)?;
        data.clear();
        return Ok(());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_memory_storage() {
        let storage = MemoryStorage::new();
        storage.set("key", "value").unwrap();
        assert_eq!(storage.get("key").unwrap().unwrap(), "value");
        storage.clear().unwrap();
        assert_eq!(storage.get("key").unwrap(), None);
    }
}
