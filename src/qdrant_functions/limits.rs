use chrono::prelude::*;
use std::{env};
use crate::{models::{limits::{EntitlementPoint, EntitlementRequest, UserEntitlement, EntitlementRootSearch}, similarity_result::RootSearch}, qdrant_functions::{middleware_conversion::unique_user_id, search::search}};



pub async fn create_user(user_id: String) -> Result<UserEntitlement, anyhow::Error> {



    let api_key = env::var("QDRANT_API_KEY").expect("QDRANT_API_KEY must be set in environment");
    let endpoint = env::var("QDRANT_ENDPOINT").expect("QDRANT_ENDPOINT must be set in environment");

    let date = Local::now().date_naive();
    let payload = EntitlementPoint {
        id: unique_user_id(&user_id),
        vector: Some([0.0].to_vec()),
        payload: UserEntitlement { 
            user_id: user_id, 
            plan: "Free".to_string(), 
            max_tweets: 0, 
            max_searches_per_day: 20, 
            tweet_count: 0, 
            searches_used_today: 0, 
            last_reset_date: date.format("%F").to_string(), 
            valid_until: None,
        },
    };
    let arr_payload = {
        EntitlementRequest {
            points: vec![payload.clone()],
            next_page_offset: None,
        }
    };


    let client = reqwest::Client::new();
    let response = client.put(format!("{}/collections/user_entitlement/points", endpoint))
        .header("api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&arr_payload)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        println!("Qdrant upload failed ({}): {}", status, text);
        anyhow::bail!("Qdrant upload failed ({}): {}", status, text);
    }

    Ok(payload.payload)


}

pub async fn get_or_create_entitlement(user_id: String) -> Result<UserEntitlement, anyhow::Error> {


    let collection = "user_entitlement".to_string();
    let result = match search(user_id.clone(), 1, collection).await {
        Ok(r) => {
            let search_result: EntitlementRootSearch = r.json()
            .await
            .map_err(|e| {
                println!("Error parsing JSON: {e}");
                anyhow::anyhow!("Failed to parse Qdrant response: {}", e)
            })?;

            // Extract UserEntitlement from the point's payload, or create if not found
            if search_result.result.points.len() != 0 {
                search_result.result.points[0].payload.clone()
            } else {
                create_user(user_id).await?
            }

        },
        Err(err) => {
            println!("No user found, creating a new entry {}", err);
            create_user(user_id).await?
        }
    };
    
    Ok(result)
}

pub async fn can_save_tweet(user_id: String) -> Result<bool, anyhow::Error> {
    let result = get_or_create_entitlement(user_id.clone()).await?;

    let collection = "tweet_userid".to_string();
    let resp = match search(user_id, 100, collection).await {
        Ok(r) => r,
        Err(err) => {
            println!("Error in search(): {err}");
            anyhow::bail!("Search failed: {}", err);
        }
    };

    // Convert reqwest error to anyhow error
    let parsed: RootSearch = resp.json()
        .await
        .map_err(|e| {
            println!("Error parsing JSON: {e}");
            anyhow::anyhow!("Failed to parse Qdrant response: {}", e)
        })?;

    

    if result.max_tweets <= parsed.result.points.len() as u32 {
        return Ok(false);
    } else {
        return Ok(true);
    }
}

// pub async fn increment_tweet_count(saved_count: usize, user_id: String ) {

// }