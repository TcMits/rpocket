use crate::{error::RPocketError, service, store};
use async_trait::async_trait;
use futures::future::BoxFuture;
use std::sync::Arc;
use tower::layer::util::Identity;

pub const TOKEN_KEY: &str = "pb_auth";
pub const USER_OR_ADMIN_KEY: &str = "pb_user_or_admin";

#[async_trait]
pub trait PocketBaseClient {
    /// returns the default language.
    fn lang(&self) -> &str;

    /// returns the base url.
    fn base_url(&self) -> &url::Url;

    /// returns the request builder.
    fn request_builder(&self, method: reqwest::Method, url: &str) -> reqwest::RequestBuilder;

    /// returns the storage.
    fn storage(&self) -> Arc<dyn store::Storage + Sync + Send>;

    /// execute a request.
    async fn call(
        &mut self,
        request: PocketBaseRequest,
    ) -> Result<PocketBaseResponse, RPocketError>
    where
        Self: Sized;

    /// returns auth state service.
    fn auth_state(&mut self) -> service::auth_state::AuthStateService<Self>
    where
        Self: Sized;

    /// returns http service.
    fn http(&mut self) -> service::http::HTTPService<Self>
    where
        Self: Sized,
    {
        service::http::HTTPService::new(self)
    }

    /// returns crud service.
    fn crud<'a>(&'a mut self, base_path: &'a str) -> service::crud::CRUDService<'a, Self>
    where
        Self: Sized,
    {
        service::crud::CRUDService::new(self, base_path)
    }

    /// returns record service.
    fn record<'a>(&'a mut self, name: &'a str) -> service::record::RecordService<'a, Self>
    where
        Self: Sized,
    {
        service::record::RecordService::new(self, name)
    }

    /// returns admin service.
    fn admin(&mut self) -> service::admin::AdminService<Self>
    where
        Self: Sized,
    {
        service::admin::AdminService::new(self)
    }

    /// returns collection service.
    fn collection(&mut self) -> service::collection::CollectionService<Self>
    where
        Self: Sized,
    {
        service::collection::CollectionService::new(self)
    }

    /// returns log service.
    fn log(&mut self) -> service::log::LogService<Self>
    where
        Self: Sized,
    {
        service::log::LogService::new(self)
    }

    /// returns setting service.
    fn setting(&mut self) -> service::setting::SettingService<Self>
    where
        Self: Sized,
    {
        service::setting::SettingService::new(self)
    }

    /// retuns health service.
    fn health(&mut self) -> service::health::HealthService<Self>
    where
        Self: Sized,
    {
        service::health::HealthService::new(self)
    }
}

/// PocketBaseHTTPRequest is the HTTP request for PocketBase.
#[derive(Debug)]
pub struct PocketBaseHTTPRequest {
    pub request_builder: reqwest::RequestBuilder,
}

/// PocketBaseRequest is the request for PocketBase.
#[derive(Debug)]
pub enum PocketBaseRequest {
    HTTP(PocketBaseHTTPRequest),
}

/// PocketBaseHTTPResponse is the HTTP response for PocketBase.
#[derive(Debug)]
pub struct PocketBaseHTTPResponse {
    pub response: reqwest::Response,
}

/// PocketBaseResponse is the response for PocketBase.
#[derive(Debug)]
pub enum PocketBaseResponse {
    HTTP(PocketBaseHTTPResponse),
}

/// PocketBaseBuilder is the builder for PocketBase.
/// it is used to create a PocketBase struct.
pub struct PocketBaseBuilder<L> {
    lang: &'static str,
    token_key: &'static str,
    user_or_admin_key: &'static str,
    base_url: url::Url,
    storage: Arc<dyn store::Storage + Sync + Send>,
    http_client: reqwest::Client,
    layer: L,
}

impl PocketBaseBuilder<Identity> {
    /// create a new PocketBaseBuilder.
    pub fn new() -> Self {
        PocketBaseBuilder {
            lang: "en",
            token_key: TOKEN_KEY,
            user_or_admin_key: USER_OR_ADMIN_KEY,
            base_url: url::Url::parse("https://pocketbase.io").unwrap(),
            storage: Arc::new(store::MemoryStorage::new()),
            http_client: reqwest::Client::new(),
            layer: Identity::new(),
        }
    }
}

impl Default for PocketBaseBuilder<Identity> {
    fn default() -> Self {
        Self::new()
    }
}

impl<L> PocketBaseBuilder<L> {
    /// set the language.
    pub fn lang(mut self, lang: &'static str) -> Self {
        self.lang = lang;
        self
    }

    /// set the base URL.
    pub fn base_url(mut self, base_url: &str) -> Self {
        self.base_url = url::Url::parse(base_url).unwrap();
        self
    }

    /// set token key
    pub fn token_key(mut self, token_key: &'static str) -> Self {
        self.token_key = token_key;
        self
    }

    /// set user or admin key
    pub fn user_or_admin_key(mut self, user_or_admin_key: &'static str) -> Self {
        self.user_or_admin_key = user_or_admin_key;
        self
    }

    /// set the storage.
    pub fn storage(mut self, storage: Arc<dyn store::Storage + Send + Sync>) -> Self {
        self.storage = storage;
        self
    }

    /// set the http client.
    pub fn http_client(mut self, http_client: reqwest::Client) -> Self {
        self.http_client = http_client;
        self
    }

