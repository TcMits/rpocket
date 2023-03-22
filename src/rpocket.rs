use crate::{error::APIError, error::RPocketError, service, store, store::auth_storage};
use async_trait::async_trait;
use futures::future::BoxFuture;
use std::sync::Arc;
use tower::layer::util::Identity;

#[async_trait]
pub trait PocketBaseClient {
    fn base_url(&self) -> &url::Url;
    fn auth_state(&self) -> Arc<dyn auth_storage::AuthState + Sync + Send>;
    fn request_builder(&self, method: reqwest::Method, url: &str) -> reqwest::RequestBuilder;
    async fn send(
        &mut self,
        request: PocketBaseRequest,
    ) -> Result<PocketBaseResponse, RPocketError>;

    /// return record service.
    /// name: the collection name.
    fn collection<'a>(&'a mut self, name: &'a str) -> service::record::RecordService<'a>
    where
        Self: Sized,
    {
        return service::record::RecordService::new(self, name);
    }
}

// PocketBaseRequest is the request for PocketBase.
// now it only supports HTTP request.
#[derive(Debug)]
pub enum PocketBaseRequest {
    HTTP {
        request_builder: reqwest::RequestBuilder,
    },
}

// PocketBaseResponse is the response for PocketBase.
// now it only supports HTTP response.
#[derive(Debug)]
pub enum PocketBaseResponse {
    HTTP { response: reqwest::Response },
}

pub struct PocketBaseBuilder<L> {
    lang: &'static str,
    base_url: url::Url,
    // auth
    auth_state: Arc<dyn auth_storage::AuthState + Send + Sync>,
    layer: L,
}

impl PocketBaseBuilder<Identity> {
    /// Create a new PocketBaseBuilder.
    pub fn new() -> Self {
        return PocketBaseBuilder {
            lang: "en",
            base_url: url::Url::parse("https://api.pocketbase.io").unwrap(),
            auth_state: Arc::new(auth_storage::AuthStorage::new(Arc::new(
                store::MemoryStorage::new(),
            ))),
            layer: Identity::new(),
        };
    }
}

impl<L> PocketBaseBuilder<L> {
    /// Set the language.
    pub fn lang(mut self, lang: &'static str) -> Self {
        self.lang = lang;
        return self;
    }

    /// Set the base URL.
    pub fn base_url(mut self, base_url: &str) -> Self {
        self.base_url = url::Url::parse(base_url).unwrap();
        return self;
    }

    /// Set the auth state.
    pub fn auth_state(
        mut self,
        auth_state: Arc<dyn auth_storage::AuthState + Send + Sync>,
    ) -> Self {
        self.auth_state = auth_state;
        return self;
    }

    /// Add middlewares.
    pub fn layer<T>(self, layer: T) -> PocketBaseBuilder<tower::layer::util::Stack<T, L>> {
        return PocketBaseBuilder {
            lang: self.lang,
            base_url: self.base_url,
            auth_state: self.auth_state,
            layer: tower::layer::util::Stack::new(layer, self.layer),
        };
    }

    /// Build the PocketBase.
    pub fn build(self) -> PocketBase<L::Service>
    where
        L: tower::Layer<PocketBaseService>,
        L::Service: tower_service::Service<
                PocketBaseRequest,
                Response = PocketBaseResponse,
                Error = RPocketError,
                Future = BoxFuture<'static, Result<PocketBaseResponse, RPocketError>>,
            > + Clone
            + Send
            + Sync,
    {
        let http_client = reqwest::Client::new();
        let client = PocketBaseService {
            inner: Arc::new(PocketBaseServiceRef {
                http_client: http_client.clone(),
            }),
        };

        let client = self.layer.layer(client);

        let inner = PocketBaseRef {
            lang: self.lang,
            base_url: self.base_url,
            auth_state: self.auth_state,
            http_client,
        };

        return PocketBase {
            inner: Arc::new(inner),
            client,
        };
    }
}

// PocketBaseServiceRef is the reference of PocketBaseService state.
// it is used to clone PocketBaseService.
struct PocketBaseServiceRef {
    http_client: reqwest::Client,
}

