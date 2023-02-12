use std::marker::PhantomData;

use async_trait::async_trait;
use axum::{
    extract::{FromRequest, FromRef, FromRequestParts},
    headers::{authorization::Bearer, Authorization},
    http::{Request, StatusCode, request::Parts},
    response::{IntoResponse, Response},
    Extension, TypedHeader,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, TokenData, Validation, EncodingKey};
use serde::{Deserialize, Serialize};

pub struct AuthToken<T: Serialize + for<'a> Deserialize<'a>> {
    pub claim: T,
}

#[derive(Clone)]
pub struct TokenConfig {
    pub decode_key: DecodingKey,
    pub encode_key: EncodingKey
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
            .map_err(|err| err.to_string().into_response())?;

        Ok(AuthToken {
            claim: token_data.claims,
        })
    }
}
