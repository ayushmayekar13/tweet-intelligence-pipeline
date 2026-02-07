use actix_web::{get, web, HttpRequest, Responder};
use actix_ws::{handle, Message};
use futures_util::StreamExt as _;
use serde_json;
use tokio::sync::RwLock;

use crate::models::internal::{AppState, Tweet, TweetPayload};
use crate::auth::verify::verify_ws_request;

#[get("/ws")]
pub async fn ws(
    req: HttpRequest,
    body: web::Payload,
    data: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {

    let _user_id = match verify_ws_request(&req).await {
        Ok(u) => u,
        Err(err) => return Err(err),
    };

    let (response, mut session, mut msg_stream) = handle(&req, body)?;

    // let data_resp = data.clone();
    // let mut session_result = session.clone();
    let user_id = web::Data::new(RwLock::new(String::new()));
    // let result_user_id = user_id.clone();

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Close(close) => {
                    println!("Session closed : {:?}", close);
                    let _ = session.close(close).await;
                    break;
                }
                Message::Text(payload) => {
        
                    *user_id.write().await = _user_id.clone();

                    let payload: TweetPayload = serde_json::from_str(&payload).unwrap();

                    let tweets: Vec<Tweet> = payload.tweets.iter().map(|t| Tweet {
                        user_id: _user_id.clone(),
                        id: t.id.clone(),
                        text: t.text.clone(),
                        username: t.username.clone(),
                    } ).collect();

                    let final_payload: TweetPayload = TweetPayload { tweets: tweets };


                    println!("User ID: {}", user_id.read().await);
                    {
                        let mut buffer = data.buffer.lock().await;
                        buffer.tweets.extend(final_payload.tweets.clone());
                    }
                    println!("Total tweets recieved: {}", payload.tweets.len());

                    let reply = serde_json::json!({
                        "status": "success",
                        "received": payload.tweets.len()
                    });
                    let _ = session.text(format!("{}", reply)).await;

                    data.notify.notified().await;
            
                    let u: String = (*user_id.read().await).to_string();
                    if data.hashset.read().await.contains_key(&u) {
                        let mut resp = data.hashset.write().await;
                        let temp = resp.remove(&u);
                        match temp {
                            Some(value) => {
                                println!("Sending data");
                                let _ = session.text(format!("{:?}", value)).await;
                            }
                            _ => {
                                println!("Result loop broken:(");
                                continue;
                            }
                        }
                    }
                }
                _ => break,
            }
        }
    });

    // actix_web::rt::spawn(async move {
    //     loop {

    //         // Loop runs only when the notifier in the main loop notifies
    //         data_resp.notify.notified().await;

    //         let u: String = (*result_user_id.read().await).to_string();
    //         if data_resp.hashset.read().await.contains_key(&u) {
    //             let mut resp = data_resp.hashset.write().await;
    //             let temp = resp.remove(&u);
    //             match temp {
    //                 Some(value) => {
    //                     println!("Sending data");
    //                     //let _ = session_result.text(format!("{:?}", value)).await;
    //                 }
    //                 _ => {
    //                     println!("Result loop broken:(");
    //                     continue;
    //                 }
    //             }
    //         } else {
    //             println!("No data present after notified terminating the sender loop");
    //             break;
    //         }

    //         println!("Data sender loop running");

    //     }
    // });

    Ok(response)
}
