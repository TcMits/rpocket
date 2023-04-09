use serde::{Deserialize, Serialize};

use crate::{error::RPocketError, model::Collection, service};

/// CollectionImportConfig is the config for importing collections.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CollectionImportConfig<T> {
    pub collections: Vec<Collection>,
    #[serde(rename = "deleteMissing")]
    pub delete_missing: bool,
    #[serde(flatten)]
    pub body: T,
    #[serde(skip)]
    pub query_params: Vec<(String, String)>,
}

/// CollectionService is the service for collections.
pub struct CollectionService<'a, C> {
    client: &'a mut C,
    collection_base_path: String,
}

impl<'a, C> CollectionService<'a, C>
where
    C: crate::rpocket::PocketBaseClient + Sized,
{
    /// create a new CollectionService.
    pub fn new(client: &'a mut C) -> Self {
        return CollectionService {
            client,
            collection_base_path: "api/collections".to_string(),
        };
    }

    /// returns crud service.
    pub fn crud(&'a mut self) -> service::crud::CRUDService<'a, C> {
        return self.client.crud(&self.collection_base_path);
    }

    /// imports the provided collections.
    pub async fn import<B>(
        &'a mut self,
        config: &CollectionImportConfig<B>,
    ) -> Result<(), RPocketError>
    where
        B: serde::Serialize,
    {
        let url = self
            .client
            .base_url()
            .join(format!("{}/import", self.collection_base_path).as_str())?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::PUT, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params)
            .json(&config);

        self.client.http().send(request_builder).await?;
        return Ok(());
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::{model::BaseModel, PocketBase};

    use super::*;

    #[test]
    fn test_collection_crud() {
        let mut base = PocketBase::new("http://test.com", "en");
        let collection_service = CollectionService::new(&mut base);

        assert!(collection_service.collection_base_path == "api/collections");
    }

    #[tokio::test]
    async fn test_collection_import() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("PUT", "/api/collections/import")
            .with_status(204)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .match_body(
                r#"{"collections":[{"id":"1","created":"","updated":"","name":"test","type":"","schema":[],"indexes":[],"system":false,"listRule":null,"viewRule":null,"createRule":null,"updateRule":null,"deleteRule":null,"options":{}},{"id":"2","created":"","updated":"","name":"test2","type":"","schema":[],"indexes":[],"system":false,"listRule":null,"viewRule":null,"createRule":null,"updateRule":null,"deleteRule":null,"options":{}}],"deleteMissing":false}"#,
            )
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut collection_service = CollectionService::new(&mut base);
        let config = CollectionImportConfig::<HashMap<String, String>> {
            collections: vec![
                Collection {
                    base: BaseModel {
                        id: "1".to_string(),
                        ..Default::default()
                    },
                    name: "test".to_string(),
                    ..Default::default()
                },
                Collection {
                    base: BaseModel {
                        id: "2".to_string(),
                        ..Default::default()
                    },
                    name: "test2".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let response = collection_service
            .import::<HashMap<String, String>>(&config)
            .await;
        mock.assert_async().await;
        response.unwrap();
    }
}
