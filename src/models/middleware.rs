use serde::{Deserialize, Serialize};

// Qdrant search request model
#[derive(Deserialize, Serialize)]
pub struct SearchRequest {
    pub searches: Vec<PointSearchVectors>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct PointSearchVectors {
    pub query: Vec<f32>,
    pub filter: FilterType,
    pub with_payload: bool,
    pub limit: u8,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct FilterType {
    pub must: Vec<Must>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Must {
    pub key: String,
    pub r#match: KeyValue,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct KeyValue {
    pub value: String,
}


