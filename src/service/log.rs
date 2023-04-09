use serde::{Deserialize, Serialize};

use crate::{error::RPocketError, service};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogGetRequestsStatsResponse {
    pub total: i64,
    pub date: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogGetRequestsStatsConfig {
    pub query_params: Vec<(String, String)>,
}

pub struct LogService<'a, C> {
    client: &'a mut C,
    request_base_path: String,
}

impl<'a, C> LogService<'a, C>
where
    C: crate::rpocket::PocketBaseClient + Sized,
{
    /// create a new LogService.
    pub fn new(client: &'a mut C) -> Self {
        return LogService {
            client,
            request_base_path: "api/logs/requests".to_string(),
        };
    }

    /// returns crud service.
    pub fn crud(&'a mut self) -> service::crud::CRUDService<'a, C> {
        return self.client.crud(&self.request_base_path);
    }

    /// get the requests stats of the server.
    pub async fn get_requests_stats<T>(
        &mut self,
        config: &LogGetRequestsStatsConfig,
    ) -> Result<Vec<T>, RPocketError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = self
            .client
            .base_url()
            .join(format!("{}/stats", self.request_base_path).as_str())?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::GET, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params);

        let response = self.client.http().send(request_builder).await?;

        return Ok(response.json::<Vec<T>>().await?);
    }
}

#[cfg(test)]
mod test {
    use crate::PocketBase;

    use super::*;

    #[test]
    fn test_log_crud() {
        let mut base = PocketBase::new("http://test.com", "en");
        let log_service = LogService::new(&mut base);

        assert!(log_service.request_base_path == "api/logs/requests");
    }

    #[tokio::test]
    async fn test_log_get_requests_stats() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("GET", "/api/logs/requests/stats")
            .with_status(200)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .with_body(
                r#"[
  {
    "total": 4,
    "date": "2022-06-01 19:00:00.000"
  },
  {
    "total": 1,
    "date": "2022-06-02 12:00:00.000"
  },
  {
    "total": 8,
    "date": "2022-06-02 13:00:00.000"
  }
                ]"#,
            )
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut log_service = LogService::new(&mut base);
        let config = LogGetRequestsStatsConfig {
            ..Default::default()
        };

        let response = log_service
            .get_requests_stats::<LogGetRequestsStatsResponse>(&config)
            .await;

        mock.assert_async().await;
        let response = response.unwrap();

        assert!(response.len() == 3);
        assert!(response[0].total == 4);
        assert!(response[0].date == "2022-06-01 19:00:00.000");
    }
}
