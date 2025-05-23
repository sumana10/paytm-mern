use actix_web::{web, HttpResponse, Responder};
use sqlx::{Postgres, Row, Transaction};
use validator::Validate;
use crate::config::AppConfig;
use crate::models::account::{BalanceResponse, TransferRequest, TransferResponse};
use crate::models::user::MessageResponse;
use crate::middleware::auth::Auth;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .wrap(Auth)
            .route("/balance", web::get().to(get_balance))
            .route("/transfer", web::post().to(transfer))
    );
}

async fn get_balance(
    app_config: web::Data<AppConfig>,
) -> impl Responder {
    HttpResponse::Ok().json(BalanceResponse { 
        balance: 1000.0 
    })
}

async fn perform_transfer(
    mut tx: Transaction<'_, Postgres>,
    from_user_id: i32,
    to_user_id: i32,
    amount: f64,
) -> Result<(), String> {
    let sender_result = sqlx::query(
        "UPDATE accounts 
         SET balance = balance - $1 
         WHERE user_id = $2 AND balance >= $1 
         RETURNING id, balance"
    )
    .bind(amount)
    .bind(from_user_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        eprintln!("Error updating sender account: {:?}", e);
        "Database error".to_string()
    })?;

    if sender_result.is_none() {
        return Err("Insufficient balance".to_string());
    }

    let recipient_result = sqlx::query(
        "UPDATE accounts 
         SET balance = balance + $1 
         WHERE user_id = $2 
         RETURNING id"
    )
    .bind(amount)
    .bind(to_user_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        eprintln!("Error updating recipient account: {:?}", e);
        "Database error".to_string()
    })?;

    if recipient_result.is_none() {
        return Err("Invalid account".to_string());
    }

    tx.commit()
        .await
        .map_err(|e| {
            eprintln!("Error committing transaction: {:?}", e);
            "Failed to commit transaction".to_string()
        })?;

    Ok(())
}

pub(crate) async fn transfer(
    app_config: web::Data<AppConfig>,
    user_id: web::ReqData<i32>,
    transfer_data: web::Json<TransferRequest>,
) -> impl Responder {
    if let Err(_) = transfer_data.validate() {
        return HttpResponse::BadRequest().json(MessageResponse {
            message: "Invalid transfer request".to_string(),
        });
    }

    let tx = match app_config.db_pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Failed to start transaction: {:?}", e);
            return HttpResponse::InternalServerError().json(MessageResponse {
                message: "Failed to start transaction".to_string(),
            });
        }
    };

    let result = perform_transfer(
        tx,
        *user_id,
        transfer_data.to,
        transfer_data.amount,
    )
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(TransferResponse {
            message: "Transfer successful".to_string(),
        }),
        Err(msg) => HttpResponse::BadRequest().json(MessageResponse {
            message: msg,
        }),
    }
}
