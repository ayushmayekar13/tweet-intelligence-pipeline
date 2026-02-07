use crate::{
    embeddings::embed,
    models::{internal::{AppState, PointVector, QdrantReqeust, TweetPayload, UserData}, similarity_result::{RootSearch, SearchParams}},
    qdrant_functions::{limits::{can_save_tweet}, middleware_conversion::{unique_custom_id, unique_point_id}, search::search, store::{delete_all, delete_pointid, upsert}},
};

use crate::auth::extractor::AuthUser;
use actix_web::{error::ErrorInternalServerError};
use actix_web::{get, post, web, Error, HttpResponse, Responder};
use std::time::Instant;

#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[post("/embed")]
async fn handle_embed(
    payload: web::Json<TweetPayload>,
    data: web::Data<AppState>,
) -> impl Responder {
    {
        let mut buffer = data.buffer.lock().await;
        buffer.tweets.extend(payload.tweets.clone());
    }
    // for tweet in &payload.tweets {
    //     println!("ID: {}, Text: {}", tweet.id, tweet.text);

    // }
    //println!("{:#?}", payload.tweets);

    println!("Total tweets recieved: {}", payload.tweets.len());

    HttpResponse::Ok().json({
        serde_json::json!({
            "status": "success",
            "received": payload.tweets.len()
        })
    })
}

#[post("/save")]
async fn handle_save(
    payload: web::Json<TweetPayload>, user: AuthUser
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    let can_save = can_save_tweet(user.user_id.clone())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if !can_save {
        return Ok(HttpResponse::TooManyRequests().json(
            serde_json::json!({
                "status": "error",
                "message": "Tweet save limit exceeded"
            })
        ));
    }


    let embedded = embed(&payload).await?;
    println!("Token Usage : {}", embedded.usage.total_tokens);

    if payload.tweets.len() != embedded.data.len() {
        return Err("Mismatch between tweets and embeddings".into());
    }

    // convert the embedded into QdrantRequest struct and then pass it to upsert function in store.rs

    let points: Vec<PointVector> = payload
        .tweets
        .iter()
        .zip(embedded.data.iter())
        .map(|(tweet, embedding)| {
            let final_id = match &tweet.id {
                // Case 1: It's a Tweet -> Use UserID + TweetID
                Some(tid) => unique_point_id(&user.user_id, tid),
                // Case 2: It's Custom Text -> Use UserID + Text Content
                None => unique_custom_id(&tweet.user_id, &tweet.text),
            };
            // let uuid = format!("{}{}", tweet.user_id, tweet.id.clone());
            PointVector {
                id: Some(final_id),
                vector: embedding.embedding.clone(),
                payload: UserData {
                    user_id: user.user_id.clone(),
                    text: tweet.text.clone(),
                },
            }
        })
        .collect();

    let processed_payload: QdrantReqeust = QdrantReqeust { points: points };
    let processed_len = processed_payload.points.len();

    //println!("{:?}", processed_payload);

    let debug = upsert(processed_payload).await?;

    println!("Debug : {:#?}", debug);
    println!("Vectors saved to db: {}", processed_len);

    // increment_tweet_count(processed_len, user.user_id.clone()).await;


    let elapsed_time = start_time.elapsed();
    println!("Time taken to save to DB : {}", elapsed_time.as_millis());

    Ok(HttpResponse::Ok().json({
        serde_json::json!({
            "status": "success",
            "saved to database": payload.tweets.len()
        })
    }))
}

#[post("/reset_qdrant")]
async fn reset_qdrant(user: AuthUser) -> Result<impl Responder, Error> {
    println!("{:?}", user);
    let resp = delete_all(user.user_id.clone()).await.map_err(ErrorInternalServerError)?;
    if !resp.status().is_success() {
        return Err(ErrorInternalServerError("Qdrant reset failed"));
    }

    println!(" Qdrant delete response: {}", user.user_id  );
    Ok(HttpResponse::Ok().body("Qdrant reset successful"))
}

#[post("/delete_points")]
async fn delete_points(point_id: web::Json<Vec<String>>, _user: AuthUser) -> Result<impl Responder, Error> {
    
    let resp = delete_pointid(&point_id).await.map_err(ErrorInternalServerError)?;
    if !resp.status().is_success() {
        return Err(ErrorInternalServerError("Qdrant reset failed"));
    }

    println!(" Qdrant delete response: {:?}", point_id  );
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "id": point_id
    })))
}

#[post("/search_payload")]
async fn search_payload(
    params: web::Json<SearchParams>, user: AuthUser
) -> Result<impl Responder, actix_web::Error> 
{
    let collection = "tweet_userid".to_string();
    // search() returns Result<reqwest::Response, reqwest::Error>
    let resp = match search(user.user_id.clone(), params.limit, collection).await {
        Ok(r) => r,
        Err(err) => {
            println!("Error in search(): {err}");
            return Err(ErrorInternalServerError("Search failed"));
        }
    };

    // Convert reqwest error to Actix error explicitly
    let parsed: RootSearch = resp.json()
        .await
        .map_err(|e| {
            println!("Error parsing JSON: {e}");
            ErrorInternalServerError("Failed to parse Qdrant response")
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "payload": parsed
    })))
}

