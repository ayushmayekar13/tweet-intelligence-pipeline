use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserEntitlement {
    pub user_id: String,
    pub plan: String,

    pub max_tweets: u32,
    pub max_searches_per_day: u32,

    pub tweet_count: u32,
    pub searches_used_today: u32,

    pub last_reset_date: String, // YYYY-MM-DD
    pub valid_until: Option<String>, // ISO8601
}


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EntitlementPoint {
    pub id: String,
    pub vector: Option<Vec<f32>>,
    pub payload: UserEntitlement,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EntitlementRequest {
    pub points: Vec<EntitlementPoint>,
    pub next_page_offset: Option<u64>,
}




// limits results 

#[derive(Debug, Serialize, Deserialize)]
pub struct EntitlementRootSearch {
    pub result: EntitlementRequest,
    pub status: String,
    pub time: f64,
}
