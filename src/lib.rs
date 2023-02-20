use async_trait::async_trait;
use axum::{
    extract::{FromRef, FromRequestParts},
    headers::{authorization::Bearer, Authorization},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response}, TypedHeader,
};
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation, EncodingKey};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct TokenConfig {
    pub decode_key: DecodingKey,
    pub encode_key: EncodingKey
}

pub struct AuthToken<T: Serialize + for<'a> Deserialize<'a>> {
    pub claim: T,
}

impl<T: Serialize + for<'a> Deserialize<'a>> AuthToken<T> {
    pub fn sign(&self, token_config : &TokenConfig) -> Result<String, jsonwebtoken::errors::Error> {
        jsonwebtoken::encode(&jsonwebtoken::Header::default(), &self.claim, &token_config.encode_key)
    }
}

#[derive(Serialize)]
pub enum AuthError {
    AuthError,
    MissingBearer,
}

#[async_trait]
impl<S: Send+Sync, T: Serialize + for<'a> Deserialize<'a>> FromRequestParts<S>
    for AuthToken<T>
    where 
        TokenConfig: FromRef<S>
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|err| err.into_response())?;

        let token_config = TokenConfig::from_ref(state);
        let token_data: TokenData<T> = decode(bearer.token(), &token_config.decode_key, &Validation::default())
            .map_err(|err| (StatusCode::UNAUTHORIZED, err.to_string()).into_response())?;

        Ok(AuthToken {
            claim: token_data.claims,
        })
    }
}
