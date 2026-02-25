use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::errors::AppError;

#[derive(Clone, Deserialize, Serialize)]
pub enum PaymentStatus {
    Paid,
    Pending,
}

#[derive(Clone, Serialize)]
pub struct Student {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub status: PaymentStatus,
    pub department: String,
}

#[derive(Deserialize)]
pub struct CreateStudentRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub department: String,
}

#[derive(Clone)]
pub struct AppStore {
    pub students: Arc<Mutex<HashMap<String, Student>>>,
}

impl AppStore {
    pub fn new() -> Self {
        Self {
            students: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create_student(&self, student: CreateStudentRequest) -> Result<(), AppError> {
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

    pub async fn get_all_students(&self) -> Result<Vec<Student>, AppError> {
        Ok(self.students.lock().await.values().cloned().collect())
    }

    pub async fn delete_student(&self, id: Uuid) -> Result<(), AppError> {
        self.students.lock().await.remove(&id.to_string());
        Ok(())
    }

    pub async fn get_student(&self, id: Uuid) -> Result<Student, AppError> {
        if let Some(student) = self.students.lock().await.get(&id.to_string()) {
            Ok(student.clone())
        } else {
            Err(AppError::NotFound)
        }
    }
}
