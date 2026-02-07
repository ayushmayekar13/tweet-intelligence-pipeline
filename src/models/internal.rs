use crate::models::similarity_result::SimilarityResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::{Mutex, Notify, RwLock};





//Server runtime models
#[derive(Deserialize, Debug, Clone, Serialize, Hash, Eq, PartialEq)]
pub struct Tweet {
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub text: String,
    pub username: String,
}

#[derive(Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct TweetPayload {
    pub tweets: Vec<Tweet>,
}

pub struct AppState {
    pub buffer: Mutex<TweetPayload>,
    pub hashset: RwLock<HashMap<String, Vec<SimilarityResult>>>,
    pub notify: Notify,
}

//Qdrant Models
#[derive(Debug, Deserialize, Serialize)]
pub struct QdrantReqeust {
    pub points: Vec<PointVector>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PointVector {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub vector: Vec<f32>,
    pub payload: UserData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserData {
    pub user_id: String,
    pub text: String,
}




