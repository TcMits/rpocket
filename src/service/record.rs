use std::collections::HashMap;

use crate::error::RPocketError;
use crate::model::Record;
use crate::service;
use serde::{Deserialize, Serialize};

/// RecordAuthResponse is the response for the auth.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordAuthResponse<T> {
    pub token: String,
    pub record: T,
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// AuthProvicderInfo is the info for an auth provider.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthProvicderInfo {
    pub name: String,
    pub state: String,
    pub code_verifier: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub auth_url: String,
}

/// ListAuthMethod is the model for a list auth method.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAuthMethod {
    pub username_password: bool,
    pub email_password: bool,
    pub auth_providers: Vec<AuthProvicderInfo>,
}

/// RecordListAuthMethodsResponse is the response for the list auth methods.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordListAuthMethodsConfig {
    pub query_params: Vec<(String, String)>,
}

/// RecordAuthWithPasswordConfig is the config for the auth with password.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordAuthWithPasswordConfig<T> {
    pub identity: String,
    pub password: String,
    #[serde(flatten)]
    pub body: T,
    #[serde(skip)]
    pub query_params: Vec<(String, String)>,
    #[serde(skip)]
    pub without_saving: bool,
}

/// RecordAuthWithOAuth2Config is the config for the auth with OAuth2.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordAuthWithOAuth2Config<T> {
    pub provider: String,
    pub code: String,
    #[serde(rename = "codeVerifier")]
    pub code_verifier: String,
    #[serde(rename = "redirectUrl")]
    pub redirect_url: String,
    #[serde(flatten)]
    pub body: T,
    #[serde(skip)]
    pub query_params: Vec<(String, String)>,
    #[serde(skip)]
    pub without_saving: bool,
}

/// RecordAuthRefreshConfig is the config for the auth refresh.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordAuthRefreshConfig<T> {
    #[serde(flatten)]
    pub body: T,
    #[serde(skip)]
    pub query_params: Vec<(String, String)>,
    #[serde(skip)]
    pub without_saving: bool,
}

/// RecordRequestPasswordResetConfig is the config for the request password reset.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordRequestPasswordResetConfig<T> {
    pub email: String,
    #[serde(flatten)]
    pub body: T,
    #[serde(skip)]
    pub query_params: Vec<(String, String)>,
}

/// RecordConfirmPasswordResetConfig is the config for the confirm password reset.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordConfirmPasswordResetConfig<T> {
    pub token: String,
    pub password: String,
    #[serde(rename = "passwordConfirm")]
    pub password_confirm: String,
    #[serde(flatten)]
    pub body: T,
    #[serde(skip)]
    pub query_params: Vec<(String, String)>,
}

/// RecordRequestVerificationConfig is the config for the request verification.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordRequestVerificationConfig<T> {
    pub email: String,
    #[serde(flatten)]
    pub body: T,
    #[serde(skip)]
    pub query_params: Vec<(String, String)>,
}

/// RecordConfirmVerificationConfig is the config for the confirm verification.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordConfirmVerificationConfig<T> {
    pub token: String,
    #[serde(flatten)]
    pub body: T,
    #[serde(skip)]
    pub query_params: Vec<(String, String)>,
}

/// RecordRequestEmailChangeConfig is the config for the request email change.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordRequestEmailChangeConfig<T> {
    #[serde(rename = "newEmail")]
    pub new_email: String,
    #[serde(flatten)]
    pub body: T,
    #[serde(skip)]
    pub query_params: Vec<(String, String)>,
}

/// RecordConfirmEmailChangeConfig is the config for the confirm email change.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordConfirmEmailChangeConfig<T> {
    pub token: String,
    pub password: String,
    #[serde(flatten)]
    pub body: T,
    #[serde(skip)]
    pub query_params: Vec<(String, String)>,
}

/// RecordListExternalAuthsConfig is the config for the list external auths.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordListExternalAuthsConfig {
    pub id: String,
    pub query_params: Vec<(String, String)>,
}

/// RecordUnlinkExternalAuthConfig is the config for the unlink external auth.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordUnlinkExternalAuthConfig {
    pub id: String,
    pub provider: String,
    pub query_params: Vec<(String, String)>,
}

