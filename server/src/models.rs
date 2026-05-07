use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

const MAX_NOTES_LEN: usize = 2000;
const MAX_FEEDBACK_LEN: usize = 64;
const MAX_EXERCISE_TYPE_LEN: usize = 80;
const MAX_TEMPLATE_NAME_LEN: usize = 120;
const MAX_SETTING_KEY_LEN: usize = 64;
const MAX_SETTING_VALUE_LEN: usize = 128;
const MAX_SETTINGS_PER_EXERCISE: usize = 24;
const TRACKING_FIELDS_SETTING_KEY: &str = "_tracking_fields";
const MAX_TEMPLATE_EXERCISES: usize = 64;
const MAX_REPS: i64 = 500;
const MAX_WEIGHT_KG: f64 = 2000.0;
const MAX_DURATION_SECONDS: i64 = 24 * 60 * 60;
const MAX_TIMEZONE_OFFSET_MINUTES: i64 = 14 * 60;
const ALLOWED_FEEDBACK: [&str; 3] = ["😊", "😐", "😞"];

fn validate_opt_len(name: &str, value: &Option<String>, max: usize) -> Result<(), String> {
    let Some(value) = value.as_ref() else {
        return Ok(());
    };
    if value.len() > max {
        return Err(format!("{name} must be at most {max} characters"));
    }
    Ok(())
}

fn validate_opt_opt_len(
    name: &str,
    value: &Option<Option<String>>,
    max: usize,
) -> Result<(), String> {
    let Some(value) = value.as_ref() else {
        return Ok(());
    };
    let Some(value) = value.as_ref() else {
        return Ok(());
    };
    if value.len() > max {
        return Err(format!("{name} must be at most {max} characters"));
    }
    Ok(())
}

fn validate_feedback(value: &Option<String>) -> Result<(), String> {
    let Some(value) = value.as_ref() else {
        return Ok(());
    };
    if ALLOWED_FEEDBACK.iter().any(|allowed| allowed == value) {
        Ok(())
    } else {
        Err("feedback must be one of 😊, 😐, 😞".to_string())
    }
}

fn validate_opt_opt_feedback(value: &Option<Option<String>>) -> Result<(), String> {
    let Some(value) = value.as_ref() else {
        return Ok(());
    };
    validate_feedback(value)
}

fn validate_nonempty_len(name: &str, value: &str, max: usize) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{name} must not be empty"));
    }
    if value.len() > max {
        return Err(format!("{name} must be at most {max} characters"));
    }
    Ok(())
}

fn validate_timezone_offset(offset: &Option<i64>) -> Result<(), String> {
    let Some(offset) = offset else {
        return Ok(());
    };
    if *offset < -MAX_TIMEZONE_OFFSET_MINUTES || *offset > MAX_TIMEZONE_OFFSET_MINUTES {
        return Err("timezone_offset_minutes is out of range".to_string());
    }
    Ok(())
}

fn validate_template_exercises(exercises: &[WorkoutTemplateExerciseRequest]) -> Result<(), String> {
    if exercises.len() > MAX_TEMPLATE_EXERCISES {
        return Err(format!(
            "exercises must have at most {MAX_TEMPLATE_EXERCISES} items"
        ));
    }

    for exercise in exercises {
        validate_nonempty_len(
            "exercise_type",
            &exercise.exercise_type,
            MAX_EXERCISE_TYPE_LEN,
        )?;
        validate_opt_len("notes", &exercise.notes, MAX_NOTES_LEN)?;
        validate_exercise_settings(&exercise.settings)?;
    }

    Ok(())
}

fn validate_exercise_settings(
    settings: &Option<Vec<ExerciseSettingRequest>>,
) -> Result<(), String> {
    let Some(settings) = settings.as_ref() else {
        return Ok(());
    };

    let visible_settings = settings
        .iter()
        .filter(|s| s.key != TRACKING_FIELDS_SETTING_KEY)
        .count();
    if visible_settings > MAX_SETTINGS_PER_EXERCISE {
        return Err(format!(
            "settings must have at most {MAX_SETTINGS_PER_EXERCISE} items"
        ));
    }
    for s in settings {
        validate_nonempty_len("settings.key", &s.key, MAX_SETTING_KEY_LEN)?;
        validate_nonempty_len("settings.value", &s.value, MAX_SETTING_VALUE_LEN)?;
    }
    Ok(())
}

