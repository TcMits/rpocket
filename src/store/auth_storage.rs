use crate::error::RPocketError;

pub static TOKEN_KEY: &str = "pb_auth";
pub static USER_OR_ADMIN_KEY: &str = "pb_user_or_admin";

pub struct AuthStorage<'a> {
    token_key: &'static str,
    user_or_admin_key: &'static str,
    store: &'a dyn crate::store::Storage,
}

impl<'a> AuthStorage<'a> {
    /// create a new AuthStorage.
    /// store: the storage to use.
    pub fn new(store: &'a dyn crate::store::Storage) -> Self {
        return AuthStorage {
            token_key: TOKEN_KEY,
            user_or_admin_key: USER_OR_ADMIN_KEY,
            store,
        };
    }

    /// get the token.
    /// return: the token if it exists, otherwise return None.
    pub fn token(&self) -> Result<Option<String>, RPocketError> {
        return self.store.get(self.token_key);
    }

    // get the user or admin record.
    // return: the user or admin record if it exists, otherwise return None.
    pub fn user_or_admin(
        &self,
    ) -> Result<Option<serde_json::Map<String, serde_json::Value>>, RPocketError> {
        let data = self.store.get(self.user_or_admin_key)?;
        return match data {
            Some(data) => {
                let record: serde_json::Map<String, serde_json::Value> =
                    serde_json::from_str(&data).map_err(|e| RPocketError::SerdeError(e))?;
                return Ok(Some(record));
            }
            None => Ok(None),
        };
    }

    /// clear the storage.
    pub fn clear(&self) -> Result<(), RPocketError> {
        return self.store.clear();
    }

    fn save_token(&self, token: &str) -> Result<(), RPocketError> {
        return self.store.set(self.token_key, token);
    }

    fn save_user_or_admin(&self, record: &serde_json::Value) -> Result<(), RPocketError> {
        return self.store.set(
            self.user_or_admin_key,
            &serde_json::to_string(record).map_err(|e| RPocketError::SerdeError(e))?,
        );
    }

    // save the token and the user or admin record.
    // token: the token to save.
    // record: the user or admin record to save.
    pub fn save(&self, token: &str, record: &serde_json::Value) -> Result<(), RPocketError> {
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
        let store = crate::store::MemoryStorage::new();
        let auth_storage = AuthStorage::new(&store);

        // test clear
        assert!(auth_storage.clear().is_ok());

        // test token
        assert!(auth_storage.token().unwrap().is_none());
        assert!(auth_storage.save_token("token").is_ok());
        assert_eq!(auth_storage.token().unwrap().unwrap(), "token");

        // test user or admin
        assert!(auth_storage.user_or_admin().unwrap().is_none());
        assert!(auth_storage
            .save_user_or_admin(&serde_json::json!({"key":"value"}))
            .is_ok());
        assert_eq!(
            auth_storage
                .user_or_admin()
                .unwrap()
                .unwrap()
                .get("key")
                .unwrap(),
            "value"
        );

        // test save
        assert!(auth_storage
            .save("token", &serde_json::json!({"key":"value"}))
            .is_ok());
        assert_eq!(auth_storage.token().unwrap().unwrap(), "token");
        assert_eq!(
            auth_storage
                .user_or_admin()
                .unwrap()
                .unwrap()
                .get("key")
                .unwrap(),
            "value"
        );

        // test clear
        assert!(auth_storage.clear().is_ok());
        assert!(auth_storage.token().unwrap().is_none());
        assert!(auth_storage.user_or_admin().unwrap().is_none());
    }
}
