use crate::error::RPocketError;
use crate::model::ListResult;
#[cfg(feature = "multipart")]
use reqwest::multipart;
use serde::{Deserialize, Serialize};

const DEFAULT_PER_PAGE: i64 = 30;
const DEFAULT_PAGE: i64 = 1;

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

#[derive(Debug, Clone)]
pub struct RecordMutateConfig<T> {
    pub id: Option<String>,
    pub body: T,
    pub query_params: Vec<(String, String)>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordDeleteConfig {
    pub id: String,
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
    /// return: the list of records.
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
            .join(format!("/collections/{}/records", self.collection).as_str())
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
            .join(format!("/collections/{}/records/{}", self.collection, config.id).as_str())
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
            .join(format!("/collections/{}/records", self.collection).as_str())
            .map_err(|e| RPocketError::UrlError(e))?;

        if let Some(ref id) = config.id {
            method = reqwest::Method::PATCH;
            url = self
                .client
                .base_url()
                .join(format!("/collections/{}/records/{}", self.collection, id).as_str())
                .map_err(|e| RPocketError::UrlError(e))?;
        }

        let request_builder = self
            .client
            .request_builder(method, url.as_str())
            .json(&config.body)
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
            .join(format!("/collections/{}/records", self.collection).as_str())
            .map_err(|e| RPocketError::UrlError(e))?;

        if let Some(ref id) = config.id {
            method = reqwest::Method::PATCH;
            url = self
                .client
                .base_url()
                .join(format!("/collections/{}/records/{}", self.collection, id).as_str())
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
            .join(format!("/collections/{}/records/{}", self.collection, config.id).as_str())
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
    use crate::model::{BaseModel, Record};
    use crate::rpocket::PocketBase;
    use std::collections::HashMap;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_record_get_list() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("GET", "/collections/test/records?perPage=10&page=1")
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
                      "title": "test2"
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
    }

    #[tokio::test]
    async fn test_record_get_one() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("GET", "/collections/test/records/1")
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
            .mock("POST", "/collections/test/records")
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
            .mock("PATCH", "/collections/test/records/d08dfc4f4d84419")
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
            .mock("POST", "/collections/test/records")
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
            .mock("PATCH", "/collections/test/records/d08dfc4f4d84419")
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
            .mock("DELETE", "/collections/test/records/d08dfc4f4d84419")
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
}
