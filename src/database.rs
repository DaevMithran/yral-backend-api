use chrono::Utc;
use sqlx::{sqlite::SqliteQueryResult, SqlitePool};
use uuid::Uuid;

use crate::types::{AppError, CreateTodoRequest, Todo, UpdateTodoRequest};

#[derive(Clone)]
pub struct Database {
    db: SqlitePool,
}

impl Database {
    pub async fn new(url: &str) -> Result<Self, AppError> {
        let db = SqlitePool::connect(url).await?;
        Ok(Self { db })
    }

    pub async fn migrate(&self) -> Result<(), AppError> {
        sqlx::migrate!("./src/migrations").run(&self.db).await?;
        Ok(())
    }

    pub async fn create(&self, request: CreateTodoRequest) -> Result<String, AppError> {
        let todo = Todo {
            id: Uuid::new_v4().to_string(),
            data: request.data,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_complete: false,
        };

        sqlx::query(
            "INSERT INTO todos (id, data, created_at, updated_at, is_complete) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(todo.id.clone())
        .bind(todo.data)
        .bind(todo.created_at)
        .bind(todo.updated_at)
        .bind(todo.is_complete)
        .execute(&self.db)
        .await?;

        Ok(todo.id)
    }

    pub async fn update(
        &self,
        todo_id: String,
        request: UpdateTodoRequest,
    ) -> Result<SqliteQueryResult, AppError> {
        let todo = sqlx::query(
            "UPDATE todos SET data = $1, is_complete = $2, updated_at = $3 WHERE id = $4",
        )
        .bind(request.data)
        .bind(request.is_complete)
        .bind(Utc::now())
        .bind(todo_id)
        .execute(&self.db)
        .await?;

        Ok(todo)
    }

    pub async fn delete(&self, todo_id: String) -> Result<bool, AppError> {
        sqlx::query("DELETE FROM todos WHERE id = $1")
            .bind(todo_id)
            .execute(&self.db)
            .await?;

        Ok(true)
    }

    pub async fn list(&self, page: u32, limit: u32) -> Result<Vec<Todo>, AppError> {
        let todos: Vec<Todo> = sqlx::query_as("SELECT * FROM todos LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(page)
            .fetch_all(&self.db)
            .await?;

        Ok(todos)
    }

    pub async fn get(&self, todo_id: String) -> Result<Option<Todo>, AppError> {
        let todo: Option<Todo> = sqlx::query_as("SELECT * FROM todos WHERE id = $1")
            .bind(todo_id)
            .fetch_optional(&self.db)
            .await?;

        Ok(todo)
    }
}
