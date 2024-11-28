use jsonwebtoken::{encode, decode, DecodingKey, EncodingKey, Header, Validation};
use std::time::{SystemTime, UNIX_EPOCH};
use rocket::serde::{Deserialize, Serialize};
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use rocket::http::Status;
pub struct TokenGuard(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for TokenGuard {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<TokenGuard, &'static str> {
        match request.headers().get_one("Authorization") {
            Some(token) => {
                if crate::token::decode_token(token).is_some() {
                    Outcome::Success(TokenGuard(token.to_string()))
                } else {
                    Outcome::Failure((Status::Unauthorized, "Invalid token"))
                }
            }
            None => Outcome::Failure((Status::Unauthorized, "Missing token"))
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

impl Claims {
    fn new(username: &str) -> Self {
        let now = SystemTime::now();
        let exp = now
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() + 1800; // Token expires in 30 minutes

        Self {
            sub: username.to_owned(),
            exp: exp as usize,
        }
    }
}

// 生成 JWT
pub fn generate_token(username: &str) -> String {
    let claims = Claims::new(username);
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("your_secret_key".as_ref()), // 替换为你的密钥
    )
    .unwrap()
} 

pub fn decode_token(token: &str) -> Option<String> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret("your_secret_key".as_ref()),
        &Validation::default(),
    );

    match token_data {
        Ok(token) => {
            if token.claims.exp
                >= SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as usize
            {
                Some(token.claims.sub)
            } else {
                None
            }
        }
        Err(_) => None,
    }
}
