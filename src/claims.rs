use rocket::request::{FromRequest, Outcome};
use rocket::http::{CookieJar, Status};
use rocket::Request;
use rocket::serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use async_trait::async_trait;
use rocket::serde::Serialize;
use crate::token::decode_token;

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
            .as_secs()
            + 1800; // Token expires in 30 minutes

        Self {
            sub: username.to_owned(),
            exp: exp as usize,
        }
    }
}

#[derive(Debug)]
pub struct JwtToken(pub String);

#[async_trait]
impl<'r> FromRequest<'r> for JwtToken {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let cookies: &CookieJar<'_> = request.cookies();
        if let Some(cookie) = cookies.get("token") {
            let token = cookie.value();
            match decode_token(token) {
                Some(username) => Outcome::Success(JwtToken(username)),
                None => Outcome::Error((Status::Unauthorized, ())),
            }
        } else {
            Outcome::Error((Status::Unauthorized, ()))
        }
    }
}
