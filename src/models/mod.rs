use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::errors::AppError;

// ---- School ----

#[derive(Clone, Serialize)]
pub struct School {
    pub id: Uuid,
    pub name: String,
    pub username: String,
    #[serde(skip_serializing)] // never expose password hash in responses
    pub password_hash: String,
}

#[derive(Deserialize)]
pub struct RegisterSchoolRequest {
    pub name: String,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginSchoolRequest {
    pub username: String,
    pub password: String,
}

// ---- Student ----

#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub enum PaymentStatus {
    Paid,
    Pending,
}

#[derive(Clone, Serialize)]
pub struct Student {
    pub id: Uuid,
    pub school_id: Uuid, // ties student to a school
    pub school_name: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub status: PaymentStatus,
    pub department: String,
    pub payment_reference: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateStudentRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub department: String,
}

// ---- AppStore ----

#[derive(Clone)]
pub struct AppStore {
    pub schools: Arc<Mutex<HashMap<String, School>>>,
    pub students: Arc<Mutex<HashMap<String, Student>>>,
}

impl AppStore {
    pub fn new() -> Self {
        Self {
            schools: Arc::new(Mutex::new(HashMap::new())),
            students: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // -- School methods --

    pub async fn register_school(&self, req: RegisterSchoolRequest) -> Result<School, AppError> {
        let mut schools = self.schools.lock().await;

        // check username is not already taken
        let taken = schools.values().any(|s| s.username == req.username);
        if taken {
            return Err(AppError::Conflict("Username already taken".to_string()));
        }

        let password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let school = School {
            id: Uuid::new_v4(),
            name: req.name,
            username: req.username,
            password_hash,
        };

        schools.insert(school.id.to_string(), school.clone());
        Ok(school)
    }

    pub async fn find_school_by_username(&self, username: &str) -> Result<School, AppError> {
        let schools = self.schools.lock().await;
        schools
            .values()
            .find(|s| s.username == username)
            .cloned()
            .ok_or(AppError::NotFound)
    }

    // -- Student methods --

    pub async fn create_student(
        &self,
        school_id: Uuid,
        school_name: String,
        req: CreateStudentRequest,
    ) -> Result<(), AppError> {
        let new_student = Student {
            id: Uuid::new_v4(),
            school_name,
            school_id,
            first_name: req.first_name,
            last_name: req.last_name,
            email: req.email,
            department: req.department,
            status: PaymentStatus::Pending,
            payment_reference: None,
        };

        self.students
            .lock()
            .await
            .insert(new_student.id.to_string(), new_student);

        Ok(())
    }

    pub async fn get_all_students(&self, school_id: Uuid) -> Result<Vec<Student>, AppError> {
        let students = self.students.lock().await;
        Ok(students
            .values()
            .filter(|s| s.school_id == school_id) // only this school's students
            .cloned()
            .collect())
    }

    pub async fn get_student(&self, school_id: Uuid, id: Uuid) -> Result<Student, AppError> {
        let students = self.students.lock().await;
        students
            .values()
            .find(|s| s.id == id && s.school_id == school_id) // must belong to this school
            .cloned()
            .ok_or(AppError::NotFound)
    }

    pub async fn delete_student(&self, school_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let mut students = self.students.lock().await;
        let key = students
            .iter()
            .find(|(_, s)| s.id == id && s.school_id == school_id)
            .map(|(k, _)| k.clone());

        if let Some(k) = key {
            students.remove(&k);
            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }

    pub async fn set_payment_reference(
        &self,
        school_id: Uuid,
        id: Uuid,
        reference: String,
    ) -> Result<(), AppError> {
        let mut students = self.students.lock().await;
        let student = students
            .values_mut()
            .find(|s| s.id == id && s.school_id == school_id);

        if let Some(student) = student {
            student.payment_reference = Some(reference);
            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }

    pub async fn mark_student_paid_by_reference(&self, reference: &str) -> Result<(), AppError> {
        let mut students = self.students.lock().await;
        let student = students
            .values_mut()
            .find(|s| s.payment_reference.as_deref() == Some(reference));

        if let Some(student) = student {
            student.status = PaymentStatus::Paid;
            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }
}