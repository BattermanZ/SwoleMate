use actix_web::{test, web, App};
use chrono::Timelike;
use once_cell::sync::Lazy;
use serde_json::{json, Value};
use sqlx::sqlite::SqlitePoolOptions;
use std::path::Path;
use std::sync::Mutex;
use tempfile::TempDir;

use swolemate_server::{db::Database, routes, schema};

static TEST_ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

struct TestEnv {
    _lock: std::sync::MutexGuard<'static, ()>,
    prev_dir: std::path::PathBuf,
    _temp_dir: TempDir,
    prev_database_url: Option<String>,
}

impl TestEnv {
    fn new() -> Self {
        let lock = match TEST_ENV_LOCK.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let prev_dir = std::env::current_dir().expect("current_dir");
        let temp_dir = tempfile::tempdir().expect("tempdir");

        std::fs::create_dir_all(temp_dir.path().join("database")).expect("database dir");
        std::fs::create_dir_all(temp_dir.path().join("logs")).expect("logs dir");
        std::fs::create_dir_all(temp_dir.path().join("backups")).expect("backups dir");
        std::fs::File::create(temp_dir.path().join("database").join("swolemate.db"))
            .expect("create db file");

        std::env::set_current_dir(temp_dir.path()).expect("set_current_dir");

        let prev_database_url = std::env::var("DATABASE_URL").ok();
        std::env::set_var("DATABASE_URL", "sqlite:database/swolemate.db");

        Self {
            _lock: lock,
            prev_dir,
            _temp_dir: temp_dir,
            prev_database_url,
        }
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.prev_dir);
        if let Some(prev) = self.prev_database_url.take() {
            std::env::set_var("DATABASE_URL", prev);
        } else {
            std::env::remove_var("DATABASE_URL");
        }
    }
}

async fn setup_test_app() -> (
    Database,
    impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite:database/swolemate.db")
        .await
        .expect("connect sqlite");

    schema::setup_schema(&pool).await.expect("setup_schema");

    for pragma in [
        "PRAGMA foreign_keys = ON",
        "PRAGMA journal_mode = WAL",
        "PRAGMA synchronous = NORMAL",
        "PRAGMA busy_timeout = 5000",
    ] {
        sqlx::query(pragma).execute(&pool).await.expect("pragma");
    }

    let database = Database::new(pool);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(database.clone()))
            .configure(routes::config),
    )
    .await;

    (database, app)
}

async fn json_body(resp: actix_web::dev::ServiceResponse) -> Value {
    let bytes = test::read_body(resp).await;
    serde_json::from_slice(&bytes).expect("valid json response")
}

