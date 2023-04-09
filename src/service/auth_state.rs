use serde::{Deserialize, Serialize};

use crate::error::RPocketError;
use crate::{model::Admin, model::Record};

/// AuthPayload is the payload for the auth state.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AuthPayload {
    User(Record),
    Admin(Admin),
}

/// AuthStateService is the service for the auth state.
pub struct AuthStateService<'a, C> {
    client: &'a mut C,
    token_key: &'a str,
    user_or_admin_key: &'a str,
}

impl<'a, C> AuthStateService<'a, C>
where
    C: crate::rpocket::PocketBaseClient + Sized,
{
    /// create a new AuthStateService with custom keys.
    pub fn new(client: &'a mut C, token_key: &'a str, user_or_admin_key: &'a str) -> Self {
        return AuthStateService {
            client,
            token_key,
            user_or_admin_key,
        };
    }

    async fn save_token(&self, token: &str) -> Result<(), RPocketError> {
        return self.client.storage().set(self.token_key, token).await;
    }

    async fn save_user_or_admin(&self, record: &AuthPayload) -> Result<(), RPocketError> {
        return self
            .client
            .storage()
            .set(self.user_or_admin_key, &serde_json::to_string(record)?)
            .await;
    }

    /// get the token.
    pub async fn get_token(&self) -> Result<Option<String>, RPocketError> {
        return self.client.storage().get(self.token_key).await;
    }

    // get the user or admin record.
    pub async fn get_user_or_admin(&self) -> Result<Option<AuthPayload>, RPocketError> {
        let storage = self.client.storage();
        let data = storage.get(self.user_or_admin_key).await?;
        return match data {
            Some(data) => {
                let record: AuthPayload = serde_json::from_str(&data)?;
                return Ok(Some(record));
            }
            None => Ok(None),
        };
    }

    /// clear the storage.
    pub async fn clear(&self) -> Result<(), RPocketError> {
        let storage = self.client.storage();
        storage.delete(self.token_key).await?;
        storage.delete(self.user_or_admin_key).await?;
        return Ok(());
    }

    // save the token and the user or admin record.
    pub async fn save(&self, token: &str, record: &AuthPayload) -> Result<(), RPocketError> {
        self.save_token(token).await?;
        self.save_user_or_admin(record).await?;
        return Ok(());
    }
}

#[cfg(test)]
mod test {
    use crate::PocketBase;

    use super::*;

    #[tokio::test]
    async fn test_auth_storage() {
        let mut base = PocketBase::new("http://hello.world", "en");
        let auth_service = AuthStateService::new(&mut base, "foo", "bar");

        // test clear
        assert!(auth_service.clear().await.is_ok());

        // test token
        assert!(auth_service.get_token().await.unwrap().is_none());
        assert!(auth_service.save_token("token").await.is_ok());
        assert_eq!(auth_service.get_token().await.unwrap().unwrap(), "token");

        // test save
        let user_or_admin = AuthPayload::Admin(Admin {
            ..Default::default()
        });
        assert!(auth_service.save("token", &user_or_admin).await.is_ok());
        assert_eq!(auth_service.get_token().await.unwrap().unwrap(), "token");
        assert_eq!(
            auth_service.get_user_or_admin().await.unwrap().unwrap(),
            user_or_admin
        );

        // test clear
        assert!(auth_service.clear().await.is_ok());
        assert!(auth_service.get_token().await.unwrap().is_none());
        assert!(auth_service.get_user_or_admin().await.unwrap().is_none());
    }
}
