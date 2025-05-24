use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    #[serde(rename = "_id")]
    pub id: i32,
    pub username: String,
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            first_name: user.first_name,
            last_name: user.last_name,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct SignupRequest {
    #[validate(email)]
    pub username: String,
    pub password: String,
    #[serde(alias = "firstName")]
    pub first_name: String,
    #[serde(alias = "lastName")]
    pub last_name: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SigninRequest {
    #[validate(email)]
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    pub password: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BulkUserResponse {
    pub user: Vec<UserResponse>,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}
