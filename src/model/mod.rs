use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct BaseModel {
    pub id: String,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Expand {
    Record(Box<Record>),
    ListRecords(Vec<Record>),
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub struct Record {
    #[serde(flatten)]
    pub base: BaseModel,

    #[serde(rename = "collectionId")]
    pub collection_id: String,

    #[serde(rename = "collectionName")]
    pub collection_name: String,

    #[serde(flatten)]
    pub data: HashMap<String, serde_json::Value>,
    pub expand: Option<Expand>,
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub struct Admin {
    #[serde(flatten)]
    pub base: BaseModel,

    pub avatar: i64,
    pub email: String,
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub struct ListResult<T> {
    pub page: i64,

    #[serde(rename = "perPage")]
    pub per_page: i64,

    #[serde(rename = "totalItems")]
    pub total_items: i64,

    pub items: Vec<T>,
}
