// take the embedding response then take the user-id and tweet id
// convert it to a json format which is suitable for query api
// return the json

use std::collections::HashMap;

use crate::models::{
    internal::TweetPayload,
    //middleware::{Condition, Filter, MatchValue, PointSearchVectors, SearchRequest},
    middleware::{FilterType, KeyValue, Must, PointSearchVectors, SearchRequest},
    response::EmbeddingResponse,
    similarity_result::{Root, SimilarityResult},
};
use uuid::Uuid;

pub fn unique_point_id(user_id: &str, tweet_id: &str) -> String {
    let combined = format!("{}:{}", user_id, tweet_id);
    Uuid::new_v5(&Uuid::NAMESPACE_OID, combined.as_bytes()).to_string()
}

pub fn unique_custom_id(user_id: &str, text: &str) -> String {
    // We combine user_id and the actual text content to create a unique signature
    let combined = format!("{}:custom:{}", user_id, text);
    Uuid::new_v5(&Uuid::NAMESPACE_OID, combined.as_bytes()).to_string()
}

pub fn unique_user_id(user_id: &str) -> String {
    Uuid::new_v5(&Uuid::NAMESPACE_OID, user_id.as_bytes()).to_string()
}


pub fn into_compatible(payload: &TweetPayload, response: &EmbeddingResponse) -> SearchRequest {
    let e_limit: u8 = 1;
    let e_key: String = "user_id".to_string();

    let points: Vec<PointSearchVectors> = payload
        .tweets
        .iter()
        .zip(response.data.iter())
        .map(|(tweet, embedding)| PointSearchVectors {
            query: embedding.embedding.clone(),
            filter: FilterType {
                must: vec![Must {
                    key: e_key.clone(),
                    r#match: KeyValue {
                        value: tweet.user_id.clone(),
                    },
                }],
            },
            with_payload: true,
            limit: e_limit
        })
        .collect();

    let result: SearchRequest = SearchRequest { searches: points };

    result
}


pub fn hashmap_score_user(
    payload: Result<Root, anyhow::Error>,
    user_payload: TweetPayload,
) -> Result<HashMap<String, Vec<SimilarityResult>>, anyhow::Error> {
    match payload {
        Ok(payload) => {
            let mut hash_score: HashMap<String, Vec<SimilarityResult>> = HashMap::new();
            payload
                .result
                .iter()
                .zip(user_payload.tweets.iter())
                .for_each(|(result, tweet)| {
                    if let Some(first_point) = result.points.first() {
                        hash_score
                            .entry(tweet.user_id.clone())
                            .or_insert_with(Vec::new)
                            .push(SimilarityResult {
                                id: tweet.id.clone().unwrap_or("Default_ID_Value".into()),
                                text: tweet.text.clone(),
                                score: first_point.score,
                            });
                    }
                });
            Ok(hash_score)
        }
        Err(e) => Err(e),
    }
}