// PocketBaseService is the service for sending requests.
#[derive(Clone)]
pub struct PocketBaseService {
    inner: Arc<PocketBaseServiceRef>,
}

impl tower_service::Service<PocketBaseRequest> for PocketBaseService {
    type Response = PocketBaseResponse;
    type Error = RPocketError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        return std::task::Poll::Ready(Ok(())); // TODO: check if the client is ready (healthcheck).
    }

    /// Call the service.
    /// request: the request.
    fn call(&mut self, request: PocketBaseRequest) -> Self::Future {
        let this = self.clone();
        let request_builder = match request {
            PocketBaseRequest::HTTP { request_builder } => request_builder,
        };

        return Box::pin(async move {
            let request = request_builder
                .build()
                .map_err(|e| RPocketError::RequestError(e))?;

            let response = this
                .inner
                .http_client
                .execute(request)
                .await
                .map_err(|e| RPocketError::RequestError(e));

            return response.map(|response| PocketBaseResponse::HTTP { response });
        });
    }
}

// PocketBaseRef is the reference of PocketBase state.
// it is used to clone PocketBase.
struct PocketBaseRef {
    lang: &'static str,
    base_url: url::Url,
    // auth
    auth_state: Arc<dyn auth_storage::AuthState + Send + Sync>,
    // cloned client of builder
    // http_client will be used to create request_builder
    http_client: reqwest::Client,
}

// PocketBase is the main struct.
// it is used to send request to PocketBase.
#[derive(Clone)]
pub struct PocketBase<S> {
    inner: Arc<PocketBaseRef>,

    // client is an executor for sending requests.
    client: S,
}

impl PocketBase<PocketBaseService> {
    /// Create a new PocketBase.
    /// base_url: the base URL.
    /// lang: the language.
    pub fn new(base_url: &str, lang: &'static str) -> Self {
        return PocketBaseBuilder::new()
            .base_url(base_url)
            .lang(lang)
            .build();
    }
}

impl<S> PocketBase<S> {
    /// Get the language.
    pub fn lang(&self) -> &str {
        return self.inner.lang;
    }

    /// Send a request.
    /// request_builder: the request builder.
    pub async fn send_request(
        &mut self,
        pb_request: PocketBaseRequest,
    ) -> Result<PocketBaseResponse, RPocketError>
    where
        S: tower_service::Service<
                PocketBaseRequest,
                Response = PocketBaseResponse,
                Error = RPocketError,
                Future = BoxFuture<'static, Result<PocketBaseResponse, RPocketError>>,
            > + Send
            + Sync,
    {
        let mut request_builder = match pb_request {
            PocketBaseRequest::HTTP { request_builder } => request_builder,
        };

        request_builder = request_builder.header("Accept-Language", self.inner.lang);
        match self.inner.auth_state.token()? {
            Some(token) => request_builder = request_builder.header("Authorization", token),
            None => {}
        }

        let resp = self
            .client
            .call(PocketBaseRequest::HTTP { request_builder })
            .await?;

        match resp {
            PocketBaseResponse::HTTP { response } => {
                if !response.status().is_success() {
                    return Err(RPocketError::APIError(
                        response
                            .json::<APIError>()
                            .await
                            .map_err(|e| RPocketError::RequestError(e))?,
                    ));
                }
                return Ok(PocketBaseResponse::HTTP { response });
            }
        };
    }
}

