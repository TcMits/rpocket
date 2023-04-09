use serde::{Deserialize, Serialize};

use crate::error::RPocketError;

/// SettingGenerateAppleClientSecretResponse is the response for the generate apple client secret.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettingGenerateAppleClientSecretResponse {
    pub secret: String,
}

/// SettingGetAllConfig is the config for the get all.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettingGetAllConfig {
    pub query_params: Vec<(String, String)>,
}

/// SettingUpdateConfig is the config for the update.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettingUpdateConfig<T> {
    #[serde(flatten)]
    pub body: T,
    #[serde(skip)]
    pub query_params: Vec<(String, String)>,
}

/// SettingTestS3Config is the config for the test s3.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettingTestS3Config<T> {
    #[serde(flatten)]
    pub body: T,
    #[serde(skip)]
    pub query_params: Vec<(String, String)>,
}

/// SettingTestEmailConfig is the config for the test email.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettingTestEmailConfig<T> {
    pub email: String,
    pub template: String,
    #[serde(flatten)]
    pub body: T,
    #[serde(skip)]
    pub query_params: Vec<(String, String)>,
}

/// SettingGenerateAppleClientSecretConfig is the config for the generate apple client secret.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettingGenerateAppleClientSecretConfig<T> {
    #[serde(rename = "clientId")]
    pub client_id: String,
    #[serde(rename = "teamId")]
    pub team_id: String,
    #[serde(rename = "keyId")]
    pub key_id: String,
    #[serde(rename = "privateKey")]
    pub private_key: String,
    pub duration: i64,
    #[serde(flatten)]
    pub body: T,
    #[serde(skip)]
    pub query_params: Vec<(String, String)>,
}

/// SettingService is the service for setting.
pub struct SettingService<'a, C> {
    client: &'a mut C,
}

