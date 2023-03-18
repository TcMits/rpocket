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

pub struct RecordService<'a> {
    client: &'a dyn crate::rpocket::PocketBaseClient,
    collection: &'a str,
}

impl<'a> RecordService<'a> {
    /// create a new RecordService.
    pub fn new(client: &'a dyn crate::rpocket::PocketBaseClient, collection: &'a str) -> Self {
        return RecordService { client, collection };
    }

    /// get a list of records.
    /// config: the config.
    /// return: the list of records.
    pub async fn get_list<T>(
        &self,
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

        let response = self.client.send_request(request_builder).await?;

        return Ok(response
            .json::<ListResult<T>>()
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

    #[tokio::test]
    async fn test_record_get_list() {
        let mut server = mockito::Server::new();
        let memeory = crate::store::MemoryStorage::new();
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

        let base = PocketBase::new(url.as_str(), "en", &memeory);
        let record_service = RecordService::new(&base, "test");
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
}
