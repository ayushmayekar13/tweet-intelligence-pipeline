use serde::{Deserialize, Serialize};


// Embedding response model
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EmbeddingResponse {
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: String,
    pub usage: Tokens
}


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EmbeddingData {
    pub object: String,
    pub embedding: Vec<f32>,
    pub index: u32
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Tokens {
    pub prompt_tokens: u32,
    pub total_tokens: u32
}


