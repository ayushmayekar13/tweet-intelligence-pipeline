use serde::{Deserialize, Serialize};

// Qdrant similarity result model
#[derive(Debug, Serialize, Deserialize)]
pub struct Root {
    pub result: Vec<ResultItem>,
    pub status: String,
    pub time: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResultItem {
    pub points: Vec<Point>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Point {
    pub id: String,
    pub version: u32,
    pub score: f32,
    pub payload: Payload,
}

// Only for searching payload of a user_id
#[derive(Debug, Serialize, Deserialize)]
pub struct RootSearch {
    pub result: ResultItemSearch,
    pub status: String,
    pub time: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResultItemSearch {
    pub points: Vec<PointSearch>,
    pub next_page_offset: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PointSearch {
    pub id: String,
    pub payload: Payload,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    pub user_id: String,
    pub text: String
}

#[derive(Debug)]
pub struct SimilarityResult {
    pub id: String,
    pub text: String,
    pub score: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchParams {
    pub user_id: String,
    pub limit: u32,
}