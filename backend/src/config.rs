pub struct AppConfig {
    pub database_url: String,
    pub server_port: u16,
    pub server_host: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("Missing DATABASE_URL env var");
        let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "13252".into());
        let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "localhost".into());

        AppConfig {
            database_url,
            server_port: port.parse().expect("Invalid port. Must be an integer."),
            server_host: host,
        }
    }
}
