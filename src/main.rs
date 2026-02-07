use actix_cors::Cors;
use actix_web::{web, App, HttpServer};

use std::env;
use std::collections::HashMap;
use tokio::sync::{Mutex, Notify, RwLock};

mod embeddings;
mod models;
mod qdrant_functions;
mod routes;
mod auth;

use embeddings::embed;
use models::internal::{AppState, TweetPayload};
use routes::{
    routes::{handle_embed, handle_save, reset_qdrant, health},
    sockets::ws,
};

use crate::{qdrant_functions::{
    middleware_conversion::{hashmap_score_user, into_compatible},
    search::similarity,
    limits::*,
}, routes::routes::{delete_points, search_payload}};

// Create a global buffer over here in which the tweets will be pushed
// and every 0.6 secs the api will be called after which once there is
// a response the buffer will be cleared and next batch of tweets will
// be stored (if possible then two buffers and switching between them to avoid waiting for a response and overflowing the buffer which is going to be cleared)

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    // Remove .ok() and handle the error
    dotenv::dotenv().ok();

    

    auth::jwt::refresh_jwks_if_needed()
        .await
        .expect("Initial JWKS fetch failed");

    // Optional: background refresh task for jwks key refresh
    tokio::spawn(async {
        loop {
            if let Err(e) = auth::jwt::refresh_jwks_if_needed().await {
                eprintln!("JWKS refresh failed: {:?}", e);
            }
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        }
    });

    let host = "0.0.0.0";
    println!("Actix server running at http://{}:8080", host);

    let app_state = web::Data::new(AppState {
        buffer: Mutex::new(TweetPayload { tweets: Vec::new() }),
        hashset: RwLock::new(HashMap::new()),
        notify: Notify::new(),
    });

    tokio::spawn({
        let app_state = app_state.clone();
        async move {
            loop {
                let buffer2: TweetPayload;
                {
                    let mut buffer = app_state.buffer.lock().await;
                    // clone the buffer
                    buffer2 = buffer.clone();
                    // clearing the original buffer
                    buffer.tweets.clear();
                }
                // this function will stay out of scope as await can't be used when the buffer is locked in std::sync
                if buffer2.tweets.is_empty() {
                    println!("No tweets found, retrying in 3 seconds...");
                    tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
                    continue;
                }
                let temp = embed(&buffer2).await;

                match temp {
                    Ok(embedding_response) => {
                        println!(
                            "Total tokens used: {:#?}",
                            embedding_response.usage.total_tokens
                        );

                        let hashset = hashmap_score_user(
                            similarity(into_compatible(&buffer2, &embedding_response)).await,
                            buffer2
                        );

                        //app_state.notify.notify_waiters();

                        match hashset {
                            Ok(temp_hashset) => {
                                // Printing tweets with similarity > 0.50 (score threashold logic)
                                // for val in temp_hashset.values() {
                                //     for (_i, item) in val.iter().enumerate() {
                                //         if item.score > 0.50 {
                                //             println!("{:#?}", item);
                                //         }
                                //     }
                                // }

                                let mut temp = app_state.hashset.write().await;
                                *temp = temp_hashset;
                                app_state.notify.notify_waiters();
                            }
                            Err(_e) => {
                                eprintln!("Error printing similar tweets :( {:?}", _e);
                            }
                        }
                    }
                    Err(_e) => {
                        if true == false {
                            println!("Error converting text to embedding :")
                        }
                    }
                }
                println!("{:#?}", app_state.hashset);

                tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
                {
                    let mut hashset = app_state.hashset.write().await;
                    hashset.clear();
                }
            }
        }
    });

    HttpServer::new(move || {
        let cors = Cors::permissive();
        //.allowed_origin("https://x.com");

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .service(handle_embed)
            .service(search_payload)
            .service(delete_points)
            .service(handle_save)
            .service(reset_qdrant)
            .service(health)
            .service(ws)
            
    })
    .bind((host, 8080))?
    .run()
    .await
}
