use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: i32,
    pub user_id: i32,
    pub balance: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub balance: f64,
}

#[derive(Debug, Deserialize, Validate)]
pub struct TransferRequest {
    #[validate(range(min = 0.01))]
    pub amount: f64,
    pub to: i32,
}

#[derive(Debug, Serialize)]
pub struct TransferResponse {
    pub message: String,
}
