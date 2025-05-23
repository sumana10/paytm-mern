mod config;
mod middleware;
mod models;
mod routes;

use actix_web::{web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, Row};
use std::env;
use crate::config::AppConfig;
use crate::routes::{user_routes, account_routes};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your_jwt_secret_key".to_string());
    
    println!("Connecting to database: {}", database_url);
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");
    
    println!("Testing database connection...");
    match sqlx::query("SELECT current_database(), current_schema(), current_user")
        .fetch_one(&pool)
        .await {
            Ok(row) => {
                let db: &str = row.get(0);
                let schema: &str = row.get(1);
                let user: &str = row.get(2);
                println!("Connected to database: {}, schema: {}, as user: {}", db, schema, user);
            },
            Err(e) => {
                eprintln!("Database connection test failed: {}", e);
                panic!("Database connection test failed");
            }
        };
    
    println!("Creating users table if it doesn't exist...");
    match sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            username VARCHAR(255) NOT NULL UNIQUE,
            password VARCHAR(255) NOT NULL,
            first_name VARCHAR(255) NOT NULL,
            last_name VARCHAR(255) NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"
    )
    .execute(&pool)
    .await {
        Ok(_) => println!("Users table ready"),
        Err(e) => {
            eprintln!("Failed to create users table: {}", e);
            panic!("Database setup failed");
        }
    };
    
    println!("Creating accounts table if it doesn't exist...");
    match sqlx::query(
        "CREATE TABLE IF NOT EXISTS accounts (
            id SERIAL PRIMARY KEY,
            user_id INTEGER NOT NULL REFERENCES users(id),
            balance DOUBLE PRECISION NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"
    )
    .execute(&pool)
    .await {
        Ok(_) => println!("Accounts table ready"),
        Err(e) => {
            eprintln!("Failed to create accounts table: {}", e);
            panic!("Database setup failed");
        }
    };
    
    println!("Verifying tables exist...");
    match sqlx::query(
        "SELECT table_name FROM information_schema.tables 
         WHERE table_schema = 'public' AND table_type = 'BASE TABLE'"
    )
    .fetch_all(&pool)
    .await {
        Ok(rows) => {
            println!("Tables in database:");
            for row in rows {
                let table: &str = row.get(0);
                println!("  - {}", table);
            }
        },
        Err(e) => {
            eprintln!("Failed to list tables: {}", e);
        }
    };
    
    println!("Database setup complete!");
    
    let app_config = web::Data::new(AppConfig {
        db_pool: pool,
        jwt_secret: jwt_secret.clone(),
    });
    
    println!("Starting server at http://127.0.0.1:3000");
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_config.clone())
            .route("/health", web::get().to(|| async { 
                println!("Health check called");
                HttpResponse::Ok().body("Server is running") 
            }))
            .configure(user_routes::configure)
            .configure(account_routes::configure)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
