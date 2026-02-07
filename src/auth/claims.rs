// src/auth/claims.rs

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Claims {
    pub sub: String,   // user ID
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
}

