use crate::error::RPocketError;
use crate::model::ListResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const DEFAULT_PER_PAGE: i64 = 30;
const DEFAULT_PAGE: i64 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordGetListConfig {
    pub per_page: i64,
    pub page: i64,
    pub query_params: HashMap<String, String>,
}

impl Default for RecordGetListConfig {
    /// create a default RecordGetListConfig.
    fn default() -> Self {
        return RecordGetListConfig {
            per_page: DEFAULT_PER_PAGE,
            page: DEFAULT_PAGE,
            query_params: HashMap::new(),
        };
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordGetOneConfig {
    pub id: String,
    pub query_params: HashMap<String, String>,
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
        T: Serialize,
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
            .header("Content-Type", "application/json")
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
        T: Serialize,
    {
        let url = self
            .client
            .base_url()
            .join(format!("/collections/{}/records/{}", self.collection, config.id).as_str())
            .map_err(|e| RPocketError::UrlError(e))?;

        let mut queries: Vec<(&str, &str)> = Vec::with_capacity(config.query_params.len());

        for (key, value) in &config.query_params {
            queries.push((key, value));
        }

        let request_builder = self
            .client
            .request_builder(reqwest::Method::GET, url.as_str())
            .header("Content-Type", "application/json")
            .query(&queries);

        let response = self.send_request(request_builder).await?;

        return Ok(response
            .json::<T>()
            .await
            .map_err(|e| RPocketError::RequestError(e))?);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::Record;
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
            .match_header("Content-Type", "application/json")
            .with_body(
                r#"{
                "items": [
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
            query_params: HashMap::new(),
        };

        let response = record_service
            .get_list::<Record>(Option::Some(&config))
            .await
            .unwrap();

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
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_record_get_one() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("GET", "/collections/test/records/1")
            .with_status(200)
            .with_header("Accept-Language", "en")
            .match_header("Content-Type", "application/json")
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
            query_params: HashMap::new(),
        };

        let response = record_service.get_one::<Record>(&config).await.unwrap();

        assert!(response.base.id == "d08dfc4f4d84419");
        assert!(response.base.created == "2022-06-25 11:03:45.876");
        assert!(response.base.updated == "2022-06-25 11:03:45.876");
        assert!(response.collection_id == "a98f514eb05f454");
        assert!(response.collection_name == "posts");
        assert!(response.data["title"] == "test2");
        mock.assert_async().await;
    }
}