fn validate_f64(name: &str, value: f64, min: f64, max: f64) -> Result<(), String> {
    if !value.is_finite() {
        return Err(format!("{name} must be a finite number"));
    }
    if value < min || value > max {
        return Err(format!("{name} is out of range"));
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Workout {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub date: DateTime<Utc>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub notes: Option<String>,
    pub feedback: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exercise_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone_offset_minutes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_closed_at: Option<DateTime<Utc>>,
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
    #[serde(default)]
    pub per_side_weight: bool,
    #[serde(default)]
    pub split_weight: bool,
    #[serde(default)]
    #[sqlx(skip)]
    pub settings: Vec<ExerciseSetting>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct ExerciseSetting {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub exercise_id: i64,
    #[serde(rename = "key")]
    pub setting_key: String,
    #[serde(rename = "value")]
    pub setting_value: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_left: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_right: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkoutTemplate {
    pub id: i64,
    pub name: String,
    pub exercise_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkoutTemplateExerciseSetting {
    pub id: i64,
    pub template_exercise_id: i64,
    #[serde(rename = "key")]
    pub setting_key: String,
    #[serde(rename = "value")]
    pub setting_value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkoutTemplateExercise {
    pub id: i64,
    pub template_id: i64,
    pub position: i64,
    pub exercise_type: String,
    pub notes: Option<String>,
    #[serde(default)]
    pub per_side_weight: bool,
    #[serde(default)]
    pub split_weight: bool,
    #[serde(default)]
    pub settings: Vec<WorkoutTemplateExerciseSetting>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkoutTemplateDetail {
    pub template: WorkoutTemplate,
    pub exercises: Vec<WorkoutTemplateExercise>,
}

// Request DTOs
#[derive(Debug, Deserialize)]
pub struct CreateWorkoutRequest {
    pub date: DateTime<Utc>,
    pub start_time: DateTime<Utc>,
    pub notes: Option<String>,
    #[serde(default)]
    pub timezone_offset_minutes: Option<i64>,
}

impl CreateWorkoutRequest {
    pub fn validate(&self) -> Result<(), String> {
        validate_opt_len("notes", &self.notes, MAX_NOTES_LEN)?;
        validate_timezone_offset(&self.timezone_offset_minutes)?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkoutRequest {
    pub end_time: DateTime<Utc>,
    pub notes: Option<String>,
    pub feedback: Option<String>,
}

impl UpdateWorkoutRequest {
    pub fn validate(&self) -> Result<(), String> {
        validate_opt_len("notes", &self.notes, MAX_NOTES_LEN)?;
        validate_opt_len("feedback", &self.feedback, MAX_FEEDBACK_LEN)?;
        validate_feedback(&self.feedback)?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkoutTimesRequest {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    #[serde(default)]
    pub notes: Option<Option<String>>,
    #[serde(default)]
    pub feedback: Option<Option<String>>,
}

impl UpdateWorkoutTimesRequest {
    pub fn validate(&self) -> Result<(), String> {
        validate_opt_opt_len("notes", &self.notes, MAX_NOTES_LEN)?;
        validate_opt_opt_len("feedback", &self.feedback, MAX_FEEDBACK_LEN)?;
        validate_opt_opt_feedback(&self.feedback)?;
        if self.end_time < self.start_time {
            return Err("end_time must be greater than or equal to start_time".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateExerciseRequest {
    pub exercise_type: String,
    pub start_time: DateTime<Utc>,
    pub notes: Option<String>,
    #[serde(default)]
    pub per_side_weight: Option<bool>,
    #[serde(default)]
    pub split_weight: Option<bool>,
    #[serde(default)]
    pub settings: Option<Vec<ExerciseSettingRequest>>,
}

impl CreateExerciseRequest {
    pub fn validate(&self) -> Result<(), String> {
        validate_nonempty_len("exercise_type", &self.exercise_type, MAX_EXERCISE_TYPE_LEN)?;
        validate_opt_len("notes", &self.notes, MAX_NOTES_LEN)?;
        validate_exercise_settings(&self.settings)?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateExerciseRequest {
    pub end_time: DateTime<Utc>,
    pub notes: Option<String>,
    #[serde(default)]
    pub per_side_weight: Option<bool>,
    #[serde(default)]
    pub split_weight: Option<bool>,
    #[serde(default)]
    pub settings: Option<Vec<ExerciseSettingRequest>>,
}

impl UpdateExerciseRequest {
    pub fn validate(&self) -> Result<(), String> {
        validate_opt_len("notes", &self.notes, MAX_NOTES_LEN)?;
        validate_exercise_settings(&self.settings)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExerciseSettingRequest {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkoutTemplateExerciseRequest {
    pub exercise_type: String,
    pub notes: Option<String>,
    #[serde(default)]
    pub per_side_weight: Option<bool>,
    #[serde(default)]
    pub split_weight: Option<bool>,
    #[serde(default)]
    pub settings: Option<Vec<ExerciseSettingRequest>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkoutTemplateRequest {
    pub name: String,
    #[serde(default)]
    pub exercises: Vec<WorkoutTemplateExerciseRequest>,
}

impl CreateWorkoutTemplateRequest {
    pub fn validate(&self) -> Result<(), String> {
        validate_nonempty_len("name", &self.name, MAX_TEMPLATE_NAME_LEN)?;
        validate_template_exercises(&self.exercises)?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkoutTemplateRequest {
    pub name: String,
    #[serde(default)]
    pub exercises: Vec<WorkoutTemplateExerciseRequest>,
}

impl UpdateWorkoutTemplateRequest {
    pub fn validate(&self) -> Result<(), String> {
        validate_nonempty_len("name", &self.name, MAX_TEMPLATE_NAME_LEN)?;
        validate_template_exercises(&self.exercises)?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct DuplicateWorkoutTemplateRequest {
    #[serde(default)]
    pub name: Option<String>,
}

impl DuplicateWorkoutTemplateRequest {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(name) = self.name.as_ref() {
            validate_nonempty_len("name", name, MAX_TEMPLATE_NAME_LEN)?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkoutTemplateFromWorkoutRequest {
    pub name: String,
}

impl CreateWorkoutTemplateFromWorkoutRequest {
    pub fn validate(&self) -> Result<(), String> {
        validate_nonempty_len("name", &self.name, MAX_TEMPLATE_NAME_LEN)
    }
}

#[derive(Debug, Deserialize)]
pub struct StartWorkoutFromTemplateRequest {
    pub date: DateTime<Utc>,
    pub start_time: DateTime<Utc>,
    #[serde(default)]
    pub timezone_offset_minutes: Option<i64>,
}

impl StartWorkoutFromTemplateRequest {
    pub fn validate(&self) -> Result<(), String> {
        validate_timezone_offset(&self.timezone_offset_minutes)?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateSetRequest {
    pub reps: i64,
    pub weight: f64,
    #[serde(default)]
    pub duration_seconds: Option<i64>,
    pub notes: Option<String>,
    #[serde(default)]
    pub weight_left: Option<f64>,
    #[serde(default)]
    pub weight_right: Option<f64>,
}

impl CreateSetRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.reps < 0 || self.reps > MAX_REPS {
            return Err(format!("reps must be between 0 and {MAX_REPS}"));
        }
        validate_f64("weight", self.weight, 0.0, MAX_WEIGHT_KG)?;
        if let Some(duration) = self.duration_seconds {
            if duration <= 0 || duration > MAX_DURATION_SECONDS {
                return Err(format!(
                    "duration_seconds must be between 1 and {MAX_DURATION_SECONDS}"
                ));
            }
        }
        if self.reps == 0 && self.duration_seconds.is_none() {
            return Err("sets must include reps or duration_seconds".to_string());
        }
        validate_opt_len("notes", &self.notes, MAX_NOTES_LEN)?;

        let left = self.weight_left;
        let right = self.weight_right;
        if left.is_some() ^ right.is_some() {
            return Err("weight_left and weight_right must be provided together".to_string());
        }
        if let Some(v) = left {
            validate_f64("weight_left", v, 0.0, MAX_WEIGHT_KG)?;
        }
        if let Some(v) = right {
            validate_f64("weight_right", v, 0.0, MAX_WEIGHT_KG)?;
        }
        Ok(())
    }
}
