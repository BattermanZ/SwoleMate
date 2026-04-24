use crate::{
    db::Database,
    errors::AppError,
    models::{
        CreateWorkoutTemplateFromWorkoutRequest, CreateWorkoutTemplateRequest,
        DuplicateWorkoutTemplateRequest, StartWorkoutFromTemplateRequest,
        UpdateWorkoutTemplateRequest, WorkoutTemplate, WorkoutTemplateDetail,
        WorkoutTemplateExerciseRequest,
    },
};

pub async fn list_templates(db: &Database, user_id: i64) -> Result<Vec<WorkoutTemplate>, AppError> {
    db.list_workout_templates(user_id).await
}

pub async fn get_template(
    db: &Database,
    user_id: i64,
    template_id: i64,
) -> Result<WorkoutTemplateDetail, AppError> {
    db.get_workout_template(user_id, template_id).await
}

pub async fn create_template(
    db: &Database,
    user_id: i64,
    req: &CreateWorkoutTemplateRequest,
) -> Result<WorkoutTemplateDetail, AppError> {
    db.create_workout_template(user_id, req).await
}

pub async fn update_template(
    db: &Database,
    user_id: i64,
    template_id: i64,
    req: &UpdateWorkoutTemplateRequest,
) -> Result<WorkoutTemplateDetail, AppError> {
    db.update_workout_template(user_id, template_id, req).await
}

pub async fn delete_template(
    db: &Database,
    user_id: i64,
    template_id: i64,
) -> Result<(), AppError> {
    db.delete_workout_template(user_id, template_id).await
}

pub async fn duplicate_template(
    db: &Database,
    user_id: i64,
    template_id: i64,
    req: &DuplicateWorkoutTemplateRequest,
) -> Result<WorkoutTemplateDetail, AppError> {
    let original = db.get_workout_template(user_id, template_id).await?;
    let name = req
        .name
        .clone()
        .unwrap_or_else(|| format!("{} Copy", original.template.name));

    let create_req = CreateWorkoutTemplateRequest {
        name,
        exercises: original
            .exercises
            .into_iter()
            .map(|exercise| WorkoutTemplateExerciseRequest {
                exercise_type: exercise.exercise_type,
                notes: exercise.notes,
                per_side_weight: Some(exercise.per_side_weight),
                split_weight: Some(exercise.split_weight),
                settings: Some(
                    exercise
                        .settings
                        .into_iter()
                        .map(|setting| crate::models::ExerciseSettingRequest {
                            key: setting.setting_key,
                            value: setting.setting_value,
                        })
                        .collect(),
                ),
            })
            .collect(),
    };

    db.create_workout_template(user_id, &create_req).await
}

pub async fn create_template_from_workout(
    db: &Database,
    user_id: i64,
    workout_id: i64,
    req: &CreateWorkoutTemplateFromWorkoutRequest,
) -> Result<WorkoutTemplateDetail, AppError> {
    let _workout = db.get_workout(user_id, workout_id).await?;
    let exercises = db.get_exercises_for_workout(user_id, workout_id).await?;

    let create_req = CreateWorkoutTemplateRequest {
        name: req.name.clone(),
        exercises: exercises
            .into_iter()
            .map(|exercise| WorkoutTemplateExerciseRequest {
                exercise_type: exercise.exercise_type,
                notes: exercise.notes,
                per_side_weight: Some(exercise.per_side_weight),
                split_weight: Some(exercise.split_weight),
                settings: Some(
                    exercise
                        .settings
                        .into_iter()
                        .map(|setting| crate::models::ExerciseSettingRequest {
                            key: setting.setting_key,
                            value: setting.setting_value,
                        })
                        .collect(),
                ),
            })
            .collect(),
    };

    db.create_workout_template(user_id, &create_req).await
}

pub async fn start_workout_from_template(
    db: &Database,
    user_id: i64,
    template_id: i64,
    req: &StartWorkoutFromTemplateRequest,
) -> Result<i64, AppError> {
    db.start_workout_from_template(user_id, template_id, req)
        .await
}
