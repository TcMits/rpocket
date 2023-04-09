use crate::error::APIError;
use crate::error::RPocketError;
use crate::rpocket::{
    PocketBaseHTTPRequest, PocketBaseHTTPResponse, PocketBaseRequest, PocketBaseResponse,
};

/// HTTPRequest is the request for the HTTP service.
pub struct HTTPService<'a, C> {
    client: &'a mut C,
}

impl<'a, C> HTTPService<'a, C>
where
    C: crate::rpocket::PocketBaseClient + Sized,
{
    /// create a new HTTPService.
    pub fn new(client: &'a mut C) -> Self {
        return HTTPService { client };
    }

    /// send a request.
    pub async fn send(
        &mut self,
        mut request_builder: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, RPocketError> {
        request_builder = request_builder.header(
            reqwest::header::ACCEPT_LANGUAGE.as_str(),
            self.client.lang(),
        );

        // add auth token
        match self.client.auth_state().get_token().await? {
            Some(token) => {
                request_builder =
                    request_builder.header(reqwest::header::AUTHORIZATION.as_str(), token)
            }
            None => {}
        }

        let pb_request = PocketBaseRequest::HTTP(PocketBaseHTTPRequest { request_builder });
        let pb_response = self.client.call(pb_request).await?;

        match pb_response {
            PocketBaseResponse::HTTP(PocketBaseHTTPResponse { response }) => {
                if !response.status().is_success() {
                    return Err(RPocketError::APIError(response.json::<APIError>().await?));
                }
                return Ok(response);
            }
        };
    }
}

#[cfg(test)]
mod test {
    use crate::{rpocket::PocketBaseClient, rpocket::TOKEN_KEY, PocketBase};

    use super::*;

    #[tokio::test]
    async fn test_http_send() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("GET", "/")
            .with_status(400)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::ACCEPT_LANGUAGE.as_str(), "en")
            .match_header(reqwest::header::AUTHORIZATION.as_str(), "test")
            .with_body(
                r#"{
                "code": 400,
                "message": "Bad Request",
                "data": {}
            }"#,
            )
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        base.storage().set(TOKEN_KEY, "test").await.unwrap();

        let request_builder = base.request_builder(reqwest::Method::GET, url.as_str());
        let mut http_service = HTTPService::new(&mut base);

        let response = http_service.send(request_builder).await.unwrap_err();
        mock.create_async().await;

        match response {
            RPocketError::APIError(APIError {
                code,
                message,
                data,
            }) => {
                assert!(code == 400);
                assert!(message == "Bad Request");
                assert!(data == serde_json::Value::Object(serde_json::Map::new()));
            }
            _ => {
                panic!("unexpected error");
            }
        }
    }
}
