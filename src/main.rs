use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Result,
    routing::{get, patch, post},
    Json, Router,
};
use database::Database;
use sqlx::{migrate::MigrateDatabase, Sqlite};
use types::{
    AppError, CreateTodoRequest, GetTodosQuery, GetTodosResponse, Todo, UpdateTodoRequest,
};
mod database;
mod types;

// functions
async fn get_todos(
    Query(params): Query<GetTodosQuery>,
    State(db): State<Database>,
) -> Result<Json<GetTodosResponse>, StatusCode> {
    let page = params.page.unwrap_or(0);
    let limit = params.limit.unwrap_or(10).min(50);

    let todos = db
        .list(page, limit)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Pagination
    let total = todos.len();

    Ok(Json(GetTodosResponse { todos, total }))
}

async fn get_todo(
    Path(todo_id): Path<String>,
    State(db): State<Database>,
) -> Result<Json<Todo>, StatusCode> {
    let todo = db
        .get(todo_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match todo {
        Some(res) => Ok(Json(res)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_todo(
    State(db): State<Database>,
    Json(payload): Json<CreateTodoRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let todo_id = db
        .create(payload)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "todo_id": todo_id
    })))
}

async fn update_todo(
    Path(todo_id): Path<String>,
    State(db): State<Database>,
    Json(payload): Json<UpdateTodoRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match db.update(todo_id.clone(), payload).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "todo_id": todo_id
        }))),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn delete_todo(
    Path(todo_id): Path<String>,
    State(db): State<Database>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match db.delete(todo_id.clone()).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "todo_id": todo_id
        }))),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let db_path = "sqlite:todos.db";

    if !Sqlite::database_exists(&db_path).await.unwrap_or(false) {
        match Sqlite::create_database(&db_path).await {
            Ok(_) => println!("Database created"),
            Err(e) => println!("Error creating database: {}", e),
        }
    } else {
        println!("Database already exists");
    }

    let db = Database::new(db_path).await?;

    db.migrate().await?;
    println!("Database migrations completed successfully");

    // Router
    let app = Router::new()
        .route("/api/todos", get(get_todos).post(create_todo))
        .route(
            "/api/todos/{id}",
            get(get_todo).delete(delete_todo).patch(update_todo),
        )
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .map_err(|err| AppError::Internal(err))?;

    println!("Server starting on http://0.0.0.0:3000");

    axum::serve(listener, app)
        .await
        .map_err(|err| AppError::Internal(err))?;

    Ok(())
}
