use jsonwebtoken::DecodingKey;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::RwLock,
    time::{Duration, Instant},
    env,
};




const JWKS_TTL: Duration = Duration::from_secs(60 * 10); // 10 minutes

#[derive(Debug, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize)]
struct Jwk {
    kid: String,
    n: String,
    e: String,
}


struct CachedJwks {
    keys: HashMap<String, DecodingKey>,
    fetched_at: Instant,
}

static JWKS_CACHE: Lazy<RwLock<Option<CachedJwks>>> =
    Lazy::new(|| RwLock::new(None));

/// Fetch JWKS from Clerk (ASYNC, network I/O)
async fn fetch_jwks() -> Result<CachedJwks, Box<dyn std::error::Error>> {

    let jwks_url = env::var("CLERK_JWKS").expect("Clerk JWKS endpoint missing");

    let jwks: Jwks = Client::new()
        .get(jwks_url)
        .send()
        .await?
        .json()
        .await?;

    let keys = jwks
        .keys
        .into_iter()
        .map(|k| {
            let decoding_key =
                DecodingKey::from_rsa_components(&k.n, &k.e)?;
            Ok((k.kid, decoding_key))
        })
        .collect::<Result<HashMap<_, _>, jsonwebtoken::errors::Error>>()?;

    Ok(CachedJwks {
        keys,
        fetched_at: Instant::now(),
    })
}

/// Call this at startup AND periodically (cron / background task)
pub async fn refresh_jwks_if_needed() -> Result<(), Box<dyn std::error::Error>> {
    let needs_refresh = {
        let cache = JWKS_CACHE.read().unwrap();
        match &*cache {
            Some(cached) => cached.fetched_at.elapsed() >= JWKS_TTL,
            None => true,
        }
    };

    if needs_refresh {
        let fresh = fetch_jwks().await?;
        let mut cache = JWKS_CACHE.write().unwrap();
        *cache = Some(fresh);
        println!("JWKS refreshing");
    }

    Ok(())
}

/// 🔴 SYNC function — SAFE to use in extractors
pub fn get_decoding_key(
    kid: &str,
) -> Result<DecodingKey, Box<dyn std::error::Error>> {
    let cache = JWKS_CACHE
        .read()
        .unwrap();

    let binding = cache.as_ref()
    .ok_or("JWKS cache not initialized")?;

    binding
        .keys
        .get(kid)
        .cloned()
        .ok_or_else(|| "KID not found in cache".into())
}
