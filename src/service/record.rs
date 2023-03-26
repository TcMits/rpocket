use std::collections::HashMap;

use crate::error::RPocketError;
use crate::model::{ListResult, Record};
use crate::store::auth_storage::AuthPayload;
#[cfg(feature = "multipart")]
use reqwest::multipart;
use serde::{Deserialize, Serialize};

const DEFAULT_PER_PAGE: i64 = 30;
const DEFAULT_PAGE: i64 = 1;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordAuthResponse<T> {
    pub token: String,
    pub record: T,
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAuthMethod {
    pub username_password: bool,
    pub email_password: bool,
    pub auth_providers: Vec<AuthProvicderInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordGetListConfig {
    pub per_page: i64,
    pub page: i64,
    pub query_params: Vec<(String, String)>,
}

impl Default for RecordGetListConfig {
    /// create a default RecordGetListConfig.
    fn default() -> Self {
        return RecordGetListConfig {
            per_page: DEFAULT_PER_PAGE,
            page: DEFAULT_PAGE,
            query_params: Vec::new(),
        };
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordGetOneConfig {
    pub id: String,
    pub query_params: Vec<(String, String)>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordMutateConfig<T> {
    #[serde(skip)]
    pub id: Option<String>,
    #[serde(flatten)]
    pub body: T,
    #[serde(skip)]
    pub query_params: Vec<(String, String)>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordDeleteConfig {
    pub id: String,
    pub query_params: Vec<(String, String)>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordListAuthMethodsConfig {
    pub query_params: Vec<(String, String)>,
}

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordAuthRefreshConfig<T> {
    #[serde(flatten)]
    pub body: T,
    #[serde(skip)]
    pub query_params: Vec<(String, String)>,
    #[serde(skip)]
    pub without_saving: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordRequestPasswordResetConfig<T> {
    pub email: String,
    #[serde(flatten)]
    pub body: T,
    #[serde(skip)]
    pub query_params: Vec<(String, String)>,
}

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

pub struct RecordService<'a> {
    client: &'a mut dyn crate::rpocket::PocketBaseClient,
    collection: &'a str,
}

impl<'a> RecordService<'a> {
    /// create a new RecordService.
    pub fn new(client: &'a mut dyn crate::rpocket::PocketBaseClient, collection: &'a str) -> Self {
        return RecordService { client, collection };
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

    /// get a list of records.
    /// config: the config.
    pub async fn get_list<T>(
        &mut self,
        config: Option<&RecordGetListConfig>,
    ) -> Result<ListResult<T>, RPocketError>
    where
        T: serde::de::DeserializeOwned,
    {
        let default_config = &RecordGetListConfig::default();
        let config = config.unwrap_or(default_config);
        let url = self
            .client
            .base_url()
            .join(format!("/api/collections/{}/records", self.collection).as_str())
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

    /// get a record.
    /// config: the config.
    pub async fn get_one<T>(&mut self, config: &RecordGetOneConfig) -> Result<T, RPocketError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = self
            .client
            .base_url()
            .join(format!("/api/collections/{}/records/{}", self.collection, config.id).as_str())
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

    /// mutate a record
    /// function will create a new record if config.id is None.
    /// function will update a record if config.id is Some.
    pub async fn mutate<T, B>(&mut self, config: &RecordMutateConfig<B>) -> Result<T, RPocketError>
    where
        T: serde::de::DeserializeOwned,
        B: Serialize,
    {
        let mut method = reqwest::Method::POST;
        let mut url = self
            .client
            .base_url()
            .join(format!("/api/collections/{}/records", self.collection).as_str())
            .map_err(|e| RPocketError::UrlError(e))?;

        if let Some(ref id) = config.id {
            method = reqwest::Method::PATCH;
            url = self
                .client
                .base_url()
                .join(format!("/api/collections/{}/records/{}", self.collection, id).as_str())
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

    /// multipart mutate a record
    /// function will create a new record if config.id is None.
    /// function will update a record if config.id is Some.
    #[cfg(feature = "multipart")]
    pub async fn multipart_mutate<T>(
        &mut self,
        config: RecordMutateConfig<multipart::Form>,
    ) -> Result<T, RPocketError>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut method = reqwest::Method::POST;
        let mut url = self
            .client
            .base_url()
            .join(format!("/api/collections/{}/records", self.collection).as_str())
            .map_err(|e| RPocketError::UrlError(e))?;

        if let Some(ref id) = config.id {
            method = reqwest::Method::PATCH;
            url = self
                .client
                .base_url()
                .join(format!("/api/collections/{}/records/{}", self.collection, id).as_str())
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

    /// delete a record
    /// config: the config.
    pub async fn delete(&mut self, config: &RecordDeleteConfig) -> Result<(), RPocketError> {
        let url = self
            .client
            .base_url()
            .join(format!("/api/collections/{}/records/{}", self.collection, config.id).as_str())
            .map_err(|e| RPocketError::UrlError(e))?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::DELETE, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params);

        self.send_request(request_builder).await?;
        return Ok(());
    }

    /// list auth methods
    /// config: the config.
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
            .join(format!("/api/collections/{}/auth-methods", self.collection).as_str())
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

    async fn save_auth_response<T>(&self, response: reqwest::Response) -> Result<T, RPocketError>
    where
        T: serde::de::DeserializeOwned,
    {
        let auth_state = self.client.auth_state();
        let auth_response = response
            .json::<RecordAuthResponse<Record>>()
            .await
            .map_err(|e| RPocketError::RequestError(e))?;

        let token = auth_response.token;
        let meta = auth_response.meta;
        let user = AuthPayload::User(auth_response.record);

        auth_state.save(token.as_str(), &user).await?;

        let record = match user {
            AuthPayload::User(user) => user,
            _ => unreachable!(),
        };

        let auth_response = RecordAuthResponse {
            token,
            record,
            meta,
        };

        let auth_response =
            serde_json::to_value(&auth_response).map_err(|e| RPocketError::SerdeError(e))?;

        return serde_json::from_value(auth_response).map_err(|e| RPocketError::SerdeError(e));
    }

    /// authenticate with password
    /// config: the config.
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
            .join(format!("/api/collections/{}/auth-with-password", self.collection).as_str())
            .map_err(|e| RPocketError::UrlError(e))?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::POST, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
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

    /// authenticate with oauth2
    /// config: the config.
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
            .join(format!("/api/collections/{}/auth-with-oauth2", self.collection).as_str())
            .map_err(|e| RPocketError::UrlError(e))?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::POST, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
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

    /// refreshes the current authenticated record instance and
    /// config: the config
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
            .join(format!("/api/collections/{}/auth-refresh", self.collection).as_str())
            .map_err(|e| RPocketError::UrlError(e))?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::POST, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
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

    /// ends auth record password reset request.
    /// config: the config.
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
            .join(
                format!(
                    "/api/collections/{}/request-password-reset",
                    self.collection
                )
                .as_str(),
            )
            .map_err(|e| RPocketError::UrlError(e))?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::POST, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .json(&config);

        self.send_request(request_builder).await?;
        return Ok(());
    }

    /// confirms auth record password reset request.
    /// config: the config.
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
            .join(
                format!(
                    "/api/collections/{}/confirm-password-reset",
                    self.collection
                )
                .as_str(),
            )
            .map_err(|e| RPocketError::UrlError(e))?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::POST, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .json(&config);

        self.send_request(request_builder).await?;
        return Ok(());
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::{BaseModel, ExpandValue, Record};
    use crate::rpocket::PocketBase;
    use std::collections::HashMap;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_record_get_list() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("GET", "/api/collections/test/records?perPage=10&page=1")
            .with_status(200)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .with_body(
                r#"{ "items": [
                    {
                      "id": "d08dfc4f4d84419",
                      "collectionId": "a98f514eb05f454",
                      "collectionName": "posts",
                      "updated": "2022-06-25 11:03:45.876",
                      "created": "2022-06-25 11:03:45.876",
                      "title": "test2",
                      "expand": {
                            "user": {
                                "id": "FtHAW9feB5rze7D",
                                "collectionId": "srmAo0hLxEqYF7F",
                                "collectionName": "users",
                                "created": "2022-01-01 00:00:00.000Z",
                                "updated": "2022-01-01 00:00:00.000Z",
                                "username": "users54126",
                                "verified": false,
                                "emailVisibility": false,
                                "name": "John Doe"
                            },
                            "relatedPeople": [
                            {
                                "id": "FtHAW9feB5rze7D",
                                "collectionId": "srmAo0hLxEqYF7F",
                                "collectionName": "users",
                                "created": "2022-01-01 00:00:00.000Z",
                                "updated": "2022-01-01 00:00:00.000Z",
                                "username": "users54126",
                                "verified": false,
                                "emailVisibility": false,
                                "name": "John Doe"
                            }
                            ]
                        }

                    }
                ],
                "totalItems": 1,
                "page": 1,
                "perPage": 10
            }"#,
            )
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut record_service = RecordService::new(&mut base, "test");
        let config = RecordGetListConfig {
            per_page: 10,
            page: 1,
            ..Default::default()
        };

        let response = record_service
            .get_list::<Record>(Option::Some(&config))
            .await;
        mock.assert_async().await;
        let response = response.unwrap();

        assert!(response.items.len() == 1);
        assert!(response.total_items == 1);
        assert!(response.page == 1);
        assert!(response.per_page == 10);
        assert!(response.items[0].base.id == "d08dfc4f4d84419");
        assert!(response.items[0].base.created == "2022-06-25 11:03:45.876");
        assert!(response.items[0].base.updated == "2022-06-25 11:03:45.876");
        assert!(response.items[0].collection_id == "a98f514eb05f454");
        assert!(response.items[0].collection_name == "posts");
        assert!(response.items[0].data["title"] == "test2");

        let expand_records = response.items[0].expand.as_ref().unwrap();

        match expand_records["user"] {
            ExpandValue::Record(ref user) => {
                assert!(user.base.id == "FtHAW9feB5rze7D");
                assert!(user.base.created == "2022-01-01 00:00:00.000Z");
                assert!(user.base.updated == "2022-01-01 00:00:00.000Z");
                assert!(user.collection_id == "srmAo0hLxEqYF7F");
                assert!(user.collection_name == "users");
                assert!(user.data["username"] == "users54126");
                assert!(user.data["verified"] == false);
                assert!(user.data["emailVisibility"] == false);
                assert!(user.data["name"] == "John Doe");
            }
            _ => {
                panic!("ExpandValue is not Record");
            }
        }

        match expand_records["relatedPeople"] {
            ExpandValue::ListRecords(ref related_people) => {
                assert!(related_people.len() == 1);
                assert!(related_people[0].base.id == "FtHAW9feB5rze7D");
                assert!(related_people[0].base.created == "2022-01-01 00:00:00.000Z");
                assert!(related_people[0].base.updated == "2022-01-01 00:00:00.000Z");
                assert!(related_people[0].collection_id == "srmAo0hLxEqYF7F");
                assert!(related_people[0].collection_name == "users");
                assert!(related_people[0].data["username"] == "users54126");
                assert!(related_people[0].data["verified"] == false);
                assert!(related_people[0].data["emailVisibility"] == false);
                assert!(related_people[0].data["name"] == "John Doe");
            }
            _ => {
                panic!("ExpandValue is not a list of records");
            }
        }
    }

    #[tokio::test]
    async fn test_record_get_one() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("GET", "/api/collections/test/records/1")
            .with_status(200)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .with_body(
                r#"{
                "id": "d08dfc4f4d84419",
                "collectionId": "a98f514eb05f454",
                "collectionName": "posts",
                "updated": "2022-06-25 11:03:45.876",
                "created": "2022-06-25 11:03:45.876",
                "title": "test2"
            }"#,
            )
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut record_service = RecordService::new(&mut base, "test");
        let config = RecordGetOneConfig {
            id: String::from_str("1").unwrap(),
            ..Default::default()
        };

        let response = record_service.get_one::<Record>(&config).await;
        mock.assert_async().await;
        let response = response.unwrap();

        assert!(response.base.id == "d08dfc4f4d84419");
        assert!(response.base.created == "2022-06-25 11:03:45.876");
        assert!(response.base.updated == "2022-06-25 11:03:45.876");
        assert!(response.collection_id == "a98f514eb05f454");
        assert!(response.collection_name == "posts");
        assert!(response.data["title"] == "test2");
    }

    #[tokio::test]
    async fn test_record_mutate_create() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("POST", "/api/collections/test/records")
            .with_status(201)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .match_body(
                r#"{"id":"d08dfc4f4d84419","created":"2022-06-25 11:03:45.876","updated":"2022-06-25 11:03:45.876","collectionId":"a98f514eb05f454","collectionName":"posts","title":"test2","expand":null}"#,
            )
            .with_body(
                r#"{
                "id": "d08dfc4f4d84419",
                "collectionId": "a98f514eb05f454",
                "collectionName": "posts",
                "updated": "2022-06-25 11:03:45.876",
                "created": "2022-06-25 11:03:45.876",
                "title": "test2"
            }"#,
            )
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut record_service = RecordService::new(&mut base, "test");
        let config = RecordMutateConfig {
            id: None,
            body: Record {
                base: BaseModel {
                    id: String::from_str("d08dfc4f4d84419").unwrap(),
                    created: String::from_str("2022-06-25 11:03:45.876").unwrap(),
                    updated: String::from_str("2022-06-25 11:03:45.876").unwrap(),
                },
                collection_id: String::from_str("a98f514eb05f454").unwrap(),
                collection_name: String::from_str("posts").unwrap(),
                data: HashMap::from([(
                    String::from_str("title").unwrap(),
                    serde_json::Value::String(String::from_str("test2").unwrap()),
                )]),
                expand: Option::None,
            },
            query_params: Vec::new(),
        };

        let response = record_service.mutate::<Record, Record>(&config).await;
        mock.assert_async().await;
        let response = response.unwrap();

        assert!(response.base.id == "d08dfc4f4d84419");
        assert!(response.base.created == "2022-06-25 11:03:45.876");
        assert!(response.base.updated == "2022-06-25 11:03:45.876");
        assert!(response.collection_id == "a98f514eb05f454");
        assert!(response.collection_name == "posts");
        assert!(response.data["title"] == "test2");
    }

    #[tokio::test]
    async fn test_record_mutate_update() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("PATCH", "/api/collections/test/records/d08dfc4f4d84419")
            .with_status(200)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .match_body(
                r#"{"id":"d08dfc4f4d84419","created":"2022-06-25 11:03:45.876","updated":"2022-06-25 11:03:45.876","collectionId":"a98f514eb05f454","collectionName":"posts","title":"test2","expand":null}"#,
            )
            .with_body(
                r#"{
                "id": "d08dfc4f4d84419",
                "collectionId": "a98f514eb05f454",
                "collectionName": "posts",
                "updated": "2022-06-25 11:03:45.876",
                "created": "2022-06-25 11:03:45.876",
                "title": "test2"
            }"#,
            )
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut record_service = RecordService::new(&mut base, "test");
        let config = RecordMutateConfig {
            id: Some(String::from_str("d08dfc4f4d84419").unwrap()),
            body: Record {
                base: BaseModel {
                    id: String::from_str("d08dfc4f4d84419").unwrap(),
                    created: String::from_str("2022-06-25 11:03:45.876").unwrap(),
                    updated: String::from_str("2022-06-25 11:03:45.876").unwrap(),
                },
                collection_id: String::from_str("a98f514eb05f454").unwrap(),
                collection_name: String::from_str("posts").unwrap(),
                data: HashMap::from([(
                    String::from_str("title").unwrap(),
                    serde_json::Value::String(String::from_str("test2").unwrap()),
                )]),
                expand: Option::None,
            },
            query_params: Vec::new(),
        };

        let response = record_service.mutate::<Record, Record>(&config).await;
        mock.assert_async().await;
        let response = response.unwrap();

        assert!(response.base.id == "d08dfc4f4d84419");
        assert!(response.base.created == "2022-06-25 11:03:45.876");
        assert!(response.base.updated == "2022-06-25 11:03:45.876");
        assert!(response.collection_id == "a98f514eb05f454");
        assert!(response.collection_name == "posts");
        assert!(response.data["title"] == "test2");
    }

    #[tokio::test]
    #[cfg(feature = "multipart")]
    async fn test_record_multipart_mutate_create() {
        let mut server = mockito::Server::new();
        let url = server.url();
        let body = multipart::Form::default()
            .text("id", "d08dfc4f4d84419")
            .text("created", "2022-06-25 11:03:45.876")
            .text("updated", "2022-06-25 11:03:45.876")
            .text("collectionId", "a98f514eb05f454")
            .text("collectionName", "posts")
            .text("title", "test2");

        let mock = server
            .mock("POST", "/api/collections/test/records")
            .with_status(201)
            .with_header("Accept-Language", "en")
            .match_header(
                reqwest::header::CONTENT_TYPE.as_str(),
                format!("multipart/form-data; boundary={}", body.boundary()).as_str(),
            )
            .with_body(
                r#"{
                "id": "d08dfc4f4d84419",
                "collectionId": "a98f514eb05f454",
                "collectionName": "posts",
                "updated": "2022-06-25 11:03:45.876",
                "created": "2022-06-25 11:03:45.876",
                "title": "test2"
            }"#,
            )
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut record_service = RecordService::new(&mut base, "test");
        let config = RecordMutateConfig {
            id: None,
            body,
            query_params: Vec::new(),
        };

        let response = record_service.multipart_mutate::<Record>(config).await;
        mock.assert_async().await;
        let response = response.unwrap();

        assert!(response.base.id == "d08dfc4f4d84419");
        assert!(response.base.created == "2022-06-25 11:03:45.876");
        assert!(response.base.updated == "2022-06-25 11:03:45.876");
        assert!(response.collection_id == "a98f514eb05f454");
        assert!(response.collection_name == "posts");
        assert!(response.data["title"] == "test2");
    }

    #[tokio::test]
    #[cfg(feature = "multipart")]
    async fn test_record_multipart_mutate_update() {
        let mut server = mockito::Server::new();
        let url = server.url();
        let body = multipart::Form::default()
            .text("id", "d08dfc4f4d84419")
            .text("created", "2022-06-25 11:03:45.876")
            .text("updated", "2022-06-25 11:03:45.876")
            .text("collectionId", "a98f514eb05f454")
            .text("collectionName", "posts")
            .text("title", "test2");

        let mock = server
            .mock("PATCH", "/api/collections/test/records/d08dfc4f4d84419")
            .with_status(200)
            .with_header("Accept-Language", "en")
            .match_header(
                reqwest::header::CONTENT_TYPE.as_str(),
                format!("multipart/form-data; boundary={}", body.boundary()).as_str(),
            )
            .with_body(
                r#"{
                "id": "d08dfc4f4d84419",
                "collectionId": "a98f514eb05f454",
                "collectionName": "posts",
                "updated": "2022-06-25 11:03:45.876",
                "created": "2022-06-25 11:03:45.876",
                "title": "test2"
            }"#,
            )
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut record_service = RecordService::new(&mut base, "test");
        let config = RecordMutateConfig {
            id: Some(String::from("d08dfc4f4d84419")),
            body,
            query_params: Vec::new(),
        };

        let response = record_service.multipart_mutate::<Record>(config).await;
        mock.assert_async().await;
        let response = response.unwrap();

        assert!(response.base.id == "d08dfc4f4d84419");
        assert!(response.base.created == "2022-06-25 11:03:45.876");
        assert!(response.base.updated == "2022-06-25 11:03:45.876");
        assert!(response.collection_id == "a98f514eb05f454");
        assert!(response.collection_name == "posts");
        assert!(response.data["title"] == "test2");
    }

    #[tokio::test]
    async fn test_record_delete() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("DELETE", "/api/collections/test/records/d08dfc4f4d84419")
            .with_status(204)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut record_service = RecordService::new(&mut base, "test");
        let config = RecordDeleteConfig {
            id: String::from("d08dfc4f4d84419"),
            query_params: Vec::new(),
        };

        let response = record_service.delete(&config).await;
        mock.assert_async().await;
        response.unwrap();
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
}
