use crate::models::internal::QdrantReqeust;
use anyhow::Ok;
use std::env;

pub async fn upsert(payload: QdrantReqeust) -> Result<reqwest::Response, anyhow::Error> {
    let api_key = env::var("QDRANT_API_KEY").expect("QDRANT_API_KEY must be set in environment");
    let endpoint = env::var("QDRANT_ENDPOINT").expect("QDRANT_ENDPOINT must be set in environment");

    let client = reqwest::Client::new();
    let response = client
        .put(format!("{}/collections/tweet_userid/points", endpoint))
        .header("api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;



    if !response.status().is_success() {
        // Print the body if something goes wrong
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        println!("Qdrant upload failed ({}): {}", status, text);
        anyhow::bail!("Qdrant upload failed ({}): {}", status, text);
    }

    println!("Data successfully uploaded!!!");
    Ok(response)
}

pub async fn delete_all(user_id: String) -> Result<reqwest::Response, anyhow::Error> {
    let api_key = env::var("QDRANT_API_KEY").expect("QDRANT_API_KEY must be set in environment");
    let endpoint = env::var("QDRANT_ENDPOINT").expect("QDRANT_ENDPOINT must be set in environment");

    let client = reqwest::Client::new();
    let response1 = client
        .post(format!(
            "{}/collections/tweet_userid/points/delete",
            endpoint
        ))
        .header("api-key", api_key.clone())
        .header("Content-Type", "application/json")
        .json(&{
            serde_json::json!({
              "filter": {
                "must": [
                    {
                        "key": "user_id",
                        "match": {
                            "value": user_id
                        }
                    }
                ]
              }
            })
        })
        .send()
        .await?;

    if !response1.status().is_success() {
        // Print the body if something goes wrong
        let status1 = response1.status();
        let text1 = response1.text().await.unwrap_or_default();
        anyhow::bail!(
            "Qdrant deletion failed -> Summarize: {}, {}",
            status1,
            text1
        );
    }

    Ok(response1)
}

pub async fn delete_pointid(point_ids: &Vec<String>) -> Result<reqwest::Response, anyhow::Error> {
    let api_key = env::var("QDRANT_API_KEY").expect("QDRANT_API_KEY must be set in environment");
    let endpoint = env::var("QDRANT_ENDPOINT").expect("QDRANT_ENDPOINT must be set in environment");

    let body = serde_json::json!({
        "points": point_ids
    });
    
    println!("DELETE BODY = {}", serde_json::to_string_pretty(&body).unwrap());
    
    let client = reqwest::Client::new();
    let response1 = client
        .post(format!(
            "{}/collections/tweet_userid/points/delete",
            endpoint
        ))
        .header("api-key", api_key.clone())
        .header("Content-Type", "application/json")
        .json(&{
            serde_json::json!({
              "points": point_ids
            })
        })
        .send()
        .await?;

    if !response1.status().is_success() {
        // Print the body if something goes wrong
        let status1 = response1.status();
        let text1 = response1.text().await.unwrap_or_default();
        anyhow::bail!(
            "Qdrant deletion failed -> Summarize: {}, {}",
            status1,
            text1
        );
    }

    Ok(response1)
}
