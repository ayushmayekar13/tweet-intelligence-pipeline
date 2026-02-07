use anyhow::Ok;

use crate::models::{middleware::SearchRequest, similarity_result::{Root}};
use std::env;

pub async fn similarity(payload: SearchRequest) -> Result<Root, anyhow::Error> {
    let api_key = env::var("QDRANT_API_KEY").expect("QDRANT_API_KEY must be set in environment");
    let endpoint = env::var("QDRANT_ENDPOINT").expect("QDRANT_ENDPOINT must be set in environment");

    let client = reqwest::Client::new();
    let response = client.post(format!("{}/collections/tweet_userid/points/query/batch", endpoint))
        .header("api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    println!("Response status: {}", response.status());

    if !response.status().is_success() {
        // Print the body if something goes wrong
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Qdrant search failed ({}): {}", status, text);
    }
    let response: Root = response.json().await?;

    println!("Search successful ! {}", response.result.len());
    Ok(response)
}


pub async fn search(user_id: String, limit: u32, collection: String) -> Result<reqwest::Response, anyhow::Error> {


    let payload = serde_json::json!({
        "filter":{
                    "must": [
                    {
                        "key": "user_id",
                        "match": {
                        "value": user_id
                        }
                    }
                    ]
                },
                "limit": limit,
                "with_payload": true
    });

    println!("Sending the search request");

    let api_key = env::var("QDRANT_API_KEY").expect("QDRANT_API_KEY must be set in environment");
    let endpoint = env::var("QDRANT_ENDPOINT").expect("QDRANT_ENDPOINT must be set in environment");

    let client = reqwest::Client::new();
    let response = client.post(format!("{}/collections/{}/points/scroll", endpoint, collection))
        .header("api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    println!("Response status: {}", response.status());

    // if !response.status().is_success() {
    //     // Print the body if something goes wrong
    //     let status = response.status();
    //     let text = response.text().await.unwrap_or_default();
    //     anyhow::bail!("Qdrant search failed ({}): {}", status, text);
    // }

    // let response: RootSearch = response.json().await?;

    //println!("Search successful: {}", response.result.len());

    Ok(response)
}

