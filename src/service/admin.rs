use crate::service;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{error::RPocketError, model::Admin, service::auth_state::AuthPayload};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminAuthResponse {
    pub token: String,
    pub admin: Admin,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdminAuthWithPasswordConfig<T> {
    pub identity: String,
    pub password: String,
    #[serde(flatten)]
    pub body: T,
    #[serde(skip)]
    pub query_params: Vec<(String, String)>,
    #[serde(skip)]
    pub without_saving: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdminAuthRefreshConfig<T> {
    #[serde(flatten)]
    pub body: T,
    #[serde(skip)]
    pub query_params: Vec<(String, String)>,
    #[serde(skip)]
    pub without_saving: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdminRequestPasswordResetConfig<T> {
    pub email: String,
    #[serde(flatten)]
    pub body: T,
    #[serde(skip)]
    pub query_params: Vec<(String, String)>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdminConfirmPasswordResetConfig<T> {
    pub token: String,
    pub password: String,
    #[serde(rename = "passwordConfirm")]
    pub password_confirm: String,
    #[serde(flatten)]
    pub body: T,
    #[serde(skip)]
    pub query_params: Vec<(String, String)>,
}

pub struct AdminService<'a, C> {
    client: &'a mut C,
    admin_base_path: String,
}

impl<'a, C> AdminService<'a, C>
where
    C: crate::rpocket::PocketBaseClient + Sized,
{
    /// create a new AdminService.
    pub fn new(client: &'a mut C) -> Self {
        return AdminService {
            client,
            admin_base_path: "api/admins".to_string(),
        };
    }

    /// returns crud service.
    pub fn crud(&'a mut self) -> service::crud::CRUDService<'a, C> {
        return self.client.crud(&self.admin_base_path);
    }

    async fn save_auth_response<T>(
        &mut self,
        response: reqwest::Response,
    ) -> Result<T, RPocketError>
    where
        T: serde::de::DeserializeOwned,
    {
        let auth_state = self.client.auth_state();
        let auth_response = response.json::<AdminAuthResponse>().await?;

        let token = auth_response.token;
        let extra = auth_response.extra;
        let user = AuthPayload::Admin(auth_response.admin);

        auth_state.save(token.as_str(), &user).await?;

        let admin = match user {
            AuthPayload::Admin(admin) => admin,
            _ => unreachable!(),
        };

        let auth_response = AdminAuthResponse {
            token,
            admin,
            extra,
        };

        let auth_response = serde_json::to_value(&auth_response)?;
        let response = serde_json::from_value(auth_response)?;

        return Ok(response);
    }

    /// authenticate with password
    pub async fn auth_with_password<T, B>(
        &mut self,
        config: &AdminAuthWithPasswordConfig<B>,
    ) -> Result<T, RPocketError>
    where
        T: serde::de::DeserializeOwned,
        B: Serialize,
    {
        let url = self
            .client
            .base_url()
            .join(format!("{}/auth-with-password", self.admin_base_path).as_str())?;

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

    /// refreshes the current authenticated admin instance and
    pub async fn auth_refresh<T, B>(
        &mut self,
        config: &AdminAuthRefreshConfig<B>,
    ) -> Result<T, RPocketError>
    where
        T: serde::de::DeserializeOwned,
        B: Serialize,
    {
        let url = self
            .client
            .base_url()
            .join(format!("{}/auth-refresh", self.admin_base_path).as_str())?;

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

    /// ends auth admin password reset request.
    pub async fn request_password_reset<B>(
        &mut self,
        config: &AdminRequestPasswordResetConfig<B>,
    ) -> Result<(), RPocketError>
    where
        B: Serialize,
    {
        let url = self
            .client
            .base_url()
            .join(format!("{}/request-password-reset", self.admin_base_path).as_str())?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::POST, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params)
            .json(&config);

        self.client.http().send(request_builder).await?;

        return Ok(());
    }

    /// confirms auth admin password reset request.
    pub async fn confirm_password_reset<B>(
        &mut self,
        config: &AdminConfirmPasswordResetConfig<B>,
    ) -> Result<(), RPocketError>
    where
        B: Serialize,
    {
        let url = self
            .client
            .base_url()
            .join(format!("{}/confirm-password-reset", self.admin_base_path).as_str())?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::POST, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params)
            .json(&config);

        self.client.http().send(request_builder).await?;

        return Ok(());
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rpocket::{PocketBase, PocketBaseClient};
    use std::{collections::HashMap, str::FromStr};

    #[test]
    fn test_admin_crud() {
        let mut base = PocketBase::new("http://test.com", "en");
        let admin_service = AdminService::new(&mut base);

        assert!(admin_service.admin_base_path == "api/admins");
    }

    #[tokio::test]
    async fn test_admin_auth_with_password() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("POST", "/api/admins/auth-with-password")
            .with_status(200)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .match_body(r#"{"identity":"test","password":"12345678"}"#)
            .with_body(
                r#"{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6InN5d2JoZWNuaDQ2cmhtMCIsInR5cGUiOiJhZG1pbiIsImV4cCI6MjIwODk4MTYwMH0.han3_sG65zLddpcX2ic78qgy7FKecuPfOpFa8Dvi5Bg",
  "admin": {
    "id": "b6e4b08274f34e9",
    "created": "2022-06-22 07:13:09.735Z",
    "updated": "2022-06-22 07:13:09.735Z",
    "email": "test@example.com",
    "avatar": 0
  }                }"#,
                )
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut admin_service = AdminService::new(&mut base);
        let config = AdminAuthWithPasswordConfig::<HashMap<String, String>> {
            identity: String::from_str("test").unwrap(),
            password: String::from_str("12345678").unwrap(),
            body: HashMap::new(),
            query_params: Vec::new(),
            ..Default::default()
        };

        let response = admin_service
            .auth_with_password::<AdminAuthResponse, HashMap<String, String>>(&config)
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
            AuthPayload::Admin(user) => user,
            _ => unreachable!(),
        };

        assert!(auth_state_token == "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6InN5d2JoZWNuaDQ2cmhtMCIsInR5cGUiOiJhZG1pbiIsImV4cCI6MjIwODk4MTYwMH0.han3_sG65zLddpcX2ic78qgy7FKecuPfOpFa8Dvi5Bg");
        assert!(auth_record.base.id == "b6e4b08274f34e9");
        assert!(auth_record.base.created == "2022-06-22 07:13:09.735Z");
        assert!(auth_record.base.updated == "2022-06-22 07:13:09.735Z");
        assert!(auth_record.email == "test@example.com");
        assert!(auth_record.avatar == 0);

        assert!(response.token == "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6InN5d2JoZWNuaDQ2cmhtMCIsInR5cGUiOiJhZG1pbiIsImV4cCI6MjIwODk4MTYwMH0.han3_sG65zLddpcX2ic78qgy7FKecuPfOpFa8Dvi5Bg");
        assert!(response.admin.base.id == "b6e4b08274f34e9");
        assert!(response.admin.base.created == "2022-06-22 07:13:09.735Z");
        assert!(response.admin.base.updated == "2022-06-22 07:13:09.735Z");
        assert!(response.admin.email == "test@example.com");
        assert!(response.admin.avatar == 0);
    }

    #[tokio::test]
    async fn test_admin_auth_refresh() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("POST", "/api/admins/auth-refresh")
            .with_status(200)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .match_body(r#"{}"#)
            .with_body(
                r#"{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6InN5d2JoZWNuaDQ2cmhtMCIsInR5cGUiOiJhZG1pbiIsImV4cCI6MjIwODk4MTYwMH0.han3_sG65zLddpcX2ic78qgy7FKecuPfOpFa8Dvi5Bg",
  "admin": {
    "id": "b6e4b08274f34e9",
    "created": "2022-06-22 07:13:09.735Z",
    "updated": "2022-06-22 07:13:09.735Z",
    "email": "test@example.com",
    "avatar": 0
  }                }"#,
                )
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut admin_service = AdminService::new(&mut base);
        let config = AdminAuthRefreshConfig::<HashMap<String, String>> {
            body: HashMap::new(),
            query_params: Vec::new(),
            ..Default::default()
        };

        let response = admin_service
            .auth_refresh::<AdminAuthResponse, HashMap<String, String>>(&config)
            .await;

        mock.assert_async().await;
        let response = response.unwrap();

        assert!(response.token == "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6InN5d2JoZWNuaDQ2cmhtMCIsInR5cGUiOiJhZG1pbiIsImV4cCI6MjIwODk4MTYwMH0.han3_sG65zLddpcX2ic78qgy7FKecuPfOpFa8Dvi5Bg");
        assert!(response.admin.base.id == "b6e4b08274f34e9");
        assert!(response.admin.base.created == "2022-06-22 07:13:09.735Z");
        assert!(response.admin.base.updated == "2022-06-22 07:13:09.735Z");
        assert!(response.admin.email == "test@example.com");
        assert!(response.admin.avatar == 0);
    }

    #[tokio::test]
    async fn test_admin_request_password_reset() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("POST", "/api/admins/request-password-reset")
            .with_status(204)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .match_body(r#"{"email":"example@example.com"}"#)
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut admin_service = AdminService::new(&mut base);
        let config = AdminRequestPasswordResetConfig::<HashMap<String, String>> {
            email: String::from_str("example@example.com").unwrap(),
            body: HashMap::new(),
            query_params: Vec::new(),
        };

        let response = admin_service
            .request_password_reset::<HashMap<String, String>>(&config)
            .await;

        mock.assert_async().await;
        response.unwrap();
    }

    #[tokio::test]
    async fn test_admin_confirm_password_reset() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("POST", "/api/admins/confirm-password-reset")
            .with_status(204)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .match_body(r#"{"token":"test","password":"test","passwordConfirm":"test"}"#)
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut admin_service = AdminService::new(&mut base);
        let config = AdminConfirmPasswordResetConfig::<HashMap<String, String>> {
            token: String::from_str("test").unwrap(),
            password: String::from_str("test").unwrap(),
            password_confirm: String::from_str("test").unwrap(),
            body: HashMap::new(),
            query_params: Vec::new(),
        };

        let response = admin_service
            .confirm_password_reset::<HashMap<String, String>>(&config)
            .await;

        mock.assert_async().await;
        response.unwrap();
    }
}