#[actix_web::test]
async fn health_check_works() {
    let _env = TestEnv::new();
    let (_, app) = setup_test_app().await;

    let req = test::TestRequest::get().uri("/api/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body = json_body(resp).await;
    assert_eq!(body["status"], "healthy");
    assert!(body.get("timestamp").is_some());
}

#[actix_web::test]
async fn workout_and_exercise_flow_works() {
    let _env = TestEnv::new();
    let (_, app) = setup_test_app().await;

    let now = chrono::Utc::now();

    let req = test::TestRequest::post()
        .uri("/api/workouts")
        .set_json(json!({
            "date": now,
            "start_time": now,
            "notes": "test workout",
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let body = json_body(resp).await;
    let workout_id = body["id"].as_i64().expect("workout id");

    let req = test::TestRequest::put()
        .uri(&format!("/api/workouts/{workout_id}/end"))
        .set_json(json!({
            "end_time": now + chrono::Duration::minutes(45),
            "notes": "ended",
            "feedback": "😊",
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::post()
        .uri(&format!("/api/workouts/{workout_id}/exercises"))
        .set_json(json!({
            "exercise_type": "Bench Press",
            "start_time": now,
            "notes": "warmup",
            "split_weight": true,
            "settings": [
                {"key": "bench_angle", "value": "15"},
                {"key": "seat_height", "value": "3"}
            ]
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let body = json_body(resp).await;
    let exercise_id = body["id"].as_i64().expect("exercise id");

    let req = test::TestRequest::post()
        .uri(&format!("/api/exercises/{exercise_id}/sets"))
        .set_json(json!({
            "reps": 10,
            "weight": 50.0,
            "weight_left": 25.0,
            "weight_right": 27.5,
            "notes": "felt good",
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let req = test::TestRequest::put()
        .uri(&format!("/api/exercises/{exercise_id}/end"))
        .set_json(json!({
            "end_time": now + chrono::Duration::minutes(10),
            "notes": "done",
            "split_weight": true,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::get()
        .uri(&format!("/api/workouts/{workout_id}"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = json_body(resp).await;

    assert_eq!(body["workout"]["id"], workout_id);
    assert_eq!(body["workout"]["feedback"], "😊");
    assert_eq!(body["exercises"].as_array().unwrap().len(), 1);

    let exercise = &body["exercises"][0]["exercise"];
    assert_eq!(exercise["exercise_type"], "Bench Press");
    assert_eq!(exercise["per_side_weight"], true);
    assert_eq!(exercise["split_weight"], true);
    assert_eq!(exercise["settings"].as_array().unwrap().len(), 2);

    let sets = body["exercises"][0]["sets"].as_array().unwrap();
    assert_eq!(sets.len(), 1);
    assert_eq!(sets[0]["reps"], 10);
    assert_eq!(sets[0]["weight_left"], 25.0);
    assert_eq!(sets[0]["weight_right"], 27.5);
}

#[actix_web::test]
async fn replace_sets_overwrites_existing_sets() {
    let _env = TestEnv::new();
    let (_, app) = setup_test_app().await;
    let now = chrono::Utc::now();

    let req = test::TestRequest::post()
        .uri("/api/workouts")
        .set_json(json!({ "date": now, "start_time": now, "notes": null }))
        .to_request();
    let workout_id = json_body(test::call_service(&app, req).await).await["id"]
        .as_i64()
        .unwrap();

    let req = test::TestRequest::post()
        .uri(&format!("/api/workouts/{workout_id}/exercises"))
        .set_json(json!({
            "exercise_type": "Dumbbell Press",
            "start_time": now,
            "notes": null,
            "per_side_weight": true,
        }))
        .to_request();
    let exercise_id = json_body(test::call_service(&app, req).await).await["id"]
        .as_i64()
        .unwrap();

    let req = test::TestRequest::post()
        .uri(&format!("/api/exercises/{exercise_id}/sets"))
        .set_json(json!({ "reps": 8, "weight": 22.5, "notes": null }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let req = test::TestRequest::put()
        .uri(&format!("/api/exercises/{exercise_id}/sets"))
        .set_json(json!([
            { "reps": 10, "weight": 20.0, "weight_left": 20.0, "weight_right": 22.5, "notes": null },
            { "reps": 12, "weight": 17.5, "weight_left": 17.5, "weight_right": 17.5, "notes": "easy" }
        ]))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let replaced = json_body(resp).await;
    assert_eq!(replaced.as_array().unwrap().len(), 2);
    assert!(replaced[0]["id"].is_number());

    let req = test::TestRequest::get()
        .uri(&format!("/api/workouts/{workout_id}"))
        .to_request();
    let body = json_body(test::call_service(&app, req).await).await;
    let sets = body["exercises"][0]["sets"].as_array().unwrap();
    assert_eq!(sets.len(), 2);
    assert_eq!(sets[0]["reps"], 10);
    assert_eq!(sets[0]["weight_left"], 20.0);
    assert_eq!(sets[0]["weight_right"], 22.5);
    assert_eq!(sets[1]["notes"], "easy");
}

#[actix_web::test]
async fn cancel_endpoints_remove_data() {
    let _env = TestEnv::new();
    let (_, app) = setup_test_app().await;
    let now = chrono::Utc::now();

    let req = test::TestRequest::post()
        .uri("/api/workouts")
        .set_json(json!({ "date": now, "start_time": now, "notes": "to delete" }))
        .to_request();
    let workout_id = json_body(test::call_service(&app, req).await).await["id"]
        .as_i64()
        .unwrap();

    let req = test::TestRequest::post()
        .uri(&format!("/api/workouts/{workout_id}/exercises"))
        .set_json(json!({ "exercise_type": "Row", "start_time": now, "notes": null }))
        .to_request();
    let exercise_id = json_body(test::call_service(&app, req).await).await["id"]
        .as_i64()
        .unwrap();

    let req = test::TestRequest::delete()
        .uri(&format!("/api/exercises/{exercise_id}"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::get()
        .uri(&format!("/api/workouts/{workout_id}"))
        .to_request();
    let body = json_body(test::call_service(&app, req).await).await;
    assert!(body["exercises"].as_array().unwrap().is_empty());

    let req = test::TestRequest::delete()
        .uri(&format!("/api/workouts/{workout_id}"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::get().uri("/api/workouts").to_request();
    let workouts = json_body(test::call_service(&app, req).await).await;
    assert!(workouts.as_array().unwrap().is_empty());
}

#[actix_web::test]
async fn exercise_lookups_and_progress_endpoints_work() {
    let _env = TestEnv::new();
    let (_, app) = setup_test_app().await;
    let now = chrono::Utc::now();

    let req = test::TestRequest::post()
        .uri("/api/workouts")
        .set_json(json!({ "date": now, "start_time": now, "notes": null }))
        .to_request();
    let workout_id = json_body(test::call_service(&app, req).await).await["id"]
        .as_i64()
        .unwrap();

    let req = test::TestRequest::post()
        .uri(&format!("/api/workouts/{workout_id}/exercises"))
        .set_json(json!({
            "exercise_type": "Squat",
            "start_time": now,
            "notes": null
        }))
        .to_request();
    let exercise_id = json_body(test::call_service(&app, req).await).await["id"]
        .as_i64()
        .unwrap();

    let req = test::TestRequest::post()
        .uri(&format!("/api/exercises/{exercise_id}/sets"))
        .set_json(json!({ "reps": 5, "weight": 100.0, "notes": null }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let req = test::TestRequest::get()
        .uri("/api/exercises/types")
        .to_request();
    let body = json_body(test::call_service(&app, req).await).await;
    let types = body.as_array().unwrap();
    assert!(types.iter().any(|t| t == "Squat"));

    let req = test::TestRequest::get()
        .uri("/api/exercises/last/Squat")
        .to_request();
    let body = json_body(test::call_service(&app, req).await).await;
    assert!(body.is_array());
    assert_eq!(body.as_array().unwrap().len(), 2);

    let req = test::TestRequest::get()
        .uri("/api/progress/exercise/Squat")
        .to_request();
    let body = json_body(test::call_service(&app, req).await).await;
    assert!(body.is_array());
    assert!(!body.as_array().unwrap().is_empty());

    let req = test::TestRequest::get()
        .uri("/api/progress/workout-stats")
        .to_request();
    let body = json_body(test::call_service(&app, req).await).await;
    assert!(body.get("total_workouts").is_some());

    let req = test::TestRequest::get()
        .uri("/api/progress/volume?exercise_type=Squat")
        .to_request();
    let body = json_body(test::call_service(&app, req).await).await;
    assert!(body.get("weekly_volume").is_some());
    assert!(body.get("monthly_volume").is_some());
    assert!(body.get("personal_records").is_some());
}

#[actix_web::test]
async fn workout_times_can_be_edited_and_date_tracks_start_time() {
    let _env = TestEnv::new();
    let (_, app) = setup_test_app().await;

    let start = chrono::Utc::now()
        .with_nanosecond(0)
        .expect("truncate nanos");

    let req = test::TestRequest::post()
        .uri("/api/workouts")
        .set_json(json!({
            "date": start,
            "start_time": start,
            "notes": "times test",
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let workout_id = json_body(resp).await["id"].as_i64().expect("workout id");

    let new_start = start + chrono::Duration::hours(36);
    let new_end = new_start + chrono::Duration::minutes(55);

    let req = test::TestRequest::put()
        .uri(&format!("/api/workouts/{workout_id}/times"))
        .set_json(json!({
            "start_time": new_start,
            "end_time": new_end,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::get()
        .uri(&format!("/api/workouts/{workout_id}"))
        .to_request();
    let body = json_body(test::call_service(&app, req).await).await;

    let returned_start: chrono::DateTime<chrono::Utc> = body["workout"]["start_time"]
        .as_str()
        .expect("start_time string")
        .parse()
        .expect("parse start_time");
    let returned_end: chrono::DateTime<chrono::Utc> = body["workout"]["end_time"]
        .as_str()
        .expect("end_time string")
        .parse()
        .expect("parse end_time");
    let returned_date: chrono::DateTime<chrono::Utc> = body["workout"]["date"]
        .as_str()
        .expect("date string")
        .parse()
        .expect("parse date");

    assert_eq!(returned_start, new_start);
    assert_eq!(returned_end, new_end);
    assert_eq!(returned_date, new_start);

    let bad_req = test::TestRequest::put()
        .uri(&format!("/api/workouts/{workout_id}/times"))
        .set_json(json!({
            "start_time": new_end,
            "end_time": new_start,
        }))
        .to_request();
    let resp = test::call_service(&app, bad_req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn workouts_list_includes_exercise_count() {
    let _env = TestEnv::new();
    let (_, app) = setup_test_app().await;

    let now = chrono::Utc::now().with_nanosecond(0).expect("truncate nanos");

    let req = test::TestRequest::post()
        .uri("/api/workouts")
        .set_json(json!({ "date": now, "start_time": now, "notes": null }))
        .to_request();
    let workout_id = json_body(test::call_service(&app, req).await).await["id"]
        .as_i64()
        .unwrap();

    let req = test::TestRequest::post()
        .uri(&format!("/api/workouts/{workout_id}/exercises"))
        .set_json(json!({
            "exercise_type": "Bench Press",
            "start_time": now,
            "notes": null
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let req = test::TestRequest::get().uri("/api/workouts").to_request();
    let workouts = json_body(test::call_service(&app, req).await).await;
    let items = workouts.as_array().expect("workouts array");
    let selected = items
        .iter()
        .find(|w| w["id"].as_i64() == Some(workout_id))
        .expect("workout present in list");

    assert_eq!(selected["exercise_count"], 1);
}

#[actix_web::test]
async fn logs_endpoints_work_and_enforce_limits() {
    let _env = TestEnv::new();
    let (_, app) = setup_test_app().await;

    let req = test::TestRequest::post().uri("/api/logs/init").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    assert!(Path::new("logs").exists());

    let req = test::TestRequest::post()
        .uri("/api/logs")
        .set_json(json!([
            {"level": "info", "msg": "hello"},
            {"level": "warn", "msg": "world"}
        ]))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    assert!(Path::new("logs/client.log").exists());

    let too_many = (0..1001).map(|i| json!({ "idx": i })).collect::<Vec<_>>();
    let req = test::TestRequest::post()
        .uri("/api/logs")
        .set_json(too_many)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 413);
}

#[actix_web::test]
async fn backups_endpoints_create_list_restore_delete() {
    let _env = TestEnv::new();
    let (_, app) = setup_test_app().await;
    let now = chrono::Utc::now();

    let req = test::TestRequest::post()
        .uri("/api/workouts")
        .set_json(json!({ "date": now, "start_time": now, "notes": "before" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let req = test::TestRequest::post().uri("/api/backups").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let backup = json_body(resp).await;
    let filename = backup["filename"].as_str().expect("filename").to_string();

    let req = test::TestRequest::get().uri("/api/backups").to_request();
    let backups = json_body(test::call_service(&app, req).await).await;
    assert!(backups
        .as_array()
        .unwrap()
        .iter()
        .any(|b| b["filename"] == filename));

    let req = test::TestRequest::post()
        .uri("/api/workouts")
        .set_json(json!({ "date": now, "start_time": now, "notes": "after" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let req = test::TestRequest::get().uri("/api/workouts").to_request();
    let workouts = json_body(test::call_service(&app, req).await).await;
    assert_eq!(workouts.as_array().unwrap().len(), 2);

    let req = test::TestRequest::post()
        .uri(&format!("/api/backups/{filename}/restore"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::get().uri("/api/workouts").to_request();
    let workouts = json_body(test::call_service(&app, req).await).await;
    assert_eq!(workouts.as_array().unwrap().len(), 1);

    let req = test::TestRequest::delete()
        .uri(&format!("/api/backups/{filename}"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn not_found_is_mapped_and_unknown_exercise_returns_null() {
    let _env = TestEnv::new();
    let (_, app) = setup_test_app().await;

    let req = test::TestRequest::get()
        .uri("/api/workouts/999999")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
    let body = json_body(resp).await;
    assert!(body.get("error").is_some());

    let req = test::TestRequest::get()
        .uri("/api/exercises/last/Definitely%20Not%20A%20Real%20Exercise")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = json_body(resp).await;
    assert!(body.is_null());
}
