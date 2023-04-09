use serde::{Deserialize, Serialize};

use crate::error::RPocketError;

/// HealthCheckResponse is the response for the health check.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: i64,
    pub message: String,
}

/// HealthCheckConfig is the config for the health check.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub query_params: Vec<(String, String)>,
}

/// HealthService is the service for health.
pub struct HealthService<'a, C> {
    client: &'a mut C,
}

impl<'a, C> HealthService<'a, C>
where
    C: crate::rpocket::PocketBaseClient + Sized,
{
    /// create a new HealthService.
    pub fn new(client: &'a mut C) -> Self {
        return HealthService { client };
    }

    /// get the health status of the server.
    pub async fn check<T>(&mut self, config: &HealthCheckConfig) -> Result<T, RPocketError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = self.client.base_url().join("api/health")?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::GET, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params);

        let response = self.client.http().send(request_builder).await?;

        return Ok(response.json::<T>().await?);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpocket::PocketBase;

    #[tokio::test]
    async fn test_health_check() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("GET", "/api/health")
            .with_status(200)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .with_body(
                r#"{
                    "status": 200,
                    "message": "API is healthy."
                }"#,
            )
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut health_service = HealthService::new(&mut base);
        let config = HealthCheckConfig {
            ..Default::default()
        };

        let response = health_service.check::<HealthCheckResponse>(&config).await;

        mock.assert_async().await;
        let response = response.unwrap();

        assert_eq!(response.status, 200);
        assert_eq!(response.message, "API is healthy.");
    }
}
