extern crate rpocket;

use std::collections::HashMap;

use rpocket::{
    model::Collection, model::Record, rpocket::PocketBaseClient, service::admin::AdminAuthResponse,
    service::admin::AdminAuthWithPasswordConfig, service::crud::CRUDGetListConfig, PocketBase,
};

#[tokio::main]
async fn main() {
    let mut pocket_base = PocketBase::new("https://pocketbase.io", "en");

    let config = AdminAuthWithPasswordConfig::<HashMap<String, String>> {
        identity: "test@example.com".to_string(),
        password: "123456".to_string(),
        ..Default::default()
    };

    let result = pocket_base
        .admin()
        .auth_with_password::<AdminAuthResponse, HashMap<String, String>>(&config)
        .await;

    match result {
        Ok(response) => {
            println!("response: {:?}", response);
        }
        Err(error) => {
            println!("error: {:?}", error);
        }
    }

    let config = CRUDGetListConfig {
        ..Default::default()
    };

    let result = pocket_base
        .collection()
        .crud()
        .get_list::<Collection>(&config)
        .await;

    match result {
        Ok(response) => {
            println!("response: {:?}", response);
        }
        Err(error) => {
            println!("error: {:?}", error);
        }
    }

    let config = CRUDGetListConfig {
        ..Default::default()
    };

    let result = pocket_base
        .record("users")
        .crud()
        .get_list::<Record>(&config)
        .await;

    match result {
        Ok(response) => {
            println!("response: {:?}", response);
        }
        Err(error) => {
            println!("error: {:?}", error);
        }
    }
}
