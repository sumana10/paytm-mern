use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    http::header,
    web, Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::future::{ready, Ready};

use crate::config::AppConfig;
use crate::models::user::Claims;

pub struct Auth;

impl<S, B> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware { service }))
    }
}

pub struct AuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_header = req.headers().get(header::AUTHORIZATION);

        let auth_header = match auth_header {
            Some(h) => h.to_str().unwrap_or(""),
            None => {
                return Box::pin(async move {
                    Err(ErrorUnauthorized("No authorization header"))
                });
            }
        };

        if !auth_header.starts_with("Bearer ") {
            return Box::pin(async move {
                Err(ErrorUnauthorized("Invalid authorization format"))
            });
        }

        let token = &auth_header[7..];

        let config = match req.app_data::<web::Data<AppConfig>>() {
            Some(c) => c.clone(),
            None => {
                return Box::pin(async move {
                    Err(ErrorUnauthorized("Server configuration error"))
                });
            }
        };

        let token_data = match decode::<Claims>(
            token,
            &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
            &Validation::default(),
        ) {
            Ok(t) => t,
            Err(_) => {
                return Box::pin(async move {
                    Err(ErrorUnauthorized("Invalid token"))
                });
            }
        };

        let user_id = match token_data.claims.sub.parse::<i32>() {
            Ok(id) => id,
            Err(_) => {
                return Box::pin(async move {
                    Err(ErrorUnauthorized("Invalid user ID in token"))
                });
            }
        };

        req.extensions_mut().insert(user_id);

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}

pub fn get_user_id(req: &ServiceRequest) -> Result<i32, Error> {
    req.extensions()
        .get::<i32>()
        .copied()
        .ok_or_else(|| ErrorUnauthorized("User not authenticated"))
}

pub async fn auth_middleware(
    req: ServiceRequest,
    jwt_secret: String,
) -> Result<i32, Error> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let auth_header = match auth_header {
        Some(h) => h,
        None => return Err(ErrorUnauthorized("No authorization header")),
    };

    if !auth_header.starts_with("Bearer ") {
        return Err(ErrorUnauthorized("Invalid authorization format"));
    }

    let token = &auth_header[7..];

    let token_data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(t) => t,
        Err(_) => return Err(ErrorUnauthorized("Invalid token")),
    };

    let user_id = match token_data.claims.sub.parse::<i32>() {
        Ok(id) => id,
        Err(_) => return Err(ErrorUnauthorized("Invalid user ID in token")),
    };

    Ok(user_id)
}
