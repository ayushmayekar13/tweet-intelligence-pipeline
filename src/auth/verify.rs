use actix_web::{HttpRequest, error::ErrorUnauthorized};
use jsonwebtoken::{decode, decode_header, Algorithm, Validation};

use crate::auth::claims::Claims;
use crate::auth::jwt::get_decoding_key;

pub async fn verify_ws_request(req: &HttpRequest) -> Result<String, actix_web::Error> {
    // let auth_header = req
    //     .headers()
    //     .get("Authorization")
    //     .ok_or_else(|| ErrorUnauthorized("Missing Authorization header"))?;

    // let auth_str = auth_header
    //     .to_str()
    //     .map_err(|_| ErrorUnauthorized("Invalid Authorization header"))?;

    let token = req
        .query_string()
        .split('&')
        .find_map(|kv| kv.strip_prefix("token="))
        .ok_or_else(|| ErrorUnauthorized("Missing token"))?;

    let token = urlencoding::decode(token)
        .map_err(|_| ErrorUnauthorized("Invalid token encoding"))?;

    let header = decode_header(&token)
        .map_err(|_| ErrorUnauthorized("Invalid JWT header"))?;

    let kid = header
        .kid
        .ok_or_else(|| ErrorUnauthorized("Missing kid"))?;

    let decoding_key = match get_decoding_key(&kid) {
        Ok(k) => k,
        Err(_) => {
            return Err(ErrorUnauthorized("Unknown signing key"))
        }
    };

    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;
    validation.leeway = 30;

    let token_data = decode::<Claims>(&token, &decoding_key, &validation)
        .map_err(|_| ErrorUnauthorized("Invalid or expired token"))?;

    Ok(token_data.claims.sub)
}
