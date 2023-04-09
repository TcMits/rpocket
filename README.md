# rpocket

rpocket is an unofficial SDK for the [PocketBase](https://pocketbase.io) written in Rust. This SDK provides an easy-to-use interface for interacting with the PocketBase API, allowing developers to manage their pocketbase server and perform CRUD operations seamlessly.

## Supported versions

- `v0.14.x`

## Features

-   Supports CRUD operations
-   MIT licensed
-   Depend on `reqwest` and `tower` package
-   In early development phase
-   Future support for realtime service

## Installation

To use Rust PocketBase SDK in your Rust project, simply add the following line to your `Cargo.toml` file:

```toml
rpocket = "0.1.0"
```

## Usage

Here's a simple example of how to use the Rust PocketBase SDK to fetch data from demo api:

```rust
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
```

## Contributing

This project is in its early stages, and contributions are welcome! If you find any bugs, please open an issue or submit a pull request. Any help in improving this SDK would be greatly appreciated.
