use sqlx::PgPool;

pub struct AppConfig {
    pub db_pool: PgPool,
    pub jwt_secret: String,
}

pub struct AppSettings {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub token_expiration_days: i64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            database_url: "postgres://postgres:password@localhost:5432/userdb".to_string(),
            jwt_secret: "your_jwt_secret_key".to_string(),
            token_expiration_days: 7,
        }
    }
}

pub fn load_settings() -> AppSettings {
    use std::env;
    
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your_jwt_secret_key".to_string());
    let token_expiration_days = env::var("TOKEN_EXPIRATION_DAYS")
        .ok()
        .and_then(|d| d.parse::<i64>().ok())
        .unwrap_or(7);
    
    AppSettings {
        host,
        port,
        database_url,
        jwt_secret,
        token_expiration_days,
    }
}
