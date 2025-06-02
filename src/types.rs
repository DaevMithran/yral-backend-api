use std::io::Error;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Error as AxumError, Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::migrate::MigrateError;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Todo {
    pub id: String,
    pub data: String,
    pub is_complete: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTodoRequest {
    #[validate(length(min = 1, max = 1000, message = "Data must be below 1000 characters"))]
    pub data: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTodoRequest {
    #[validate(length(min = 1, max = 1000, message = "Data must be below 1000 characters"))]
    pub data: String,
    pub is_complete: bool,
}

#[derive(Debug, Serialize)]
pub struct GetTodoResponse {
    pub data: String,
}

#[derive(Debug, Serialize)]
pub struct GetTodosResponse {
    pub todos: Vec<Todo>,
    pub total: usize,
}

#[derive(Debug, Deserialize)]
pub struct GetTodosQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug)]
pub enum AppError {
    Database(sqlx::Error),
    Migration(MigrateError),
    Internal(Error),
    Axum(AxumError),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err)
    }
}

impl From<MigrateError> for AppError {
    fn from(err: MigrateError) -> Self {
        AppError::Migration(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, err_msg) = match self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error occurred"),
        };
        let body = Json(json!({
            "error": err_msg,
            "status":  status.as_u16()
        }));

        (status, body).into_response()
    }
}
