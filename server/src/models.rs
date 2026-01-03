use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Workout {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub date: DateTime<Utc>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub notes: Option<String>,
    pub feedback: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Exercise {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub workout_id: i64,
    pub exercise_type: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub notes: Option<String>,
}

/// Represents a set of an exercise
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Set {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub exercise_id: i64,
    pub reps: i64,
    /// Weight in kilograms
    pub weight: f64,
    pub notes: Option<String>,
}

// Request DTOs
#[derive(Debug, Deserialize)]
pub struct CreateWorkoutRequest {
    pub date: DateTime<Utc>,
    pub start_time: DateTime<Utc>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkoutRequest {
    pub end_time: DateTime<Utc>,
    pub notes: Option<String>,
    pub feedback: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateExerciseRequest {
    pub exercise_type: String,
    pub start_time: DateTime<Utc>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateExerciseRequest {
    pub end_time: DateTime<Utc>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSetRequest {
    pub reps: i64,
    pub weight: f64,
    pub notes: Option<String>,
}
