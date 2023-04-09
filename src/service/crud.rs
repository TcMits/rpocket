use crate::{error::RPocketError, model::ListResult};
#[cfg(feature = "multipart")]
use reqwest::multipart;
use serde::{Deserialize, Serialize};

pub const DEFAULT_PER_PAGE: i64 = 30;
pub const DEFAULT_PAGE: i64 = 1;

/// CRUDGetListConfig is the config for the get list method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CRUDGetListConfig {
    pub per_page: i64,
    pub page: i64,
    pub query_params: Vec<(String, String)>,
}

impl Default for CRUDGetListConfig {
    /// create a default CRUDGetListConfig.
    fn default() -> Self {
        return CRUDGetListConfig {
            per_page: DEFAULT_PER_PAGE,
            page: DEFAULT_PAGE,
            query_params: Vec::new(),
        };
    }
}

/// CRUDGetOneConfig is the config for the get one method.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CRUDGetOneConfig {
    pub id: String,
    pub query_params: Vec<(String, String)>,
}

/// CRUDMutateConfig is the config for the mutate method.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CRUDMutateConfig<T> {
    #[serde(skip)]
    pub id: Option<String>,
    #[serde(flatten)]
    pub body: T,
    #[serde(skip)]
    pub query_params: Vec<(String, String)>,
}

/// CRUDDeleteConfig is the config for the delete method.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CRUDDeleteConfig {
    pub id: String,
    pub query_params: Vec<(String, String)>,
}

/// CRUDService is the service for CRUD operations.
pub struct CRUDService<'a, C> {
    client: &'a mut C,
    base_path: &'a str,
}

impl<'a, C> CRUDService<'a, C>
where
    C: crate::rpocket::PocketBaseClient + Sized,
{
    /// create a new CRUDService.
    pub fn new(client: &'a mut C, base_path: &'a str) -> Self {
        return CRUDService { client, base_path };
    }

    /// get a list of records.
    pub async fn get_list<T>(
        &mut self,
        config: &CRUDGetListConfig,
    ) -> Result<ListResult<T>, RPocketError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = self.client.base_url().join(&self.base_path)?;
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

        let response = self.client.http().send(request_builder).await?;

        return Ok(response.json::<ListResult<T>>().await?);
    }

    /// get a record.
    pub async fn get_one<T>(&mut self, config: &CRUDGetOneConfig) -> Result<T, RPocketError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = self
            .client
            .base_url()
            .join(format!("{}/{}", self.base_path, config.id).as_str())?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::GET, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params);

        let response = self.client.http().send(request_builder).await?;

        return Ok(response.json::<T>().await?);
    }

    /// mutate a record
    /// function will create a new record if config.id is None.
    /// function will update a record if config.id is Some.
    pub async fn mutate<T, B>(&mut self, config: &CRUDMutateConfig<B>) -> Result<T, RPocketError>
    where
        T: serde::de::DeserializeOwned,
        B: Serialize,
    {
        let mut method = reqwest::Method::POST;
        let mut url = self.client.base_url().join(&self.base_path)?;

        if let Some(ref id) = config.id {
            method = reqwest::Method::PATCH;
            url = self
                .client
                .base_url()
                .join(format!("{}/{}", self.base_path, id).as_str())?;
        }

        let request_builder = self
            .client
            .request_builder(method, url.as_str())
            .json(&config)
            .query(&config.query_params);

        let response = self.client.http().send(request_builder).await?;

        return Ok(response.json::<T>().await?);
    }

    /// multipart mutate a record
    /// function will create a new record if config.id is None.
    /// function will update a record if config.id is Some.
    #[cfg(feature = "multipart")]
    pub async fn multipart_mutate<T>(
        &mut self,
        config: CRUDMutateConfig<multipart::Form>,
    ) -> Result<T, RPocketError>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut method = reqwest::Method::POST;
        let mut url = self.client.base_url().join(&self.base_path)?;

        if let Some(ref id) = config.id {
            method = reqwest::Method::PATCH;
            url = self
                .client
                .base_url()
                .join(format!("{}/{}", self.base_path, id).as_str())?;
        }

        let request_builder = self
            .client
            .request_builder(method, url.as_str())
            .multipart(config.body)
            .query(&config.query_params);

        let response = self.client.http().send(request_builder).await?;

        return Ok(response.json::<T>().await?);
    }

    /// delete a record
    pub async fn delete(&mut self, config: &CRUDDeleteConfig) -> Result<(), RPocketError> {
        let url = self
            .client
            .base_url()
            .join(format!("{}/{}", self.base_path, config.id).as_str())?;

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
mod tests {
    use super::*;
    use crate::model::{BaseModel, ExpandValue, Record};
    use crate::rpocket::PocketBase;
    #[cfg(feature = "multipart")]
    use reqwest::multipart;
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
        let mut record_service = CRUDService::new(&mut base, "api/collections/test/records");
        let config = CRUDGetListConfig {
            per_page: 10,
            page: 1,
            ..Default::default()
        };

        let response = record_service.get_list::<Record>(&config).await;
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
        let mut record_service = CRUDService::new(&mut base, "api/collections/test/records");
        let config = CRUDGetOneConfig {
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
        let mut record_service = CRUDService::new(&mut base, "api/collections/test/records");
        let config = CRUDMutateConfig {
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
        let mut record_service = CRUDService::new(&mut base, "api/collections/test/records");
        let config = CRUDMutateConfig {
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
        let mut record_service = CRUDService::new(&mut base, "api/collections/test/records");
        let config = CRUDMutateConfig {
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
        let mut record_service = CRUDService::new(&mut base, "api/collections/test/records");
        let config = CRUDMutateConfig {
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
        let mut record_service = CRUDService::new(&mut base, "api/collections/test/records");
        let config = CRUDDeleteConfig {
            id: String::from("d08dfc4f4d84419"),
            query_params: Vec::new(),
        };

        let response = record_service.delete(&config).await;
        mock.assert_async().await;
        response.unwrap();
    }
}
