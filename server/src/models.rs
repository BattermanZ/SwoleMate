use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Workout {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub date: DateTime<Utc>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Exercise {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub workout_id: i64,
    pub exercise_type: String,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Set {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub exercise_id: i64,
    pub reps: i64,
    pub weight: f64,
    pub notes: Option<String>,
} 