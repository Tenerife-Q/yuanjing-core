use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub db_path: String,
    pub key_path: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("PORT must be a number"),
            db_path: env::var("DB_PATH").unwrap_or_else(|_| "data/db/mmr_db".to_string()),
            key_path: env::var("KEY_PATH").unwrap_or_else(|_| "yuanjing.key".to_string()),
        }
    }
}
