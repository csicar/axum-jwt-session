use axum::{
    extract::{FromRef, State},
    routing::{get, post},
    Router,
};
use axum_jwt_session::{AuthToken, TokenConfig};
use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};

#[derive(Clone, FromRef)]
struct AppState {
    token_config: TokenConfig,
}

#[tokio::main]
async fn main() {
    // created our signing keys
    let jwt_secret = "eEUHdrTCvH/IlAiybLJS+2KBN1kp0pW9UwjnBxN+kvRlqk+8y++5sulaxinIQ/xg";
    let token_config = TokenConfig {
        encode_key: EncodingKey::from_base64_secret(jwt_secret).unwrap(),
        decode_key: DecodingKey::from_base64_secret(jwt_secret).unwrap(),
    };

    // put the keys in our app state
    let app_state = AppState { token_config };

    // build our application
    let app = Router::new()
        .route("/restricted", get(restricted))
        .route("/login", post(login))
        .with_state(app_state);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Serialize, Deserialize)]
pub struct UserClaim {
    pub email: String,
    pub exp: u64,
}

// this route is only accessible to clients with a JWT created with our signing key
async fn restricted(AuthToken { claim }: AuthToken<UserClaim>) -> String {
    format!(
        "Hello {}, you have access to the restricted area",
        claim.email
    )
}

// this route returns a valid JWT token
async fn login(State(token_config): State<TokenConfig>) -> String {
    // TODO: extract user credentials and validate them here...

    let auth_token = AuthToken {
        claim: UserClaim {
            email: "mymail@test.test".into(),
            exp: jsonwebtoken::get_current_timestamp() +  120 * 24 * 60 * 60, /* in seconds */
        },
    };
    let jwt = auth_token
        .sign(&token_config)
        .expect("Unexpected signing error");
    jwt
}
