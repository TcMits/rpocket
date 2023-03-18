use crate::{error::APIError, error::RPocketError, service, store, store::auth_storage};
use async_trait::async_trait;

#[async_trait]
pub trait PocketBaseClient: Send + Sync {
    fn base_url(&self) -> &url::Url;
    fn auth_state(&self) -> &auth_storage::AuthStorage;
    fn request_builder(&self, method: reqwest::Method, url: &str) -> reqwest::RequestBuilder;
    async fn send_request(
        &self,
        request_builder: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, RPocketError>;

    /// return record service.
    /// name: the collection name.
    fn collection<'a>(&'a self, name: &'a str) -> service::record::RecordService
    where
        Self: Sized,
    {
        return service::record::RecordService::new(self, name);
    }
}

pub struct PocketBase<'a> {
    base_url: url::Url,
    lang: &'a str,
    auth_state: auth_storage::AuthStorage<'a>,
    http_client: reqwest::Client,
}

impl<'a> PocketBase<'a> {
    /// Create a new PocketBase client.
    /// base_url: the base URL for the PocketBase server.
    /// lang: the language to use.
    /// storage: the storage to use.
    pub fn new(base_url: &'a str, lang: &'a str, storage: &'a dyn store::Storage) -> Self {
        return crate::rpocket::PocketBase::new_with_auth_state(
            base_url,
            lang,
            auth_storage::AuthStorage::new(storage),
        );
    }

    /// Create a new PocketBase with an existing AuthStorage.
    /// base_url: the base URL for the PocketBase server.
    /// lang: the language to use.
    /// storage: the storage to use.
    pub fn new_with_auth_state(
        base_url: &'a str,
        lang: &'a str,
        auth_state: auth_storage::AuthStorage<'a>,
    ) -> Self {
        return PocketBase {
            base_url: url::Url::parse(base_url).unwrap(),
            lang,
            auth_state,
            http_client: reqwest::Client::new(),
        };
    }

    /// Get the language.
    fn lang(&self) -> &str {
        return self.lang;
    }
}

#[async_trait]
impl<'a> PocketBaseClient for PocketBase<'a> {
    /// Get the base URL.
    fn base_url(&self) -> &url::Url {
        return &self.base_url;
    }

    /// Get the AuthStorage.
    fn auth_state(&self) -> &auth_storage::AuthStorage {
        return &self.auth_state;
    }

    /// Get request builder.
    /// method: the HTTP method.
    /// url: the URL.
    fn request_builder(&self, method: reqwest::Method, url: &str) -> reqwest::RequestBuilder {
        return self.http_client.request(method, url);
    }

    /// Send a request.
    /// request_builder: the request builder.
    async fn send_request(
        &self,
        mut request_builder: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, RPocketError> {
        match self.auth_state().token()? {
            Some(token) => request_builder = request_builder.header("Authorization", token),
            None => {}
        }

        let request = request_builder
            .header("Accept-Language", self.lang())
            .build()
            .map_err(|e| RPocketError::RequestError(e))?;

        // using self.client.execute() instead of request_builder.send()
        let resp: reqwest::Response = self
            .http_client
            .execute(request)
            .await
            .map_err(|e| RPocketError::RequestError(e))?;

        if !resp.status().is_success() {
            return Err(RPocketError::APIError(
                resp.json::<APIError>()
                    .await
                    .map_err(|e| RPocketError::RequestError(e))?,
            ));
        }

        return Ok(resp);
    }
}

#[cfg(test)]
mod test {
    use crate::store::{auth_storage::TOKEN_KEY, Storage};

    use super::*;

    #[test]
    fn test_pocket_base_new() {
        let memeory = store::MemoryStorage::new();
        let base = PocketBase::new("http://localhost:8080", "en", &memeory);
        assert_eq!(base.base_url().as_str(), "http://localhost:8080/");
        assert_eq!(base.lang(), "en");
    }

    #[test]
    fn test_pocket_base_request_builder() {
        let memeory = store::MemoryStorage::new();
        let base = PocketBase::new("http://localhost:8080", "en", &memeory);
        let request_builder = base.request_builder(reqwest::Method::GET, "http://localhost:8080");
        let request = request_builder.build().unwrap();
        assert_eq!(request.method(), reqwest::Method::GET);
        assert_eq!(request.url().as_str(), "http://localhost:8080/");
    }

    #[tokio::test]
    async fn test_pocket_base_send_request() {
        let mut server = mockito::Server::new();
        let memeory = store::MemoryStorage::new();
        memeory.set(TOKEN_KEY, "token").unwrap();
        let url = server.url();

        let mock = server
            .mock("GET", "/")
            .with_status(200)
            .with_header("Accept-Language", "en")
            .match_header("Accept-Language", "en")
            .match_header("Authorization", "token")
            .create_async()
            .await;

        let base = PocketBase::new(url.as_str(), "en", &memeory);
        let request_builder = base.request_builder(reqwest::Method::GET, url.as_str());
        let response = base.send_request(request_builder).await.unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_pocket_base_send_request_error() {
        let mut server = mockito::Server::new();
        let memeory = store::MemoryStorage::new();
        let url = server.url();

        let mock = server
            .mock("GET", "/")
            .with_status(400)
            .with_header("Accept-Language", "en")
            .match_header("Accept-Language", "en")
            .with_body("{\"code\": 400, \"message\": \"Bad Request\", \"data\": {}}")
            .create_async()
            .await;

        let base = PocketBase::new(url.as_str(), "en", &memeory);
        let request_builder = base.request_builder(reqwest::Method::GET, url.as_str());
        let err = base.send_request(request_builder).await.unwrap_err();

        match err {
            RPocketError::APIError(e) => {
                assert_eq!(e.code, 400);
                assert_eq!(e.message, "Bad Request");
                assert_eq!(e.data, serde_json::json!({}));
            }
            _ => panic!("Unexpected error {:?}", err),
        }
        mock.assert_async().await;
    }

    #[test]
    fn test_pocket_base_collection() {
        let memeory = store::MemoryStorage::new();
        let base = PocketBase::new("http://localhost:8080", "en", &memeory);
        base.collection("test");
    }
}
