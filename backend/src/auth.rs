use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, request::Parts},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use crate::{error::AppError, routes::AppState};

type HmacSha256 = Hmac<Sha256>;

pub const COOKIE_NAME: &str = "wl_uid";
const COOKIE_MAX_AGE: time::Duration = time::Duration::days(365);

fn sign(secret: &[u8], token: &str) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts a key of any length");
    mac.update(token.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

/// Generate a fresh random token and its signed cookie value `<token>.<hmac>`.
pub fn issue_token(secret: &[u8]) -> (String, String) {
    let token = BASE64_URL_SAFE_NO_PAD.encode(rand::random::<[u8; 16]>());
    let signature = BASE64_URL_SAFE_NO_PAD.encode(sign(secret, &token));
    let cookie_value = format!("{token}.{signature}");
    (token, cookie_value)
}

/// Verify a cookie value, returning the embedded token if the signature is
/// valid. Uses a constant-time comparison so verification does not leak the
/// expected signature via timing.
pub fn verify_token(secret: &[u8], cookie_value: &str) -> Option<String> {
    let (token, signature_b64) = cookie_value.split_once('.')?;
    let signature = BASE64_URL_SAFE_NO_PAD.decode(signature_b64).ok()?;

    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts a key of any length");
    mac.update(token.as_bytes());
    mac.verify_slice(&signature).ok()?;

    Some(token.to_string())
}

/// Build the persistent, HTTP-only identity cookie for a given signed value.
pub fn build_cookie(value: String, secure: bool) -> Cookie<'static> {
    Cookie::build((COOKIE_NAME, value))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(secure)
        .max_age(COOKIE_MAX_AGE)
        .build()
}

/// Best-effort client IP for logging: first hop of `X-Forwarded-For` if present
/// (set by a trusted reverse proxy), otherwise unknown. Never trusted for
/// identity or authorization.
pub fn client_ip(headers: &HeaderMap) -> Option<String> {
    let forwarded = headers.get("x-forwarded-for")?.to_str().ok()?;
    forwarded
        .split(',')
        .next()
        .map(|ip| ip.trim().to_string())
        .filter(|ip| !ip.is_empty())
}

/// User-agent header as a string, for logging only.
pub fn user_agent(headers: &HeaderMap) -> Option<String> {
    headers.get("user-agent")?.to_str().ok().map(str::to_string)
}

/// Extractor that yields the verified opaque user identifier from the signed
/// identity cookie, rejecting the request with `401` if it is missing or the
/// signature does not verify.
pub struct UserId(pub String);

impl FromRequestParts<AppState> for UserId {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let cookie = jar
            .get(COOKIE_NAME)
            .ok_or(AppError::Unauthorized("missing identity cookie"))?;

        let token = verify_token(&state.cookie_secret, cookie.value())
            .ok_or(AppError::Unauthorized("invalid identity cookie"))?;

        Ok(UserId(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &[u8] = b"test-secret-value-not-for-production";

    #[test]
    fn issued_token_verifies() {
        let (token, cookie_value) = issue_token(SECRET);
        assert_eq!(verify_token(SECRET, &cookie_value), Some(token));
    }

    #[test]
    fn tampered_token_is_rejected() {
        let (_token, cookie_value) = issue_token(SECRET);
        let (_, signature) = cookie_value.split_once('.').unwrap();
        let forged = format!("forged-token.{signature}");
        assert_eq!(verify_token(SECRET, &forged), None);
    }

    #[test]
    fn wrong_secret_is_rejected() {
        let (_token, cookie_value) = issue_token(SECRET);
        assert_eq!(verify_token(b"a-different-secret", &cookie_value), None);
    }

    #[test]
    fn malformed_value_is_rejected() {
        assert_eq!(verify_token(SECRET, "no-separator"), None);
        assert_eq!(verify_token(SECRET, "token.not-base64!!"), None);
    }
}
