use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub const DEFAULT_COLLECTION_TYPE: &str = "base";

pub fn get_default_collection_type() -> String {
    DEFAULT_COLLECTION_TYPE.to_string()
}

/// BaseModel is the base model for all models.
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseModel {
    pub id: String,
    pub created: String,
    pub updated: String,
}

/// ExpandValue is the value for the expand field.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExpandValue {
    Record(Box<Record>),
    ListRecords(Vec<Record>),
}

/// Record is the model for a record.
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

/// Admin is the model for an admin.
#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Admin {
    #[serde(flatten)]
    pub base: BaseModel,
    pub avatar: i64,
    pub email: String,
}

/// ListResult is the model for a list result.
#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResult<T> {
    pub page: i64,
    pub per_page: i64,
    pub total_items: i64,
    pub items: Vec<T>,
}

/// ExternalAuth is the model for an external auth.
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

/// SchemaField is the model for a schema field.
#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaField {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub system: bool,
    pub required: bool,
    pub options: HashMap<String, serde_json::Value>,
}

/// Collection is the model for a collection.
#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    #[serde(flatten)]
    pub base: BaseModel,
    pub name: String,
    #[serde(rename = "type", default = "get_default_collection_type")]
    pub collection_type: String,
    pub schema: Vec<SchemaField>,
    pub indexes: Vec<String>,
    pub system: bool,
    pub list_rule: Option<String>,
    pub view_rule: Option<String>,
    pub create_rule: Option<String>,
    pub update_rule: Option<String>,
    pub delete_rule: Option<String>,
    pub options: HashMap<String, serde_json::Value>,
}

/// LogRequest is the model for a log request.
#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogRequest {
    #[serde(flatten)]
    pub base: BaseModel,
    pub method: String,
    pub status: i64,
    pub auth: String,
    pub remote_ip: String,
    pub user_ip: String,
    pub referer: String,
    pub user_agent: String,
    pub meta: HashMap<String, serde_json::Value>,
}
