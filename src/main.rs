use std::{clone, collections::HashMap, sync::Arc};

use axum::{Router, routing::get};
use thiserror::Error;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Error)]
enum AppError {
    #[error("Resource not found")]
    NotFound,
    #[error("Internal Server Error: {0}")]
    InternalServerError(String),
    #[error("Invalid Input, cannot be processed: {field} - {message}")]
    UnProcessableEntity { field: String, message: String },
    #[error("Environement Variable is missing: {0}")]
    MissingEnvironmentVarible(String),
    #[error("Failed to Parse: {0}")]
    ParsingError(String),
}

fn get_env_vars<T>(key: String) -> Result<T, AppError>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let value = std::env::var(&key).map_err(|_| AppError::MissingEnvironmentVarible(key))?;
    value
        .parse::<T>()
        .map_err(|err| AppError::ParsingError(err.to_string()))
}

#[derive(Clone)]
enum PaymentStatus {
    Paid,
    Pending,
}

#[derive(Clone)]
struct Student {
    id: Uuid,
    first_name: String,
    last_name: String,
    email: String,
    status: PaymentStatus,
    department: String,
}

struct AppStore {
    students: Arc<Mutex<HashMap<String, Student>>>,
}

impl AppStore {
    fn new() -> Self {
        Self {
            students: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn create_student(&self, student: Student) -> Result<(), AppError> {
        let new_student = Student {
            id: Uuid::new_v4(),
            first_name: student.first_name,
            last_name: student.last_name,
            email: student.email,
            department: student.department,
            status: PaymentStatus::Pending,
        };

        self.students
            .lock()
            .await
            .insert(new_student.id.to_string(), new_student);

        Ok(())
    }

    async fn get_all_students(&self) -> Result<Vec<Student>, AppError> {
        Ok(self.students.lock().await.values().cloned().collect())
    }

    async fn delete_student(&self, id: Uuid) -> Result<(), AppError> {
        self.students.lock().await.remove(&id.to_string());
        Ok(())
    }

    
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { "Hello from Axum! 🦀" }));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("Listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