    /// add middlewares.
    pub fn layer<T>(self, layer: T) -> PocketBaseBuilder<tower::layer::util::Stack<T, L>> {
        PocketBaseBuilder {
            lang: self.lang,
            token_key: self.token_key,
            user_or_admin_key: self.user_or_admin_key,
            base_url: self.base_url,
            storage: self.storage,
            layer: tower::layer::util::Stack::new(layer, self.layer),
            http_client: self.http_client,
        }
    }

    /// build the PocketBase.
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
        let inner = Arc::new(PocketBaseRef {
            lang: self.lang,
            token_key: self.token_key,
            user_or_admin_key: self.user_or_admin_key,
            base_url: self.base_url,
            storage: self.storage,
            http_client: self.http_client,
        });

        let client = PocketBaseService {
            inner: inner.clone(),
        };

        let client = self.layer.layer(client);

        PocketBase { inner, client }
    }
}

// PocketBaseRef is the reference of PocketBase state.
// it is used to clone PocketBase.
#[derive(Clone)]
struct PocketBaseRef {
    lang: &'static str,
    token_key: &'static str,
    user_or_admin_key: &'static str,
    base_url: url::Url,
    storage: Arc<dyn store::Storage + Sync + Send>,
    http_client: reqwest::Client,
}

/// PocketBaseService is the service for sending requests.
#[derive(Clone)]
pub struct PocketBaseService {
    inner: Arc<PocketBaseRef>,
}

impl tower_service::Service<PocketBaseRequest> for PocketBaseService {
    type Response = PocketBaseResponse;
    type Error = RPocketError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(())) // TODO: check if the client is ready (healthcheck).
    }

    fn call(&mut self, request: PocketBaseRequest) -> Self::Future {
        let this = self.clone();
        let PocketBaseRequest::HTTP(req) = request;

        Box::pin(async move {
            let request = req.request_builder.build()?;

            let response = this.inner.http_client.execute(request).await?;

            Ok(PocketBaseResponse::HTTP(PocketBaseHTTPResponse {
                response,
            }))
        })
    }
}

/// PocketBase is the main struct.
/// it is used to send request to PocketBase.
#[derive(Clone)]
pub struct PocketBase<S> {
    inner: Arc<PocketBaseRef>,

    // client is an executor for sending requests.
    client: S,
}

impl PocketBase<PocketBaseService> {
    /// create a new PocketBase.
    pub fn new(base_url: &str, lang: &'static str) -> Self {
        PocketBaseBuilder::new()
            .base_url(base_url)
            .lang(lang)
            .build()
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
    /// get the base URL.
    fn base_url(&self) -> &url::Url {
        &self.inner.base_url
    }

    /// get the language.
    fn lang(&self) -> &str {
        self.inner.lang
    }

    /// get the storage.
    fn storage(&self) -> Arc<dyn store::Storage + Sync + Send> {
        self.inner.storage.clone()
    }

    fn auth_state(&mut self) -> service::auth_state::AuthStateService<Self> {
        service::auth_state::AuthStateService::new(
            self,
            self.inner.token_key,
            self.inner.user_or_admin_key,
        )
    }

    /// get request builder.
    fn request_builder(&self, method: reqwest::Method, url: &str) -> reqwest::RequestBuilder {
        self.inner.http_client.request(method, url)
    }

    /// execute a request.
    async fn call(
        &mut self,
        request: PocketBaseRequest,
    ) -> Result<PocketBaseResponse, RPocketError> {
        tower::util::Ready::new(&mut self.client)
            .await?
            .call(request)
            .await
    }
}

#[cfg(test)]
mod test {
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
        let url = server.url();

        let mock = server
            .mock("GET", "/")
            .with_status(200)
            .with_header(reqwest::header::ACCEPT_LANGUAGE.as_str(), "en")
            .create_async()
            .await;

        let mut base = PocketBaseBuilder::new()
            .base_url(url.as_str())
            .lang("en")
            .build();

        let request_builder = base.request_builder(reqwest::Method::GET, url.as_str());

        let response = base
            .call(PocketBaseRequest::HTTP(PocketBaseHTTPRequest {
                request_builder,
            }))
            .await
            .unwrap();

        let response = match response {
            PocketBaseResponse::HTTP(PocketBaseHTTPResponse { response }) => response,
        };
        assert_eq!(response.status(), reqwest::StatusCode::OK);
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
            self.inner.poll_ready(cx)
        }

        fn call(&mut self, req: PocketBaseRequest) -> Self::Future {
            let PocketBaseRequest::HTTP(mut req) = req;

            req.request_builder = req.request_builder.header("X-Test", "test");
            self.inner.call(PocketBaseRequest::HTTP(req))
        }
    }

    #[tokio::test]
    async fn test_pocket_base_builder_add_middlewares() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("GET", "/")
            .with_status(400)
            .with_header(reqwest::header::ACCEPT_LANGUAGE.as_str(), "en")
            .match_header("X-Test", "test")
            .create_async()
            .await;

        let mut base = PocketBaseBuilder::new()
            .base_url("http://localhost:8080")
            .lang("en")
            .layer(tower::layer::layer_fn(|s| {
                let service = TestService { inner: s };
                service
            }))
            .build();

        let request_builder = base.request_builder(reqwest::Method::GET, url.as_str());
        base.call(PocketBaseRequest::HTTP(PocketBaseHTTPRequest {
            request_builder,
        }))
        .await
        .unwrap();
        mock.assert_async().await;
    }
}