/// RecordService is the service for the record.
pub struct RecordService<'a, C> {
    client: &'a mut C,
    collection: &'a str,
    record_base_path: String,
}

impl<'a, C> RecordService<'a, C>
where
    C: crate::rpocket::PocketBaseClient + Sized,
{
    /// create a new RecordService.
    pub fn new(client: &'a mut C, collection: &'a str) -> Self {
        return RecordService {
            client,
            collection,
            record_base_path: format!("api/collections/{}/records", collection),
        };
    }

    /// returns crud service.
    pub fn crud(&'a mut self) -> service::crud::CRUDService<'a, C> {
        return self.client.crud(&self.record_base_path);
    }

    /// list auth methods
    pub async fn list_auth_methods<T>(
        &mut self,
        config: &RecordListAuthMethodsConfig,
    ) -> Result<T, RPocketError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = self
            .client
            .base_url()
            .join(format!("api/collections/{}/auth-methods", self.collection).as_str())?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::GET, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params);

        let response = self.client.http().send(request_builder).await?;

        return Ok(response.json::<T>().await?);
    }

    async fn save_auth_response<T>(
        &mut self,
        response: reqwest::Response,
    ) -> Result<T, RPocketError>
    where
        T: serde::de::DeserializeOwned,
    {
        let auth_state = self.client.auth_state();
        let auth_response = response.json::<RecordAuthResponse<Record>>().await?;

        let token = auth_response.token;
        let meta = auth_response.meta;
        let user = service::auth_state::AuthPayload::User(auth_response.record);

        auth_state.save(token.as_str(), &user).await?;

        let record = match user {
            service::auth_state::AuthPayload::User(user) => user,
            _ => unreachable!(),
        };

        let auth_response = RecordAuthResponse {
            token,
            record,
            meta,
        };

        let auth_response = serde_json::to_value(&auth_response)?;
        let response = serde_json::from_value(auth_response)?;

        return Ok(response);
    }

    /// authenticate with password
    pub async fn auth_with_password<T, B>(
        &mut self,
        config: &RecordAuthWithPasswordConfig<B>,
    ) -> Result<T, RPocketError>
    where
        T: serde::de::DeserializeOwned,
        B: Serialize,
    {
        let url = self
            .client
            .base_url()
            .join(format!("api/collections/{}/auth-with-password", self.collection).as_str())?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::POST, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params)
            .json(&config);

        let response = self.client.http().send(request_builder).await?;

        if !config.without_saving {
            return self.save_auth_response::<T>(response).await;
        }

        return Ok(response.json::<T>().await?);
    }

    /// authenticate with oauth2
    pub async fn auth_with_oauth2<T, B>(
        &mut self,
        config: &RecordAuthWithOAuth2Config<B>,
    ) -> Result<T, RPocketError>
    where
        T: serde::de::DeserializeOwned,
        B: Serialize,
    {
        let url = self
            .client
            .base_url()
            .join(format!("api/collections/{}/auth-with-oauth2", self.collection).as_str())?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::POST, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params)
            .json(&config);

        let response = self.client.http().send(request_builder).await?;

        if !config.without_saving {
            return self.save_auth_response::<T>(response).await;
        }

        return Ok(response.json::<T>().await?);
    }

    /// refreshes the current authenticated record instance and
    pub async fn auth_refresh<T, B>(
        &mut self,
        config: &RecordAuthRefreshConfig<B>,
    ) -> Result<T, RPocketError>
    where
        T: serde::de::DeserializeOwned,
        B: Serialize,
    {
        let url = self
            .client
            .base_url()
            .join(format!("api/collections/{}/auth-refresh", self.collection).as_str())?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::POST, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params)
            .json(&config);

        let response = self.client.http().send(request_builder).await?;

        if !config.without_saving {
            return self.save_auth_response::<T>(response).await;
        }

        return Ok(response.json::<T>().await?);
    }

    /// sends auth record password reset request.
    pub async fn request_password_reset<B>(
        &mut self,
        config: &RecordRequestPasswordResetConfig<B>,
    ) -> Result<(), RPocketError>
    where
        B: Serialize,
    {
        let url = self
            .client
            .base_url()
            .join(format!("api/collections/{}/request-password-reset", self.collection).as_str())?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::POST, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params)
            .json(&config);

        self.client.http().send(request_builder).await?;
        return Ok(());
    }

    /// confirms auth record password reset request.
    pub async fn confirm_password_reset<B>(
        &mut self,
        config: &RecordConfirmPasswordResetConfig<B>,
    ) -> Result<(), RPocketError>
    where
        B: Serialize,
    {
        let url = self
            .client
            .base_url()
            .join(format!("api/collections/{}/confirm-password-reset", self.collection).as_str())?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::POST, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params)
            .json(&config);

        self.client.http().send(request_builder).await?;
        return Ok(());
    }

    /// sends auth record verification email request.
    pub async fn request_verification<B>(
        &mut self,
        config: &RecordRequestVerificationConfig<B>,
    ) -> Result<(), RPocketError>
    where
        B: Serialize,
    {
        let url = self
            .client
            .base_url()
            .join(format!("api/collections/{}/request-verification", self.collection).as_str())?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::POST, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params)
            .json(&config);

        self.client.http().send(request_builder).await?;

        return Ok(());
    }

    /// confirms auth record verification email request.
    pub async fn confirm_verification<B>(
        &mut self,
        config: &RecordConfirmVerificationConfig<B>,
    ) -> Result<(), RPocketError>
    where
        B: Serialize,
    {
        let url = self
            .client
            .base_url()
            .join(format!("api/collections/{}/confirm-verification", self.collection).as_str())?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::POST, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params)
            .json(&config);

        self.client.http().send(request_builder).await?;

        return Ok(());
    }

    /// sends an email change request to the authenticated record model.
    pub async fn request_email_change<B>(
        &mut self,
        config: &RecordRequestEmailChangeConfig<B>,
    ) -> Result<(), RPocketError>
    where
        B: Serialize,
    {
        let url = self
            .client
            .base_url()
            .join(format!("api/collections/{}/request-email-change", self.collection).as_str())?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::POST, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params)
            .json(&config);

        self.client.http().send(request_builder).await?;

        return Ok(());
    }

    /// confirms auth record's new email address.
    pub async fn confirm_email_change<B>(
        &mut self,
        config: &RecordConfirmEmailChangeConfig<B>,
    ) -> Result<(), RPocketError>
    where
        B: Serialize,
    {
        let url = self
            .client
            .base_url()
            .join(format!("api/collections/{}/confirm-email-change", self.collection).as_str())?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::POST, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params)
            .json(&config);

        self.client.http().send(request_builder).await?;

        return Ok(());
    }

    /// lists all linked external auth providers for the specified auth record.
    pub async fn list_external_auths<T>(
        &mut self,
        config: &RecordListExternalAuthsConfig,
    ) -> Result<Vec<T>, RPocketError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = self.client.base_url().join(
            format!(
                "api/collections/{}/records/{}/external-auths",
                self.collection, config.id
            )
            .as_str(),
        )?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::GET, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params);

        let response = self.client.http().send(request_builder).await?;

        return Ok(response.json::<Vec<T>>().await?);
    }

    /// unlink a single external auth provider from the specified auth record.
    pub async fn unlink_external_auth(
        &mut self,
        config: &RecordUnlinkExternalAuthConfig,
    ) -> Result<(), RPocketError> {
        let url = self.client.base_url().join(
            format!(
                "api/collections/{}/records/{}/external-auths/{}",
                self.collection, config.id, config.provider
            )
            .as_str(),
        )?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::DELETE, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params);

        self.client.http().send(request_builder).await?;

        return Ok(());
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::{ExternalAuth, Record};
    use crate::rpocket::{PocketBase, PocketBaseClient};
    use std::collections::HashMap;
    use std::str::FromStr;

    #[test]
    fn test_record_crud() {
        let mut base = PocketBase::new("http://test.com", "en");
        let record_service = RecordService::new(&mut base, "test");

        assert!(record_service.record_base_path == "api/collections/test/records");
    }

    #[tokio::test]
    async fn test_record_list_auth_methods() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("GET", "/api/collections/test/auth-methods")
            .with_status(200)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .with_body(
                r#"{
  "usernamePassword": false,
  "emailPassword": true,
  "authProviders": [
    {
      "name": "github",
      "state": "3Yd8jNkK_6PJG6hPWwBjLqKwse6Ejd",
      "codeVerifier": "KxFDWz1B3fxscCDJ_9gHQhLuh__ie7",
      "codeChallenge": "NM1oVexB6Q6QH8uPtOUfK7tq4pmu4Jz6lNDIwoxHZNE=",
      "codeChallengeMethod": "S256",
      "authUrl": "https://github.com/login/oauth/authorize?client_id=demo&code_challenge=NM1oVexB6Q6QH8uPtOUfK7tq4pmu4Jz6lNDIwoxHZNE%3D&code_challenge_method=S256&response_type=code&scope=user&state=3Yd8jNkK_6PJG6hPWwBjLqKwse6Ejd&redirect_uri="
    },
    {
      "name": "gitlab",
      "state": "NeQSbtO5cShr_mk5__3CUukiMnymeb",
      "codeVerifier": "ahTFHOgua8mkvPAlIBGwCUJbWKR_xi",
      "codeChallenge": "O-GATkTj4eXDCnfonsqGLCd6njvTixlpCMvy5kjgOOg=",
      "codeChallengeMethod": "S256",
      "authUrl": "https://gitlab.com/oauth/authorize?client_id=demo&code_challenge=O-GATkTj4eXDCnfonsqGLCd6njvTixlpCMvy5kjgOOg%3D&code_challenge_method=S256&response_type=code&scope=read_user&state=NeQSbtO5cShr_mk5__3CUukiMnymeb&redirect_uri="
    },
    {
      "name": "google",
      "state": "zB3ZPifV1TW2GMuvuFkamSXfSNkHPQ",
      "codeVerifier": "t3CmO5VObGzdXqieakvR_fpjiW0zdO",
      "codeChallenge": "KChwoQPKYlz2anAdqtgsSTdIo8hdwtc1fh2wHMwW2Yk=",
      "codeChallengeMethod": "S256",
      "authUrl": "https://accounts.google.com/o/oauth2/auth?client_id=demo&code_challenge=KChwoQPKYlz2anAdqtgsSTdIo8hdwtc1fh2wHMwW2Yk%3D&code_challenge_method=S256&response_type=code&scope=https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fuserinfo.profile+https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fuserinfo.email&state=zB3ZPifV1TW2GMuvuFkamSXfSNkHPQ&redirect_uri="
    }
  ]            }"#,
            )
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut record_service = RecordService::new(&mut base, "test");
        let config = RecordListAuthMethodsConfig {
            query_params: Vec::new(),
        };

        let response = record_service
            .list_auth_methods::<ListAuthMethod>(&config)
            .await;
        mock.assert_async().await;
        let response = response.unwrap();

        assert!(response.username_password == false);
        assert!(response.email_password == true);
        assert!(response.auth_providers.len() == 3);
        assert!(response.auth_providers[0].name == "github");
        assert!(response.auth_providers[0].state == "3Yd8jNkK_6PJG6hPWwBjLqKwse6Ejd");
        assert!(response.auth_providers[0].code_verifier == "KxFDWz1B3fxscCDJ_9gHQhLuh__ie7");
        assert!(
            response.auth_providers[0].code_challenge
                == "NM1oVexB6Q6QH8uPtOUfK7tq4pmu4Jz6lNDIwoxHZNE="
        );
        assert!(response.auth_providers[0].code_challenge_method == "S256");
        assert!(
            response.auth_providers[0].auth_url
                == "https://github.com/login/oauth/authorize?client_id=demo&code_challenge=NM1oVexB6Q6QH8uPtOUfK7tq4pmu4Jz6lNDIwoxHZNE%3D&code_challenge_method=S256&response_type=code&scope=user&state=3Yd8jNkK_6PJG6hPWwBjLqKwse6Ejd&redirect_uri=")
    }

    #[tokio::test]
    async fn test_record_auth_with_password() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("POST", "/api/collections/test/auth-with-password")
            .with_status(200)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .match_body(r#"{"identity":"test","password":"12345678"}"#)
            .with_body(
                r#"{
  "token": "eyJhbGciOiJIUzI1NiJ9.eyJpZCI6IjRxMXhsY2xtZmxva3UzMyIsInR5cGUiOiJhdXRoUmVjb3JkIiwiY29sbGVjdGlvbklkIjoiX3BiX3VzZXJzX2F1dGhfIiwiZXhwIjoyMjA4OTg1MjYxfQ.UwD8JvkbQtXpymT09d7J6fdA0aP9g4FJ1GPh_ggEkzc",
  "record": {
    "id": "8171022dc95a4ed",
    "collectionId": "d2972397d45614e",
    "collectionName": "users",
    "created": "2022-06-24 06:24:18.434Z",
    "updated": "2022-06-24 06:24:18.889Z",
    "username": "test@example.com",
    "email": "test@example.com",
    "verified": false,
    "emailVisibility": true,
    "someCustomField": "example 123"
  }
                }"#,
                )
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut record_service = RecordService::new(&mut base, "test");
        let config = RecordAuthWithPasswordConfig::<HashMap<String, String>> {
            identity: String::from_str("test").unwrap(),
            password: String::from_str("12345678").unwrap(),
            body: HashMap::new(),
            query_params: Vec::new(),
            ..Default::default()
        };

        let response = record_service
            .auth_with_password::<RecordAuthResponse<Record>, HashMap<String, String>>(&config)
            .await;

        mock.assert_async().await;
        let response = response.unwrap();

        let auth_state_token = base.auth_state().get_token().await.unwrap().unwrap();
        let auth_record = match base
            .auth_state()
            .get_user_or_admin()
            .await
            .unwrap()
            .unwrap()
        {
            service::auth_state::AuthPayload::User(user) => user,
            _ => unreachable!(),
        };

        assert!(auth_state_token == "eyJhbGciOiJIUzI1NiJ9.eyJpZCI6IjRxMXhsY2xtZmxva3UzMyIsInR5cGUiOiJhdXRoUmVjb3JkIiwiY29sbGVjdGlvbklkIjoiX3BiX3VzZXJzX2F1dGhfIiwiZXhwIjoyMjA4OTg1MjYxfQ.UwD8JvkbQtXpymT09d7J6fdA0aP9g4FJ1GPh_ggEkzc");
        assert!(auth_record.base.id == "8171022dc95a4ed");
        assert!(auth_record.collection_id == "d2972397d45614e");
        assert!(auth_record.collection_name == "users");
        assert!(auth_record.base.created == "2022-06-24 06:24:18.434Z");
        assert!(auth_record.base.updated == "2022-06-24 06:24:18.889Z");
        assert!(auth_record.data["username"] == "test@example.com");
        assert!(auth_record.data["email"] == "test@example.com");
        assert!(auth_record.data["verified"] == false);
        assert!(auth_record.data["emailVisibility"] == true);
        assert!(auth_record.data["someCustomField"] == "example 123");

        assert!(response.token == "eyJhbGciOiJIUzI1NiJ9.eyJpZCI6IjRxMXhsY2xtZmxva3UzMyIsInR5cGUiOiJhdXRoUmVjb3JkIiwiY29sbGVjdGlvbklkIjoiX3BiX3VzZXJzX2F1dGhfIiwiZXhwIjoyMjA4OTg1MjYxfQ.UwD8JvkbQtXpymT09d7J6fdA0aP9g4FJ1GPh_ggEkzc");
        assert!(response.record.base.id == "8171022dc95a4ed");
        assert!(response.record.collection_id == "d2972397d45614e");
        assert!(response.record.collection_name == "users");
        assert!(response.record.base.created == "2022-06-24 06:24:18.434Z");
        assert!(response.record.base.updated == "2022-06-24 06:24:18.889Z");
        assert!(response.record.data["username"] == "test@example.com");
        assert!(response.record.data["email"] == "test@example.com");
        assert!(response.record.data["verified"] == false);
        assert!(response.record.data["emailVisibility"] == true);
        assert!(response.record.data["someCustomField"] == "example 123");
    }

    #[tokio::test]
    async fn test_record_auth_with_oauth2() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("POST", "/api/collections/test/auth-with-oauth2")
            .with_status(200)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .match_body(r#"{"provider":"google","code":"12345678","codeVerifier":"12345678","redirectUrl":"https://example.com"}"#)
            .with_body(
                r#"{
  "token": "eyJhbGciOiJIUzI1NiJ9.eyJpZCI6IjRxMXhsY2xtZmxva3UzMyIsInR5cGUiOiJhdXRoUmVjb3JkIiwiY29sbGVjdGlvbklkIjoiX3BiX3VzZXJzX2F1dGhfIiwiZXhwIjoyMjA4OTg1MjYxfQ.UwD8JvkbQtXpymT09d7J6fdA0aP9g4FJ1GPh_ggEkzc",
  "record": {
    "id": "8171022dc95a4ed",
    "collectionId": "d2972397d45614e",
    "collectionName": "users",
    "created": "2022-06-24 06:24:18.434Z",
    "updated": "2022-06-24 06:24:18.889Z",
    "username": "test@example.com",
    "email": "test@example.com",
    "verified": true,
    "emailVisibility": false,
    "someCustomField": "example 123"
  },
  "meta": {
    "id": "abc123",
    "name": "John Doe",
    "username": "john.doe",
    "email": "test@example.com",
    "avatarUrl": "https://example.com/avatar.png"
  }                }"#,
                )
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut record_service = RecordService::new(&mut base, "test");
        let config = RecordAuthWithOAuth2Config::<HashMap<String, String>> {
            provider: String::from_str("google").unwrap(),
            code: String::from_str("12345678").unwrap(),
            code_verifier: String::from_str("12345678").unwrap(),
            redirect_url: String::from_str("https://example.com").unwrap(),
            body: HashMap::new(),
            query_params: Vec::new(),
            ..Default::default()
        };

        let response = record_service
            .auth_with_oauth2::<RecordAuthResponse<Record>, HashMap<String, String>>(&config)
            .await;

        mock.assert_async().await;
        let response = response.unwrap();

        assert!(response.token == "eyJhbGciOiJIUzI1NiJ9.eyJpZCI6IjRxMXhsY2xtZmxva3UzMyIsInR5cGUiOiJhdXRoUmVjb3JkIiwiY29sbGVjdGlvbklkIjoiX3BiX3VzZXJzX2F1dGhfIiwiZXhwIjoyMjA4OTg1MjYxfQ.UwD8JvkbQtXpymT09d7J6fdA0aP9g4FJ1GPh_ggEkzc");
        assert!(response.record.base.id == "8171022dc95a4ed");
        assert!(response.record.collection_id == "d2972397d45614e");
        assert!(response.record.collection_name == "users");
        assert!(response.record.base.created == "2022-06-24 06:24:18.434Z");
        assert!(response.record.base.updated == "2022-06-24 06:24:18.889Z");
        assert!(response.record.data["username"] == "test@example.com");
        assert!(response.record.data["email"] == "test@example.com");
        assert!(response.record.data["verified"] == true);
        assert!(response.record.data["emailVisibility"] == false);
        assert!(response.record.data["someCustomField"] == "example 123");
        let meta = response.meta.unwrap();
        assert!(meta["id"] == "abc123");
        assert!(meta["name"] == "John Doe");
        assert!(meta["username"] == "john.doe");
        assert!(meta["email"] == "test@example.com");
        assert!(meta["avatarUrl"] == "https://example.com/avatar.png");
    }

    #[tokio::test]
    async fn test_record_auth_refresh() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("POST", "/api/collections/test/auth-refresh")
            .with_status(200)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .match_body(r#"{}"#)
            .with_body(
                r#"{
  "token": "eyJhbGciOiJIUzI1NiJ9.eyJpZCI6IjRxMXhsY2xtZmxva3UzMyIsInR5cGUiOiJhdXRoUmVjb3JkIiwiY29sbGVjdGlvbklkIjoiX3BiX3VzZXJzX2F1dGhfIiwiZXhwIjoyMjA4OTg1MjYxfQ.UwD8JvkbQtXpymT09d7J6fdA0aP9g4FJ1GPh_ggEkzc",
  "record": {
    "id": "8171022dc95a4ed",
    "collectionId": "d2972397d45614e",
    "collectionName": "users",
    "created": "2022-06-24 06:24:18.434Z",
    "updated": "2022-06-24 06:24:18.889Z",
    "username": "test@example.com",
    "email": "test@example.com",
    "verified": false,
    "emailVisibility": true,
    "someCustomField": "example 123"
  }
                }"#,
                )
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut record_service = RecordService::new(&mut base, "test");
        let config = RecordAuthRefreshConfig::<HashMap<String, String>> {
            body: HashMap::new(),
            query_params: Vec::new(),
            ..Default::default()
        };

        let response = record_service
            .auth_refresh::<RecordAuthResponse<Record>, HashMap<String, String>>(&config)
            .await;

        mock.assert_async().await;
        let response = response.unwrap();

        assert!(response.token == "eyJhbGciOiJIUzI1NiJ9.eyJpZCI6IjRxMXhsY2xtZmxva3UzMyIsInR5cGUiOiJhdXRoUmVjb3JkIiwiY29sbGVjdGlvbklkIjoiX3BiX3VzZXJzX2F1dGhfIiwiZXhwIjoyMjA4OTg1MjYxfQ.UwD8JvkbQtXpymT09d7J6fdA0aP9g4FJ1GPh_ggEkzc");
        assert!(response.record.base.id == "8171022dc95a4ed");
        assert!(response.record.collection_id == "d2972397d45614e");
        assert!(response.record.collection_name == "users");
        assert!(response.record.base.created == "2022-06-24 06:24:18.434Z");
        assert!(response.record.base.updated == "2022-06-24 06:24:18.889Z");
        assert!(response.record.data["username"] == "test@example.com");
        assert!(response.record.data["email"] == "test@example.com");
        assert!(response.record.data["verified"] == false);
        assert!(response.record.data["emailVisibility"] == true);
        assert!(response.record.data["someCustomField"] == "example 123");
    }

    #[tokio::test]
    async fn test_record_request_password_reset() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("POST", "/api/collections/test/request-password-reset")
            .with_status(204)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .match_body(r#"{"email":"example@example.com"}"#)
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut record_service = RecordService::new(&mut base, "test");
        let config = RecordRequestPasswordResetConfig::<HashMap<String, String>> {
            email: String::from_str("example@example.com").unwrap(),
            body: HashMap::new(),
            query_params: Vec::new(),
        };

        let response = record_service
            .request_password_reset::<HashMap<String, String>>(&config)
            .await;

        mock.assert_async().await;
        response.unwrap();
    }

    #[tokio::test]
    async fn test_record_confirm_password_reset() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("POST", "/api/collections/test/confirm-password-reset")
            .with_status(204)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .match_body(r#"{"token":"test","password":"test","passwordConfirm":"test"}"#)
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut record_service = RecordService::new(&mut base, "test");
        let config = RecordConfirmPasswordResetConfig::<HashMap<String, String>> {
            token: String::from_str("test").unwrap(),
            password: String::from_str("test").unwrap(),
            password_confirm: String::from_str("test").unwrap(),
            body: HashMap::new(),
            query_params: Vec::new(),
        };

        let response = record_service
            .confirm_password_reset::<HashMap<String, String>>(&config)
            .await;

        mock.assert_async().await;
        response.unwrap();
    }

    #[tokio::test]
    async fn test_record_request_verification() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("POST", "/api/collections/test/request-verification")
            .with_status(204)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .match_body(r#"{"email":"example@example.com"}"#)
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut record_service = RecordService::new(&mut base, "test");
        let config = RecordRequestVerificationConfig::<HashMap<String, String>> {
            email: String::from_str("example@example.com").unwrap(),
            body: HashMap::new(),
            query_params: Vec::new(),
        };

        let response = record_service
            .request_verification::<HashMap<String, String>>(&config)
            .await;

        mock.assert_async().await;
        response.unwrap();
    }

    #[tokio::test]
    async fn test_record_confirm_verification() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("POST", "/api/collections/test/confirm-verification")
            .with_status(204)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .match_body(r#"{"token":"test"}"#)
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut record_service = RecordService::new(&mut base, "test");
        let config = RecordConfirmVerificationConfig::<HashMap<String, String>> {
            token: String::from_str("test").unwrap(),
            body: HashMap::new(),
            query_params: Vec::new(),
        };

        let response = record_service
            .confirm_verification::<HashMap<String, String>>(&config)
            .await;

        mock.assert_async().await;
        response.unwrap();
    }

    #[tokio::test]
    async fn test_record_request_email_change() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("POST", "/api/collections/test/request-email-change")
            .with_status(204)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .match_body(r#"{"newEmail":"example@example.com"}"#)
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut record_service = RecordService::new(&mut base, "test");
        let config = RecordRequestEmailChangeConfig::<HashMap<String, String>> {
            new_email: String::from_str("example@example.com").unwrap(),
            body: HashMap::new(),
            query_params: Vec::new(),
        };

        let response = record_service
            .request_email_change::<HashMap<String, String>>(&config)
            .await;

        mock.assert_async().await;
        response.unwrap();
    }

    #[tokio::test]
    async fn test_record_confirm_email_change() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("POST", "/api/collections/test/confirm-email-change")
            .with_status(204)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .match_body(r#"{"token":"test","password":"test"}"#)
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut record_service = RecordService::new(&mut base, "test");
        let config = RecordConfirmEmailChangeConfig::<HashMap<String, String>> {
            token: String::from_str("test").unwrap(),
            password: String::from_str("test").unwrap(),
            body: HashMap::new(),
            query_params: Vec::new(),
        };

        let response = record_service
            .confirm_email_change::<HashMap<String, String>>(&config)
            .await;

        mock.assert_async().await;
        response.unwrap();
    }

    #[tokio::test]
    async fn test_record_list_external_auths() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("GET", "/api/collections/test/records/test/external-auths")
            .with_status(200)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .with_body(
                r#"[
               {
      "id": "8171022dc95a4e8",
      "created": "2022-09-01 10:24:18.434",
      "updated": "2022-09-01 10:24:18.889",
      "recordId": "e22581b6f1d44ea",
      "collectionId": "POWMOh0W6IoLUAI",
      "provider": "google",
      "providerId": "2da15468800514p"
    },
    {
      "id": "171022dc895a4e8",
      "created": "2022-09-01 10:24:18.434",
      "updated": "2022-09-01 10:24:18.889",
      "recordId": "e22581b6f1d44ea",
      "collectionId": "POWMOh0W6IoLUAI",
      "provider": "twitter",
      "providerId": "720688005140514"
    } ]"#,
            )
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut record_service = RecordService::new(&mut base, "test");
        let config = RecordListExternalAuthsConfig {
            id: String::from_str("test").unwrap(),
            query_params: Vec::new(),
        };

        let response = record_service
            .list_external_auths::<ExternalAuth>(&config)
            .await;

        mock.assert_async().await;
        let response = response.unwrap();
        assert_eq!(response.len(), 2);
        assert_eq!(response[0].base.id, "8171022dc95a4e8");
        assert_eq!(response[0].base.created, "2022-09-01 10:24:18.434");
        assert_eq!(response[0].base.updated, "2022-09-01 10:24:18.889");
        assert_eq!(response[0].record_id, "e22581b6f1d44ea");
        assert_eq!(response[0].collection_id, "POWMOh0W6IoLUAI");
        assert_eq!(response[0].provider, "google");
        assert_eq!(response[0].provider_id, "2da15468800514p");
    }

    #[tokio::test]
    async fn test_record_unlink_external_auth() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock(
                "DELETE",
                "/api/collections/test/records/test/external-auths/test",
            )
            .with_status(204)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut record_service = RecordService::new(&mut base, "test");
        let config = RecordUnlinkExternalAuthConfig {
            id: String::from_str("test").unwrap(),
            provider: String::from_str("test").unwrap(),
            query_params: Vec::new(),
        };

        let response = record_service.unlink_external_auth(&config).await;

        mock.assert_async().await;
        response.unwrap();
    }
}
