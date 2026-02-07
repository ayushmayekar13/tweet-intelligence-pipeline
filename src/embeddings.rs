//use serde::Deserialize;
use serde_json::json;
use std::env;

use crate::{models::response::EmbeddingResponse, TweetPayload};

pub async fn embed(buffer2: &TweetPayload) -> Result<EmbeddingResponse, reqwest::Error> {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set in environment");
    let endpoint = env::var("OPENAI_ENDPOINT").expect("OPENAI_ENDPOINT must be set in environment");

    let texts: Vec<&String> = buffer2.tweets.iter().map(|tweet| &tweet.text).collect();
    // let user_id: Vec<&String> = buffer2.tweets.iter().map(|tweet| &tweet.user_id).collect();

    let body = json!({
        "input": texts,
        // "user": user_id,
        "model": "text-embedding-3-small"
    });

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}", endpoint))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let response = response.error_for_status()?;

    let embedding: EmbeddingResponse = response.json().await?;

    return Ok(embedding);
}

// pub fn rotate(matrix: Vec<Vec<i32>>){
//     let i = 0;
//     let j = 1;
//     println!("{}",matrix[i][j]);
// }
