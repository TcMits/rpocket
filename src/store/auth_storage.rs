use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::RPocketError;
use crate::model::{Admin, Record};

pub static TOKEN_KEY: &str = "pb_auth";
pub static USER_OR_ADMIN_KEY: &str = "pb_user_or_admin";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AuthPayload {
    User(Record),
    Admin(Admin),
}

pub trait AuthState {
    fn token(&self) -> Result<Option<String>, RPocketError>;
    fn user_or_admin(&self) -> Result<Option<AuthPayload>, RPocketError>;
    fn save(&self, token: &str, record: &AuthPayload) -> Result<(), RPocketError>;
    fn clear(&self) -> Result<(), RPocketError>;
}

pub struct AuthStorage {
    token_key: &'static str,
    user_or_admin_key: &'static str,
    store: Arc<dyn crate::store::Storage + Send + Sync>,
}

impl AuthStorage {
    /// create a new AuthStorage.
    /// store: the storage to use.
    pub fn new(store: Arc<dyn crate::store::Storage + Send + Sync>) -> Self {
        return crate::store::auth_storage::AuthStorage::new_with_keys(
            store,
            TOKEN_KEY,
            USER_OR_ADMIN_KEY,
        );
    }

    /// create a new AuthStorage with custom keys.
    /// store: the storage to use.
    /// token_key: the key to use for the token.
    /// user_or_admin_key: the key to use for the user or admin record.
    pub fn new_with_keys(
        store: Arc<dyn crate::store::Storage + Send + Sync>,
        token_key: &'static str,
        user_or_admin_key: &'static str,
    ) -> Self {
        return AuthStorage {
            token_key,
            user_or_admin_key,
            store,
        };
    }

    fn save_token(&self, token: &str) -> Result<(), RPocketError> {
        return self.store.set(self.token_key, token);
    }

    fn save_user_or_admin(&self, record: &AuthPayload) -> Result<(), RPocketError> {
        return self.store.set(
            self.user_or_admin_key,
            &serde_json::to_string(record).map_err(|e| RPocketError::SerdeError(e))?,
        );
    }
}

impl AuthState for AuthStorage {
    /// get the token.
    /// return: the token if it exists, otherwise return None.
    fn token(&self) -> Result<Option<String>, RPocketError> {
        return self.store.get(self.token_key);
    }

    // get the user or admin record.
    // return: the user or admin record if it exists, otherwise return None.
    fn user_or_admin(&self) -> Result<Option<AuthPayload>, RPocketError> {
        let data = self.store.get(self.user_or_admin_key)?;
        return match data {
            Some(data) => {
                let record: AuthPayload =
                    serde_json::from_str(&data).map_err(|e| RPocketError::SerdeError(e))?;
                return Ok(Some(record));
            }
            None => Ok(None),
        };
    }

    /// clear the storage.
    fn clear(&self) -> Result<(), RPocketError> {
        return self.store.clear();
    }

    // save the token and the user or admin record.
    // token: the token to save.
    // record: the user or admin record to save.
    fn save(&self, token: &str, record: &AuthPayload) -> Result<(), RPocketError> {
        self.save_token(token)?;
        self.save_user_or_admin(record)?;
        return Ok(());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_auth_storage() {
        let store = Arc::new(crate::store::MemoryStorage::new());
        let auth_storage = AuthStorage::new(store.clone());

        // test clear
        assert!(auth_storage.clear().is_ok());

        // test token
        assert!(auth_storage.token().unwrap().is_none());
        assert!(auth_storage.save_token("token").is_ok());
        assert_eq!(auth_storage.token().unwrap().unwrap(), "token");

        // test save
        let user_or_admin = AuthPayload::Admin(Admin {
            ..Default::default()
        });
        assert!(auth_storage.save("token", &user_or_admin).is_ok());
        assert_eq!(auth_storage.token().unwrap().unwrap(), "token");
        assert_eq!(
            auth_storage.user_or_admin().unwrap().unwrap(),
            user_or_admin
        );

        // test clear
        assert!(auth_storage.clear().is_ok());
        assert!(auth_storage.token().unwrap().is_none());
        assert!(auth_storage.user_or_admin().unwrap().is_none());
    }
}