impl<'a, C> SettingService<'a, C>
where
    C: crate::rpocket::PocketBaseClient + Sized,
{
    /// create a new SettingService.
    pub fn new(client: &'a mut C) -> Self {
        return SettingService { client };
    }

    /// returns all settings.
    pub async fn get_all<T>(&mut self, config: &SettingGetAllConfig) -> Result<T, RPocketError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = self.client.base_url().join("api/settings")?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::GET, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params);

        let response = self.client.http().send(request_builder).await?;

        return Ok(response.json::<T>().await?);
    }

    /// updates the provided settings.
    pub async fn update<T, B>(&mut self, config: &SettingUpdateConfig<B>) -> Result<T, RPocketError>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let url = self.client.base_url().join("api/settings")?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::PATCH, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params)
            .json(&config);

        let response = self.client.http().send(request_builder).await?;

        return Ok(response.json::<T>().await?);
    }

    /// tests the provided s3 settings.
    pub async fn test_s3<B>(&mut self, config: &SettingTestS3Config<B>) -> Result<(), RPocketError>
    where
        B: serde::Serialize,
    {
        let url = self.client.base_url().join("api/settings/test/s3")?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::POST, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params)
            .json(&config);

        self.client.http().send(request_builder).await?;

        return Ok(());
    }

    /// tests the provided email settings.
    pub async fn test_email<B>(
        &mut self,
        config: &SettingTestEmailConfig<B>,
    ) -> Result<(), RPocketError>
    where
        B: serde::Serialize,
    {
        let url = self.client.base_url().join("api/settings/test/email")?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::POST, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params)
            .json(&config);

        self.client.http().send(request_builder).await?;

        return Ok(());
    }

    /// generates an apple client secret.
    pub async fn generate_apple_client_secret<T, B>(
        &mut self,
        config: &SettingGenerateAppleClientSecretConfig<B>,
    ) -> Result<T, RPocketError>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let url = self
            .client
            .base_url()
            .join("api/settings/apple/generate-client-secret")?;

        let request_builder = self
            .client
            .request_builder(reqwest::Method::POST, url.as_str())
            .header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .query(&config.query_params)
            .json(&config);

        let response = self.client.http().send(request_builder).await?;

        return Ok(response.json::<T>().await?);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpocket::PocketBase;
    use std::collections::{BTreeMap, HashMap};

    #[tokio::test]
    async fn test_setting_get_all() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("GET", "/api/settings")
            .with_status(200)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .with_body(
                r#"{
  "meta": {
    "appName": "Acme",
    "appUrl": "http://127.0.0.1:8090",
    "hideControls": false,
    "senderName": "Support",
    "senderAddress": "support@example.com",
    "verificationTemplate": " ... ",
    "resetPasswordTemplate": " ... ",
    "confirmEmailChangeTemplate": " ... "
  },
  "logs": {
    "maxDays": 7
  },
  "smtp": {
    "enabled": false,
    "host": "smtp.example.com",
    "port": 587,
    "username": "",
    "password": "",
    "tls": true
  },
  "s3": {
    "enabled": false,
    "bucket": "",
    "region": "",
    "endpoint": "",
    "accessKey": "",
    "secret": "",
    "forcePathStyle": false
  },
  "adminAuthToken": {
    "secret": "******",
    "duration": 1209600
  },
  "adminPasswordResetToken": {
    "secret": "******",
    "duration": 1800
  },
  "recordAuthToken": {
    "secret": "******",
    "duration": 1209600
  },
  "recordPasswordResetToken": {
    "secret": "******",
    "duration": 1800
  },
  "recordEmailChangeToken": {
    "secret": "******",
    "duration": 1800
  },
  "recordVerificationToken": {
    "secret": "******",
    "duration": 604800
  },
  "googleAuth": {
    "enabled": true,
    "clientId": "demo",
    "clientSecret": "******"
  },
  "facebookAuth": {
    "enabled": false,
    "allowRegistrations": false
  },
  "githubAuth": {
    "enabled": true,
    "clientId": "demo",
    "clientSecret": "******"
  },
  "gitlabAuth": {
    "enabled": true,
    "clientId": "demo",
    "clientSecret": "******"
  },
  "discordAuth": {
    "enabled": true,
    "clientId": "demo",
    "clientSecret": "******"
  },
  "twitterAuth": {
    "enabled": true,
    "clientId": "demo",
    "clientSecret": "******"
  },
  "microsoftAuth": {
    "enabled": true,
    "clientId": "demo",
    "clientSecret": "******"
  },
  "spotifyAuth": {
    "enabled": true,
    "clientId": "demo",
    "clientSecret": "******"
  }                }"#,
            )
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut setting_service = SettingService::new(&mut base);
        let config = SettingGetAllConfig {
            ..Default::default()
        };

        let response = setting_service
            .get_all::<HashMap<String, HashMap<String, serde_json::Value>>>(&config)
            .await;

        mock.assert_async().await;
        let response = response.unwrap();

        assert!(response["meta"]["appName"] == serde_json::Value::String("Acme".to_string()));
        assert!(
            response["meta"]["appUrl"]
                == serde_json::Value::String("http://127.0.0.1:8090".to_string())
        );
    }

    #[tokio::test]
    async fn test_setting_update() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("PATCH", "/api/settings")
            .with_status(200)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .match_body(r#"{"meta":{"appName":"test","appUrl":"http://127.0.0.1:8091"}}"#)
            .with_body(
                r#"{
  "meta": {
    "appName": "test",
    "appUrl": "http://127.0.0.1:8091",
    "hideControls": false,
    "senderName": "Support",
    "senderAddress": "support@example.com",
    "verificationTemplate": " ... ",
    "resetPasswordTemplate": " ... ",
    "confirmEmailChangeTemplate": " ... "
  },
  "logs": {
    "maxDays": 7
  },
  "smtp": {
    "enabled": false,
    "host": "smtp.example.com",
    "port": 587,
    "username": "",
    "password": "",
    "tls": true
  },
  "s3": {
    "enabled": false,
    "bucket": "",
    "region": "",
    "endpoint": "",
    "accessKey": "",
    "secret": "",
    "forcePathStyle": false
  },
  "adminAuthToken": {
    "secret": "******",
    "duration": 1209600
  },
  "adminPasswordResetToken": {
    "secret": "******",
    "duration": 1800
  },
  "recordAuthToken": {
    "secret": "******",
    "duration": 1209600
  },
  "recordPasswordResetToken": {
    "secret": "******",
    "duration": 1800
  },
  "recordEmailChangeToken": {
    "secret": "******",
    "duration": 1800
  },
  "recordVerificationToken": {
    "secret": "******",
    "duration": 604800
  },
  "googleAuth": {
    "enabled": true,
    "clientId": "demo",
    "clientSecret": "******"
  },
  "facebookAuth": {
    "enabled": false,
    "allowRegistrations": false
  },
  "githubAuth": {
    "enabled": true,
    "clientId": "demo",
    "clientSecret": "******"
  },
  "gitlabAuth": {
    "enabled": true,
    "clientId": "demo",
    "clientSecret": "******"
  },
  "discordAuth": {
    "enabled": true,
    "clientId": "demo",
    "clientSecret": "******"
  },
  "twitterAuth": {
    "enabled": true,
    "clientId": "demo",
    "clientSecret": "******"
  },
  "microsoftAuth": {
    "enabled": true,
    "clientId": "demo",
    "clientSecret": "******"
  },
  "spotifyAuth": {
    "enabled": true,
    "clientId": "demo",
    "clientSecret": "******"
  }
            }"#,
            )
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut setting_service = SettingService::new(&mut base);
        let config = SettingUpdateConfig::<HashMap<String, BTreeMap<String, String>>> {
            body: HashMap::from([(
                "meta".to_string(),
                BTreeMap::from([
                    ("appName".to_string(), "test".to_string()),
                    ("appUrl".to_string(), "http://127.0.0.1:8091".to_string()),
                ]),
            )]),
            ..Default::default()
        };

        let response = setting_service
            .update::<HashMap<String, HashMap<String, serde_json::Value>>, HashMap<String, BTreeMap<String, String>>>(
                &config,
            )
            .await;

        mock.assert_async().await;
        let response = response.unwrap();

        assert!(response["meta"]["appName"] == serde_json::Value::String("test".to_string()));
        assert!(
            response["meta"]["appUrl"]
                == serde_json::Value::String("http://127.0.0.1:8091".to_string())
        )
    }

    #[tokio::test]
    async fn test_setting_test_s3() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("POST", "/api/settings/test/s3")
            .with_status(204)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut setting_service = SettingService::new(&mut base);
        let config = SettingTestS3Config::<HashMap<String, String>> {
            ..Default::default()
        };

        let response = setting_service
            .test_s3::<HashMap<String, String>>(&config)
            .await;

        mock.assert_async().await;
        response.unwrap();
    }

    #[tokio::test]
    async fn test_setting_test_email() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("POST", "/api/settings/test/email")
            .with_status(204)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .match_body(r#"{"email":"test@example.com","template":"test"}"#)
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut setting_service = SettingService::new(&mut base);
        let config = SettingTestEmailConfig::<HashMap<String, String>> {
            email: "test@example.com".to_string(),
            template: "test".to_string(),
            ..Default::default()
        };

        let response = setting_service
            .test_email::<HashMap<String, String>>(&config)
            .await;

        mock.assert_async().await;
        response.unwrap();
    }

    #[tokio::test]
    async fn test_setting_generate_apple_client_secret() {
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("POST", "/api/settings/apple/generate-client-secret")
            .with_status(200)
            .with_header("Accept-Language", "en")
            .match_header(reqwest::header::CONTENT_TYPE.as_str(), "application/json")
            .match_body(r#"{"clientId":"test","teamId":"test","keyId":"test","privateKey":"test","duration":3600}"#)
            .with_body(r#"{
                "secret":"test"
            }"#)
            .create_async()
            .await;

        let mut base = PocketBase::new(url.as_str(), "en");
        let mut setting_service = SettingService::new(&mut base);
        let config = SettingGenerateAppleClientSecretConfig::<HashMap<String, String>> {
            client_id: "test".to_string(),
            team_id: "test".to_string(),
            key_id: "test".to_string(),
            private_key: "test".to_string(),
            duration: 3600,
            ..Default::default()
        };

        let response = setting_service
            .generate_apple_client_secret::<SettingGenerateAppleClientSecretResponse, HashMap<String, String>>(
                &config,
            )
            .await;

        mock.assert_async().await;
        let response = response.unwrap();

        assert!(response.secret == "test".to_string());
    }
}
