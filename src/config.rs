#[derive(Clone)]
pub struct Config {
    pub host: String,
    pub port: String,
    pub db_url: String,
}

impl Config {
    pub fn from_env() -> Config {
        dotenvy::dotenv().ok();
        Config {
            host: std::env::var("HOST").unwrap_or("0.0.0.0".to_string()),
            port: std::env::var("PORT").unwrap_or("3000".to_string()),
            db_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        }
    }
}