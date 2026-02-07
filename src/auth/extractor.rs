use actix_web::{
    dev::Payload,
    error::ErrorUnauthorized,
    FromRequest,
    HttpRequest,
};
use futures_util::future::{ready, Ready};
use jsonwebtoken::{decode, Algorithm, Validation};

use crate::auth::claims::Claims;
use crate::auth::jwt::get_decoding_key;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
}

impl FromRequest for AuthUser {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        println!("AuthUser::from_request called");

        // 1. Read Authorization header
        let auth_header = match req.headers().get("Authorization") {
            Some(h) => h,
            None => {
                return ready(Err(ErrorUnauthorized("Missing Authorization header")))
            }
        };

        let auth_str = match auth_header.to_str() {
            Ok(s) => s,
            Err(_) => {
                return ready(Err(ErrorUnauthorized("Invalid Authorization header")))
            }
        };

        // 2. Extract Bearer token
        let token = match auth_str.strip_prefix("Bearer ") {
            Some(t) => t,
            None => {
                return ready(Err(ErrorUnauthorized("Expected Bearer token")))
            }
        };

        // 3. Decode JWT header to get kid
        let header = match jsonwebtoken::decode_header(token) {
            Ok(h) => h,
            Err(_) => {
                return ready(Err(ErrorUnauthorized("Invalid JWT header")))
            }
        };

        let kid = match header.kid {
            Some(k) => k,
            None => {
                return ready(Err(ErrorUnauthorized("Missing kid")))
            }
        };

        // 4. Get decoding key from cache (NON-ASYNC)
        let decoding_key = match get_decoding_key(&kid) {
            Ok(k) => k,
            Err(_) => {
                return ready(Err(ErrorUnauthorized("Unknown signing key")))
            }
        };

        // 5. Validate token
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;

        match decode::<Claims>(token, &decoding_key, &validation) {
            Ok(data) => {
                println!("Authenticated user: {}", data.claims.sub);
                ready(Ok(AuthUser {
                    user_id: data.claims.sub,
                }))
            }
            Err(err) => {
                println!("JWT validation failed: {:?}", err);
                ready(Err(ErrorUnauthorized("Invalid token")))
            }
        }
    }
}
