use tracing::warn;

pub struct AppConfig {
    pub database_url: String,
    pub server_port: u16,
    pub server_host: String,
    /// HMAC secret used to sign identity cookies.
    pub cookie_secret: Vec<u8>,
    /// Whether to set the `Secure` attribute on the identity cookie. Must be
    /// `true` in production (HTTPS); `false` for local HTTP development.
    pub cookie_secure: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("Missing DATABASE_URL env var");
        let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "13252".into());
        let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "localhost".into());

        let cookie_secret = match std::env::var("COOKIE_SECRET") {
            Ok(secret) if !secret.is_empty() => secret.into_bytes(),
            _ => {
                warn!(
                    "COOKIE_SECRET is unset; generating a random secret. Existing identity \
                     cookies will be invalidated on every restart. Set COOKIE_SECRET in prod."
                );
                rand::random::<[u8; 32]>().to_vec()
            }
        };

        let cookie_secure = std::env::var("COOKIE_SECURE")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        AppConfig {
            database_url,
            server_port: port.parse().expect("Invalid port. Must be an integer."),
            server_host: host,
            cookie_secret,
            cookie_secure,
        }
    }
}
