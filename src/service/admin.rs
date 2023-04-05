#[cfg(feature = "multipart")]
use reqwest::multipart;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    error::RPocketError,
    model::{Admin, ListResult},
    store::auth_storage::AuthPayload,
};

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdminGetListConfig {
    pub per_page: i64,
    pub page: i64,
    pub query_params: Vec<(String, String)>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdminGetOneConfig {
    pub id: String,
    pub query_params: Vec<(String, String)>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdminMutateConfig<T> {
    #[serde(skip)]
    pub id: Option<String>,
    #[serde(flatten)]
    pub body: T,
    #[serde(skip)]
    pub query_params: Vec<(String, String)>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdminDeleteConfig {
    pub id: String,
    pub query_params: Vec<(String, String)>,
}

pub struct AdminService<'a> {
    client: &'a mut dyn crate::rpocket::PocketBaseClient,
}

impl<'a> AdminService<'a> {
    /// create a new AdminService.
    pub fn new(client: &'a mut dyn crate::rpocket::PocketBaseClient) -> Self {
        return AdminService { client };
    }

    /// send a request.
    async fn send_request(
        &mut self,
        request_builder: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, RPocketError> {
        let pb_request = crate::rpocket::PocketBaseRequest::HTTP { request_builder };
        return self.client.send(pb_request).await.map(|pb_response| {
            return match pb_response {
                crate::rpocket::PocketBaseResponse::HTTP { response } => response,
            };
        });
    }

    async fn save_auth_response<T>(&self, response: reqwest::Response) -> Result<T, RPocketError>
    where
        T: serde::de::DeserializeOwned,
    {
        let auth_state = self.client.auth_state();
        let auth_response = response
            .json::<AdminAuthResponse>()
            .await
            .map_err(|e| RPocketError::RequestError(e))?;

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

        let auth_response =
            serde_json::to_value(&auth_response).map_err(|e| RPocketError::SerdeError(e))?;

        return serde_json::from_value(auth_response).map_err(|e| RPocketError::SerdeError(e));
    }

    /// authenticate with password
    /// config: the config.
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
            .join("/api/admins/auth-with-password")
            .map_err(|e| RPocketError::UrlError(e))?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::POST, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params)
            .json(&config);

        let response = self.send_request(request_builder).await?;

        if !config.without_saving {
            return self.save_auth_response::<T>(response).await;
        }

        return Ok(response
            .json::<T>()
            .await
            .map_err(|e| RPocketError::RequestError(e))?);
    }

    /// refreshes the current authenticated admin instance and
    /// config: the config
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
            .join("/api/admins/auth-refresh")
            .map_err(|e| RPocketError::UrlError(e))?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::POST, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params)
            .json(&config);

        let response = self.send_request(request_builder).await?;

        if !config.without_saving {
            return self.save_auth_response::<T>(response).await;
        }

        return Ok(response
            .json::<T>()
            .await
            .map_err(|e| RPocketError::RequestError(e))?);
    }

    /// ends auth admin password reset request.
    /// config: the config.
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
            .join("/api/admins/request-password-reset")
            .map_err(|e| RPocketError::UrlError(e))?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::POST, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params)
            .json(&config);

        self.send_request(request_builder).await?;
        return Ok(());
    }

    /// confirms auth admin password reset request.
    /// config: the config.
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
            .join("/api/admins/confirm-password-reset")
            .map_err(|e| RPocketError::UrlError(e))?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::POST, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params)
            .json(&config);

        self.send_request(request_builder).await?;
        return Ok(());
    }

    /// get a list of admins.
    /// config: the config.
    pub async fn get_list<T>(
        &mut self,
        config: &AdminGetListConfig,
    ) -> Result<ListResult<T>, RPocketError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = self
            .client
            .base_url()
            .join("/api/admins")
            .map_err(|e| RPocketError::UrlError(e))?;
        let mut queries: Vec<(&str, &str)> = Vec::with_capacity(2 + config.query_params.len());
        let per_page = &config.per_page.to_string();
        let page = &config.page.to_string();

        queries.push(("perPage", per_page));
        queries.push(("page", page));

        for (key, value) in &config.query_params {
            queries.push((key, value));
        }

        let request_builder = self
            .client
            .request_builder(reqwest::Method::GET, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&queries);

        let response = self.send_request(request_builder).await?;

        return Ok(response
            .json::<ListResult<T>>()
            .await
            .map_err(|e| RPocketError::RequestError(e))?);
    }

    /// get a admin.
    /// config: the config.
    pub async fn get_one<T>(&mut self, config: &AdminGetOneConfig) -> Result<T, RPocketError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = self
            .client
            .base_url()
            .join(format!("/api/admins/{}", config.id).as_str())
            .map_err(|e| RPocketError::UrlError(e))?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::GET, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params);

        let response = self.send_request(request_builder).await?;

        return Ok(response
            .json::<T>()
            .await
            .map_err(|e| RPocketError::RequestError(e))?);
    }

    /// mutate a admin
    /// function will create a new admin if config.id is None.
    /// function will update a admin if config.id is Some.
    pub async fn mutate<T, B>(&mut self, config: &AdminMutateConfig<B>) -> Result<T, RPocketError>
    where
        T: serde::de::DeserializeOwned,
        B: Serialize,
    {
        let mut method = reqwest::Method::POST;
        let mut url = self
            .client
            .base_url()
            .join(format!("/api/admins").as_str())
            .map_err(|e| RPocketError::UrlError(e))?;

        if let Some(ref id) = config.id {
            method = reqwest::Method::PATCH;
            url = self
                .client
                .base_url()
                .join(format!("/api/admins/{}", id).as_str())
                .map_err(|e| RPocketError::UrlError(e))?;
        }

        let request_builder = self
            .client
            .request_builder(method, url.as_str())
            .json(&config)
            .query(&config.query_params);

        let response = self.send_request(request_builder).await?;

        return Ok(response
            .json::<T>()
            .await
            .map_err(|e| RPocketError::RequestError(e))?);
    }

    /// multipart mutate a admin
    /// function will create a new admin if config.id is None.
    /// function will update a admin if config.id is Some.
    #[cfg(feature = "multipart")]
    pub async fn multipart_mutate<T>(
        &mut self,
        config: AdminMutateConfig<multipart::Form>,
    ) -> Result<T, RPocketError>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut method = reqwest::Method::POST;
        let mut url = self
            .client
            .base_url()
            .join(format!("/api/admins").as_str())
            .map_err(|e| RPocketError::UrlError(e))?;

        if let Some(ref id) = config.id {
            method = reqwest::Method::PATCH;
            url = self
                .client
                .base_url()
                .join(format!("/api/admins/{}", id).as_str())
                .map_err(|e| RPocketError::UrlError(e))?;
        }

        let request_builder = self
            .client
            .request_builder(method, url.as_str())
            .multipart(config.body)
            .query(&config.query_params);

        let response = self.send_request(request_builder).await?;

        return Ok(response
            .json::<T>()
            .await
            .map_err(|e| RPocketError::RequestError(e))?);
    }

    /// delete a admin
    /// config: the config.
    pub async fn delete(&mut self, config: &AdminDeleteConfig) -> Result<(), RPocketError> {
        let url = self
            .client
            .base_url()
            .join(format!("/api/admins/{}", config.id).as_str())
            .map_err(|e| RPocketError::UrlError(e))?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::DELETE, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params);

        self.send_request(request_builder).await?;
        return Ok(());
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        model::BaseModel,
        rpocket::{PocketBase, PocketBaseClient},
    };
    use std::{collections::HashMap, str::FromStr};

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

        let auth_state_token = base.auth_state().token().await.unwrap().unwrap();
        let auth_record = match base.auth_state().user_or_admin().await.unwrap().unwrap() {
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

    #[tokio::test]
    async fn test_admin_get_list() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("GET", "/api/admins?perPage=10&page=1")
            .with_status(200)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .with_body(
                r#"{
  "page": 1,
  "perPage": 10,
  "totalItems": 2,
  "items": [
    {
      "id": "b6e4b08274f34e9",
      "created": "2022-06-22 07:13:09.735Z",
      "updated": "2022-06-22 07:15:09.735Z",
      "email": "test@example.com",
      "avatar": 0
    },
    {
      "id": "e99c3f2aff6d695",
      "created": "2022-06-25 16:14:23.037Z",
      "updated": "2022-06-25 16:14:27.495Z",
      "email": "test2@example.com",
      "avatar": 6
    }
  ]
                }"#,
            )
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut admin_service = AdminService::new(&mut base);
        let config = AdminGetListConfig {
            per_page: 10,
            page: 1,
            ..Default::default()
        };

        let response = admin_service.get_list::<Admin>(&config).await;

        mock.assert_async().await;
        let response = response.unwrap();

        assert!(response.items.len() == 2);
        assert!(response.total_items == 2);
        assert!(response.page == 1);
        assert!(response.per_page == 10);
        assert!(response.items[0].base.id == "b6e4b08274f34e9");
        assert!(response.items[0].base.created == "2022-06-22 07:13:09.735Z");
        assert!(response.items[0].base.updated == "2022-06-22 07:15:09.735Z");
        assert!(response.items[0].email == "test@example.com");
        assert!(response.items[0].avatar == 0);
    }

    #[tokio::test]
    async fn test_admin_get_one() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("GET", "/api/admins/b6e4b08274f34e9")
            .with_status(200)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .with_body(
                r#"{
      "id": "b6e4b08274f34e9",
      "created": "2022-06-22 07:13:09.735Z",
      "updated": "2022-06-22 07:15:09.735Z",
      "email": "test@example.com",
      "avatar": 0
                }"#,
            )
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut admin_service = AdminService::new(&mut base);
        let config = AdminGetOneConfig {
            id: String::from_str("b6e4b08274f34e9").unwrap(),
            ..Default::default()
        };

        let response = admin_service.get_one::<Admin>(&config).await;

        mock.assert_async().await;
        let response = response.unwrap();

        assert!(response.base.id == "b6e4b08274f34e9");
        assert!(response.base.created == "2022-06-22 07:13:09.735Z");
        assert!(response.base.updated == "2022-06-22 07:15:09.735Z");
        assert!(response.email == "test@example.com");
        assert!(response.avatar == 0);
    }

    #[tokio::test]
    async fn test_admin_mutate_create() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("POST", "/api/admins")
            .with_status(201)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .match_body(
                r#"{"id":"b6e4b08274f34e9","created":"2022-06-22 07:13:09.735Z","updated":"2022-06-22 07:15:09.735Z","avatar":8,"email":"new@example.com"}"#,
            )
            .with_body(
                r#"{
  "id": "b6e4b08274f34e9",
  "created": "2022-06-22 07:13:09.735Z",
  "updated": "2022-06-22 07:15:09.735Z",
  "email": "new@example.com",
  "avatar": 8
                }"#)
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut admin_service = AdminService::new(&mut base);
        let config = AdminMutateConfig {
            id: None,
            body: Admin {
                base: BaseModel {
                    id: String::from_str("b6e4b08274f34e9").unwrap(),
                    created: String::from_str("2022-06-22 07:13:09.735Z").unwrap(),
                    updated: String::from_str("2022-06-22 07:15:09.735Z").unwrap(),
                },
                avatar: 8,
                email: String::from_str("new@example.com").unwrap(),
            },
            query_params: Vec::new(),
        };

        let response = admin_service.mutate::<Admin, Admin>(&config).await;
        mock.assert_async().await;
        let response = response.unwrap();

        assert!(response.base.id == "b6e4b08274f34e9");
        assert!(response.base.created == "2022-06-22 07:13:09.735Z");
        assert!(response.base.updated == "2022-06-22 07:15:09.735Z");
        assert!(response.email == "new@example.com");
        assert!(response.avatar == 8);
    }

    #[tokio::test]
    async fn test_admin_mutate_update() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("PATCH", "/api/admins/b6e4b08274f34e9")
            .with_status(200)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .match_body(
                r#"{"id":"b6e4b08274f34e9","created":"2022-06-22 07:13:09.735Z","updated":"2022-06-22 07:15:09.735Z","avatar":8,"email":"new@example.com"}"#,
            )
            .with_body(
                r#"{
  "id": "b6e4b08274f34e9",
  "created": "2022-06-22 07:13:09.735Z",
  "updated": "2022-06-22 07:15:09.735Z",
  "email": "new@example.com",
  "avatar": 8
                }"#)
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut admin_service = AdminService::new(&mut base);
        let config = AdminMutateConfig {
            id: Some(String::from_str("b6e4b08274f34e9").unwrap()),
            body: Admin {
                base: BaseModel {
                    id: String::from_str("b6e4b08274f34e9").unwrap(),
                    created: String::from_str("2022-06-22 07:13:09.735Z").unwrap(),
                    updated: String::from_str("2022-06-22 07:15:09.735Z").unwrap(),
                },
                avatar: 8,
                email: String::from_str("new@example.com").unwrap(),
            },
            query_params: Vec::new(),
        };

        let response = admin_service.mutate::<Admin, Admin>(&config).await;
        mock.assert_async().await;
        let response = response.unwrap();

        assert!(response.base.id == "b6e4b08274f34e9");
        assert!(response.base.created == "2022-06-22 07:13:09.735Z");
        assert!(response.base.updated == "2022-06-22 07:15:09.735Z");
        assert!(response.email == "new@example.com");
        assert!(response.avatar == 8);
    }

    #[tokio::test]
    #[cfg(feature = "multipart")]
    async fn test_admin_multipart_mutate_create() {
        let mut server = mockito::Server::new();
        let url = server.url();
        let body = multipart::Form::default()
            .text("id", "d08dfc4f4d84419")
            .text("created", "2022-06-25 11:03:45.876")
            .text("updated", "2022-06-25 11:03:45.876")
            .text("email", "new@example.com")
            .text("avatar", "8");

        let mock = server
            .mock("POST", "/api/admins")
            .with_status(201)
            .with_header("Accept-Language", "en")
            .match_header(
                reqwest::header::CONTENT_TYPE.as_str(),
                format!("multipart/form-data; boundary={}", body.boundary()).as_str(),
            )
            .with_body(
                r#"{
  "id": "b6e4b08274f34e9",
  "created": "2022-06-22 07:13:09.735Z",
  "updated": "2022-06-22 07:15:09.735Z",
  "email": "new@example.com",
  "avatar": 8
                }"#,
            )
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut admin_service = AdminService::new(&mut base);
        let config = AdminMutateConfig {
            id: None,
            body,
            query_params: Vec::new(),
        };

        let response = admin_service.multipart_mutate::<Admin>(config).await;
        mock.assert_async().await;
        let response = response.unwrap();

        assert!(response.base.id == "b6e4b08274f34e9");
        assert!(response.base.created == "2022-06-22 07:13:09.735Z");
        assert!(response.base.updated == "2022-06-22 07:15:09.735Z");
        assert!(response.email == "new@example.com");
        assert!(response.avatar == 8);
    }

    #[tokio::test]
    async fn test_admin_multipart_mutate_update() {
        let mut server = mockito::Server::new();
        let url = server.url();
        let body = multipart::Form::default()
            .text("id", "d08dfc4f4d84419")
            .text("created", "2022-06-25 11:03:45.876")
            .text("updated", "2022-06-25 11:03:45.876")
            .text("email", "new@example.com")
            .text("avatar", "8");

        let mock = server
            .mock("PATCH", "/api/admins/b6e4b08274f34e9")
            .with_status(200)
            .with_header("Accept-Language", "en")
            .match_header(
                reqwest::header::CONTENT_TYPE.as_str(),
                format!("multipart/form-data; boundary={}", body.boundary()).as_str(),
            )
            .with_body(
                r#"{
  "id": "b6e4b08274f34e9",
  "created": "2022-06-22 07:13:09.735Z",
  "updated": "2022-06-22 07:15:09.735Z",
  "email": "new@example.com",
  "avatar": 8
                }"#,
            )
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut admin_service = AdminService::new(&mut base);
        let config = AdminMutateConfig {
            id: Some(String::from_str("b6e4b08274f34e9").unwrap()),
            body,
            query_params: Vec::new(),
        };

        let response = admin_service.multipart_mutate::<Admin>(config).await;
        mock.assert_async().await;
        let response = response.unwrap();

        assert!(response.base.id == "b6e4b08274f34e9");
        assert!(response.base.created == "2022-06-22 07:13:09.735Z");
        assert!(response.base.updated == "2022-06-22 07:15:09.735Z");
        assert!(response.email == "new@example.com");
        assert!(response.avatar == 8);
    }

    #[tokio::test]
    async fn test_admin_delete() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("DELETE", "/api/admins/b6e4b08274f34e9")
            .with_status(204)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut admin_service = AdminService::new(&mut base);
        let config = AdminDeleteConfig {
            id: String::from_str("b6e4b08274f34e9").unwrap(),
            query_params: Vec::new(),
        };

        let response = admin_service.delete(&config).await;
        mock.assert_async().await;
        response.unwrap();
    }
}
