use actix_web::{web, HttpResponse, Responder};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::Row;
use validator::Validate;
use crate::config::AppConfig;
use crate::models::user::{
    BulkUserResponse, Claims, MessageResponse, SigninRequest, SignupRequest, TokenResponse,
    UpdateUserRequest, UserResponse,
};
use crate::middleware::Auth;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/user")
            // Public routes
            .route("/signup", web::post().to(signup))
            .route("/signin", web::post().to(signin))
            .route("/bulk", web::get().to(get_users_bulk))
            // Protected routes with individual middleware
            .route("/update", web::put().to(update_user).wrap(Auth))
    );
}



async fn signup(
    app_config: web::Data<AppConfig>,
    user_data: web::Json<SignupRequest>,
) -> impl Responder {
    if let Err(_) = user_data.validate() {
        return HttpResponse::BadRequest().json(MessageResponse {
            message: "Email already taken / Incorrect inputs......".to_string(),
        });
    }

    let existing_user = sqlx::query("SELECT id FROM users WHERE username = $1")
        .bind(&user_data.username)
        .fetch_optional(&app_config.db_pool)
        .await;

    if let Ok(Some(_)) = existing_user {
        return HttpResponse::BadRequest().json(MessageResponse {
            message: "Email already taken/Incorrect inputs exist".to_string(),
        });
    }

    let user_result = sqlx::query(
        "INSERT INTO users (username, password, first_name, last_name)
         VALUES ($1, $2, $3, $4)
         RETURNING id"
    )
    .bind(&user_data.username)
    .bind(&user_data.password)
    .bind(&user_data.first_name)
    .bind(&user_data.last_name)
    .fetch_one(&app_config.db_pool)
    .await;

    match user_result {
        Ok(row) => {
            let user_id: i32 = row.get("id");
            let balance = 1.0 + rand::random::<f64>() * 10000.0;
            let account_result = sqlx::query("INSERT INTO accounts (user_id, balance) VALUES ($1, $2)")
                .bind(user_id)
                .bind(balance)
                .execute(&app_config.db_pool)
                .await;

            if account_result.is_err() {
                return HttpResponse::InternalServerError().json(MessageResponse {
                    message: "Error creating account".to_string(),
                });
            }

            let claims = Claims {
                sub: user_id.to_string(),
                exp: (Utc::now() + Duration::days(7)).timestamp() as usize,
            };
            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(app_config.jwt_secret.as_bytes()),
            )
            .unwrap_or_default();

            HttpResponse::Ok().json(TokenResponse {
                message: Some("User created successfully".to_string()),
                token,
            })
        }
        Err(_) => HttpResponse::InternalServerError().json(MessageResponse {
            message: "Database error".to_string(),
        }),
    }
}

async fn signin(
    app_config: web::Data<AppConfig>,
    login_data: web::Json<SigninRequest>,
) -> impl Responder {
    if let Err(_) = login_data.validate() {
        return HttpResponse::BadRequest().json(MessageResponse {
            message: "Email already taken / Incorrect inputs".to_string(),
        });
    }

    let user_result = sqlx::query(
        "SELECT id FROM users
         WHERE username = $1 AND password = $2"
    )
    .bind(&login_data.username)
    .bind(&login_data.password)
    .fetch_optional(&app_config.db_pool)
    .await;

    match user_result {
        Ok(Some(row)) => {
            let user_id: i32 = row.get("id");
            let claims = Claims {
                sub: user_id.to_string(),
                exp: (Utc::now() + Duration::days(7)).timestamp() as usize,
            };
            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(app_config.jwt_secret.as_bytes()),
            )
            .unwrap_or_default();

            HttpResponse::Ok().json(TokenResponse {
                message: None,
                token,
            })
        }
        _ => HttpResponse::BadRequest().json(MessageResponse {
            message: "Error while logging in".to_string(),
        }),
    }
}

async fn update_user(
    app_config: web::Data<AppConfig>,
    update_data: web::Json<UpdateUserRequest>,
    user_id: web::ReqData<i32>,
) -> impl Responder {
    if let Err(_) = update_data.validate() {
        return HttpResponse::BadRequest().json(MessageResponse {
            message: "Error while updating information".to_string(),
        });
    }

    let mut updates = Vec::new();
    let mut params = Vec::new();
    let mut param_index = 1;
    
    if let Some(password) = &update_data.password {
        updates.push(format!("password = ${}", param_index));
        params.push(password.clone());
        param_index += 1;
    }
    
    if let Some(first_name) = &update_data.first_name {
        updates.push(format!("first_name = ${}", param_index));
        params.push(first_name.clone());
        param_index += 1;
    }
    
    if let Some(last_name) = &update_data.last_name {
        updates.push(format!("last_name = ${}", param_index));
        params.push(last_name.clone());
        param_index += 1;
    }
    
    if updates.is_empty() {
        return HttpResponse::BadRequest().json(MessageResponse {
            message: "No fields to update".to_string(),
        });
    }

    let query_str = format!(
        "UPDATE users SET {} WHERE id = ${}",
        updates.join(", "),
        param_index
    );
    
    let result = match params.len() {
        1 => sqlx::query(&query_str)
            .bind(&params[0])
            .bind(*user_id)
            .execute(&app_config.db_pool)
            .await,
        2 => sqlx::query(&query_str)
            .bind(&params[0])
            .bind(&params[1])
            .bind(*user_id)
            .execute(&app_config.db_pool)
            .await,
        3 => sqlx::query(&query_str)
            .bind(&params[0])
            .bind(&params[1])
            .bind(&params[2])
            .bind(*user_id)
            .execute(&app_config.db_pool)
            .await,
        _ => {
            return HttpResponse::InternalServerError().json(MessageResponse {
                message: "Too many parameters for update".to_string(),
            });
        }
    };

    match result {
        Ok(_) => HttpResponse::Ok().json(MessageResponse {
            message: "Updated successfully".to_string(),
        }),
        Err(e) => {
            eprintln!("Error updating user: {:?}", e);
            HttpResponse::InternalServerError().json(MessageResponse {
                message: "Error while updating information".to_string(),
            })
        },
    }
}

async fn get_users_bulk(
    app_config: web::Data<AppConfig>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let filter = query.get("filter").unwrap_or(&String::new()).clone();
    
    let users_result = sqlx::query(
        "SELECT id, username, first_name, last_name
         FROM users
         WHERE first_name ILIKE $1 OR last_name ILIKE $1"
    )
    .bind(format!("%{}%", filter))
    .fetch_all(&app_config.db_pool)
    .await;

    match users_result {
        Ok(rows) => {
            let users: Vec<UserResponse> = rows
                .iter()
                .map(|row| {
                    let id: i32 = row.get("id");
                    UserResponse {
                        id,
                        username: row.get("username"),
                        first_name: row.get("first_name"),
                        last_name: row.get("last_name"),
                    }
                })
                .collect();
            
            HttpResponse::Ok().json(BulkUserResponse { user: users })
        },
        Err(e) => {
            eprintln!("Error fetching users: {:?}", e);
            HttpResponse::InternalServerError().json(MessageResponse {
                message: "Error fetching users".to_string(),
            })
        },
    }
}
