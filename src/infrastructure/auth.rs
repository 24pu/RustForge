use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use anyhow::Result;
use uuid::Uuid;
use bcrypt::{hash, verify, DEFAULT_COST};

const JWT_SECRET: &[u8] = b"your-secret-key-change-in-production";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

impl Claims {
    pub fn new(user_id: Uuid) -> Self {
        let now = Utc::now();
        let expire = now + Duration::hours(24);
        Self {
            sub: user_id.to_string(),
            exp: expire.timestamp() as usize,
            iat: now.timestamp() as usize,
        }
    }
}

pub fn generate_token(user_id: Uuid) -> Result<String> {
    let claims = Claims::new(user_id);
    Ok(encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET))?)
}

pub fn verify_token(token: &str) -> Result<Claims> {
    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(JWT_SECRET), &Validation::default())?;
    Ok(token_data.claims)
}

pub fn hash_password(password: &str) -> Result<String> {
    Ok(hash(password, DEFAULT_COST)?)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    Ok(verify(password, hash)?)
}