#[async_trait]
impl<S> PocketBaseClient for PocketBase<S>
where
    S: tower_service::Service<
            PocketBaseRequest,
            Response = PocketBaseResponse,
            Error = RPocketError,
            Future = BoxFuture<'static, Result<PocketBaseResponse, RPocketError>>,
        > + Send
        + Sync,
{
    /// Get the base URL.
    fn base_url(&self) -> &url::Url {
        return &self.inner.base_url;
    }

    /// Get the AuthStorage.
    fn auth_state(&self) -> Arc<dyn auth_storage::AuthState + Sync + Send> {
        return self.inner.auth_state.clone();
    }

    /// Get request builder.
    fn request_builder(&self, method: reqwest::Method, url: &str) -> reqwest::RequestBuilder {
        return self.inner.http_client.request(method, url);
    }

    /// Send a request.
    async fn send(
        &mut self,
        request: PocketBaseRequest,
    ) -> Result<PocketBaseResponse, RPocketError> {
        match request {
            PocketBaseRequest::HTTP { request_builder } => {
                return self
                    .send_request(PocketBaseRequest::HTTP { request_builder })
                    .await;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::store::{auth_storage::TOKEN_KEY, Storage};

    use super::*;

    #[test]
    fn test_pocket_base_new() {
        let base = PocketBase::new("http://localhost:8080", "en");
        assert_eq!(base.base_url().as_str(), "http://localhost:8080/");
        assert_eq!(base.lang(), "en");
    }

    #[test]
    fn test_pocket_base_request_builder() {
        let base = PocketBase::new("http://localhost:8080", "en");
        let request_builder = base.request_builder(reqwest::Method::GET, "http://localhost:8080");
        let request = request_builder.build().unwrap();
        assert_eq!(request.method(), reqwest::Method::GET);
        assert_eq!(request.url().as_str(), "http://localhost:8080/");
    }

    #[tokio::test]
    async fn test_pocket_base_send_request() {
        let mut server = mockito::Server::new();
        let memeory = Arc::new(store::MemoryStorage::new());
        memeory.set(TOKEN_KEY, "token").unwrap();
        let auth_state = Arc::new(auth_storage::AuthStorage::new(memeory.clone()));
        let url = server.url();

        let mock = server
            .mock("GET", "/")
            .with_status(200)
            .with_header("Accept-Language", "en")
            .match_header("Accept-Language", "en")
            .match_header("Authorization", "token")
            .create_async()
            .await;

        let mut base = PocketBaseBuilder::new()
            .base_url(url.as_str())
            .lang("en")
            .auth_state(auth_state)
            .build();
        let request_builder = base.request_builder(reqwest::Method::GET, url.as_str());
        let response = base
            .send(PocketBaseRequest::HTTP { request_builder })
            .await
            .unwrap();

        let response = match response {
            PocketBaseResponse::HTTP { response } => response,
        };
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_pocket_base_send_request_error() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("GET", "/")
            .with_status(400)
            .with_header("Accept-Language", "en")
            .match_header("Accept-Language", "en")
            .with_body("{\"code\": 400, \"message\": \"Bad Request\", \"data\": {}}")
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let request_builder = base.request_builder(reqwest::Method::GET, url.as_str());
        let err = base
            .send(PocketBaseRequest::HTTP { request_builder })
            .await
            .unwrap_err();

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

    #[derive(Clone)]
    struct TestService<S> {
        inner: S,
    }

    impl<S> tower_service::Service<PocketBaseRequest> for TestService<S>
    where
        S: tower_service::Service<PocketBaseRequest>,
    {
        type Response = S::Response;
        type Error = S::Error;
        type Future = S::Future;

        fn poll_ready(
            &mut self,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            return self.inner.poll_ready(cx);
        }

        fn call(&mut self, req: PocketBaseRequest) -> Self::Future {
            let mut request_builder = match req {
                PocketBaseRequest::HTTP { request_builder } => request_builder,
            };

            request_builder = request_builder.header("X-Test", "test");
            self.inner.call(PocketBaseRequest::HTTP { request_builder })
        }
    }

    #[tokio::test]
    async fn test_pocket_base_builder_add_middlewares() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("GET", "/")
            .with_status(400)
            .with_header("Accept-Language", "en")
            .match_header("Accept-Language", "en")
            .match_header("X-Test", "test")
            .with_body("{\"code\": 400, \"message\": \"Bad Request\", \"data\": {}}")
            .create_async()
            .await;

        let mut base = PocketBaseBuilder::new()
            .base_url("http://localhost:8080")
            .lang("en")
            .layer(tower::layer::layer_fn(|s| {
                let service = TestService { inner: s };
                return service;
            }))
            .build();

        let request_builder = base.request_builder(reqwest::Method::GET, url.as_str());
        base.send(PocketBaseRequest::HTTP { request_builder })
            .await
            .unwrap_err();
        mock.assert_async().await;
    }

    #[test]
    fn test_pocket_base_collection() {
        let mut base = PocketBase::new("http://localhost:8080", "en");
        base.collection("test");
    }
}
