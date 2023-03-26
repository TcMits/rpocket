use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseModel {
    pub id: String,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExpandValue {
    Record(Box<Record>),
    ListRecords(Vec<Record>),
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    #[serde(flatten)]
    pub base: BaseModel,
    pub collection_id: String,
    pub collection_name: String,
    #[serde(flatten)]
    pub data: HashMap<String, serde_json::Value>,
    pub expand: Option<HashMap<String, ExpandValue>>,
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Admin {
    #[serde(flatten)]
    pub base: BaseModel,
    pub avatar: i64,
    pub email: String,
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResult<T> {
    pub page: i64,
    pub per_page: i64,
    pub total_items: i64,
    pub items: Vec<T>,
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAuth {
    #[serde(flatten)]
    pub base: BaseModel,
    pub record_id: String,
    pub collection_id: String,
    pub provider: String,
    pub provider_id: String,
}
