use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenvy::dotenv().ok();

        let host = env::var("APP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("APP_PORT")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(3000);

        let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        Ok(Self {
            host,
            port,
            log_level,
        })
    }
}
