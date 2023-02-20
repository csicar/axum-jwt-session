# Axum Extractor for JWT Tokens (WIP)

This crate exposes the necessary extractors for `HS256` symmetric HMAC-based sessions.

## Usage

### Setup the Keys
JWTs require a secret for encoding and decoding. These keys should be accessible to your handler. This crate exposes the `TokenConfig` type for this purpose.

Add a `TokenConfig` field to your AppState:

```rust
struct AppState {
  // ...
  token_config: TokenConfig
}
```

and initialize it when setting up axum:

```rust
fn main() {
  ...
  let jwt_secret = ...;
  let token_config = TokenConfig {
      encode_key: EncodingKey::from_base64_secret(jwt_secret).unwrap(),
      decode_key: DecodingKey::from_base64_secret(jwt_secret).unwrap(),
  };
  ...
}
```

### Sign a JWT

First we need to define what claim we like to hand to the user. In this case, the claim just contains the email of the user. The `exp` field is necessary (might become optional later).

```rust
#[derive(Serialize, Deserialize)]
pub struct UserClaim {
    pub email: String,
    pub exp: u64,
}
```

Next, we can create this claim and construct a JWT token using our JWT keys.

```rust
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
```

This handler returns a signed jwt containing the email address.

### Validate a JWT

When extracting your `AuthToken`, this crate will automatically decode and validate the JWT Token signature contained in the `Authorization: Bearer` header against the `TokenConfig` from your `AppState`. 

Making a Route restricted to user group therefore only involves adding the appropriate extractor:

```rust
async fn restricted(AuthToken { claim }: AuthToken<UserClaim>) -> String {
    format!(
        "Hello {}, you have access to the restricted area",
        claim.email
    )
}
```

## Example

see `jwt_session`

Usage:

```bash
$ INVALID_TOKEN =
$ cargo run --example jwt_session
$ curl -H "Authorization: Bearer $INVALID_TOKEN" -i localhost:3000/restricted
HTTP/1.1 401 Unauthorized
content-type: text/plain; charset=utf-8
content-length: 16

InvalidSignature
$ curl -X POST localhost:3000/login
eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJlbWFpbCI6Im15bWFpbEB0ZXN0LnRlc3QiLCJleHAiOjE2ODcyMTg2Mzd9.yuj_fw7JMQdWPIdmVM7lZ_pVvWhz8K8syxDh5amdC5o
$ curl -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJlbWFpbCI6Im15bWFpbEB0ZXN0LnRlc3QiLCJleHAiOjE2ODcyMTg2Mzd9.yuj_fw7JMQdWPIdmVM7lZ_pVvWhz8K8syxDh5amdC5o" -i localhost:3000/restricted
Hello mymail@test.test, you have access to the restricted area
```