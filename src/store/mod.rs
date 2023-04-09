use crate::error::RPocketError;
use async_trait::async_trait;

#[async_trait]
pub trait Storage {
    async fn get(&self, key: &str) -> Result<Option<String>, RPocketError>;
    async fn set(&self, key: &str, value: &str) -> Result<(), RPocketError>;
    async fn delete(&self, key: &str) -> Result<(), RPocketError>;
}

/// MemoryStorage is a simple implementation of Storage.
pub struct MemoryStorage {
    pub data: std::sync::RwLock<std::collections::HashMap<String, std::sync::RwLock<String>>>,
}

impl MemoryStorage {
    /// create a new MemoryStorage.
    pub fn new() -> Self {
        return MemoryStorage {
            data: std::sync::RwLock::new(std::collections::HashMap::new()),
        };
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    /// get the value of a key.
    async fn get(&self, key: &str) -> Result<Option<String>, RPocketError> {
        let data = self.data.read().map_err(|_| RPocketError::MutexError)?;
        let value = data.get(key);

        return match value {
            Some(value) => {
                let value = value.read().map_err(|_| RPocketError::MutexError)?;
                return Ok(Some(value.to_string()));
            }
            None => Ok(None),
        };
    }

    /// set the value of a key.
    async fn set(&self, key: &str, value: &str) -> Result<(), RPocketError> {
        let data = self.data.read().map_err(|_| RPocketError::MutexError)?;
        let inner_entry = data.get(key);

        match inner_entry {
            Some(entry) => {
                let mut inner_value = entry.write().map_err(|_| RPocketError::MutexError)?;
                *inner_value = value.to_string();
            }
            None => {
                drop(data);
                let mut data = self.data.write().map_err(|_| RPocketError::MutexError)?;
                data.insert(key.to_string(), std::sync::RwLock::new(value.to_string()));
            }
        }

        return Ok(());
    }

    /// delete a key.
    async fn delete(&self, key: &str) -> Result<(), RPocketError> {
        let data = self.data.read().map_err(|_| RPocketError::MutexError)?;
        let inner_entry = data.get(key);

        match inner_entry {
            Some(..) => {
                drop(data);
                let mut data = self.data.write().map_err(|_| RPocketError::MutexError)?;
                data.remove(key);
            }
            None => {}
        }

        return Ok(());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_memory_storage() {
        let storage = MemoryStorage::new();
        storage.set("key", "value").await.unwrap();
        assert_eq!(storage.get("key").await.unwrap().unwrap(), "value");

        storage.set("key", "value2").await.unwrap();
        assert_eq!(storage.get("key").await.unwrap().unwrap(), "value2");

        storage.delete("key").await.unwrap();
        assert_eq!(storage.get("key").await.unwrap(), None);
    }
}
