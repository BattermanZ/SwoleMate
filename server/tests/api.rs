use actix_web::{test, web, App, HttpResponse};
use once_cell::sync::Lazy;
use serde_json::{json, Value};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::Row;
use std::sync::Mutex;
use tempfile::TempDir;

use swolemate_server::{auth, db::Database, routes, schema};

static TEST_ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

const ADMIN_USERNAME: &str = "admin";
const ADMIN_PASSWORD: &str = "test-admin-password";
const ADMIN_PASSWORD_CHANGED: &str = "test-admin-password-changed";

struct EnvVarGuard {
    key: &'static str,
    prev: Option<String>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let prev = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self { key, prev }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(prev) = self.prev.take() {
            std::env::set_var(self.key, prev);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

struct TestEnv {
    _lock: std::sync::MutexGuard<'static, ()>,
    prev_dir: std::path::PathBuf,
    _temp_dir: TempDir,
    prev_database_url: Option<String>,
    prev_app_env: Option<String>,
    prev_bootstrap_admin_username: Option<String>,
    prev_bootstrap_admin_password: Option<String>,
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
        std::fs::create_dir_all(temp_dir.path().join("backups")).expect("backups dir");
        std::fs::File::create(temp_dir.path().join("database").join("swolemate.db"))
            .expect("create db file");

        std::env::set_current_dir(temp_dir.path()).expect("set_current_dir");

        let prev_database_url = std::env::var("DATABASE_URL").ok();
        std::env::set_var("DATABASE_URL", "sqlite:database/swolemate.db");

        let prev_app_env = std::env::var("APP_ENV").ok();
        std::env::set_var("APP_ENV", "development");

        let prev_bootstrap_admin_username = std::env::var("BOOTSTRAP_ADMIN_USERNAME").ok();
        let prev_bootstrap_admin_password = std::env::var("BOOTSTRAP_ADMIN_PASSWORD").ok();
        std::env::set_var("BOOTSTRAP_ADMIN_USERNAME", ADMIN_USERNAME);
        std::env::set_var("BOOTSTRAP_ADMIN_PASSWORD", ADMIN_PASSWORD);

        Self {
            _lock: lock,
            prev_dir,
            _temp_dir: temp_dir,
            prev_database_url,
            prev_app_env,
            prev_bootstrap_admin_username,
            prev_bootstrap_admin_password,
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

        if let Some(prev) = self.prev_app_env.take() {
            std::env::set_var("APP_ENV", prev);
        } else {
            std::env::remove_var("APP_ENV");
        }

        if let Some(prev) = self.prev_bootstrap_admin_username.take() {
            std::env::set_var("BOOTSTRAP_ADMIN_USERNAME", prev);
        } else {
            std::env::remove_var("BOOTSTRAP_ADMIN_USERNAME");
        }

        if let Some(prev) = self.prev_bootstrap_admin_password.take() {
            std::env::set_var("BOOTSTRAP_ADMIN_PASSWORD", prev);
        } else {
            std::env::remove_var("BOOTSTRAP_ADMIN_PASSWORD");
        }
    }
}

async fn setup_test_app_raw() -> (
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
    let session_cfg = auth::SessionConfig::for_env("development");
    let app = test::init_service(
        App::new()
            .wrap(swolemate_server::middleware::SessionAuth::new(
                database.clone(),
                session_cfg.clone(),
            ))
            .app_data(web::Data::new(database.clone()))
            .app_data(web::Data::new(session_cfg))
            .configure(routes::config),
    )
    .await;

    (database, app)
}

async fn setup_test_app_raw_with_cfg(
    session_cfg: auth::SessionConfig,
) -> (
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
            .wrap(swolemate_server::middleware::SessionAuth::new(
                database.clone(),
                session_cfg.clone(),
            ))
            .app_data(web::Data::new(database.clone()))
            .app_data(web::Data::new(session_cfg))
            .configure(routes::config),
    )
    .await;

    (database, app)
}

async fn setup_test_app_raw_with_cfg_and_concurrency(
    session_cfg: auth::SessionConfig,
) -> (
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
            .wrap(swolemate_server::middleware::SessionAuth::new(
                database.clone(),
                session_cfg.clone(),
            ))
            .wrap(swolemate_server::middleware::ApiConcurrency::from_env())
            .app_data(web::Data::new(database.clone()))
            .app_data(web::Data::new(session_cfg))
            .configure(routes::config)
            .service(test_slow_route),
    )
    .await;

    (database, app)
}

async fn setup_test_app() -> (
    Database,
    actix_web::cookie::Cookie<'static>,
    impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
) {
    let (database, app) = setup_test_app_raw().await;
    let admin_cookie = login_cookie(&app, ADMIN_USERNAME, ADMIN_PASSWORD).await;
    let admin_cookie =
        change_password_cookie(&app, &admin_cookie, ADMIN_PASSWORD, ADMIN_PASSWORD_CHANGED).await;
    (database, admin_cookie, app)
}

#[actix_web::get("/api/test/slow")]
async fn test_slow_route(_user: swolemate_server::middleware::CurrentUser) -> HttpResponse {
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    HttpResponse::Ok().json(json!({ "status": "ok" }))
}

async fn json_body(resp: actix_web::dev::ServiceResponse) -> Value {
    let bytes = test::read_body(resp).await;
    serde_json::from_slice(&bytes).expect("valid json response")
}

fn with_cookie(
    req: test::TestRequest,
    cookie: &actix_web::cookie::Cookie<'static>,
) -> test::TestRequest {
    req.cookie(cookie.clone())
}

async fn login_cookie(
    app: &impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
    username: &str,
    password: &str,
) -> actix_web::cookie::Cookie<'static> {
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({ "username": username, "password": password }))
        .to_request();
    let resp = test::call_service(app, req).await;
    assert!(resp.status().is_success());

    let set_cookie = resp
        .headers()
        .get(actix_web::http::header::SET_COOKIE)
        .expect("set-cookie header")
        .to_str()
        .expect("set-cookie str")
        .to_string();

    let cookie_pair = set_cookie
        .split(';')
        .next()
        .expect("cookie pair")
        .to_string();
    actix_web::cookie::Cookie::parse(cookie_pair)
        .expect("parse cookie")
        .into_owned()
}

async fn change_password_cookie(
    app: &impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
    cookie: &actix_web::cookie::Cookie<'static>,
    current_password: &str,
    new_password: &str,
) -> actix_web::cookie::Cookie<'static> {
    let req = with_cookie(test::TestRequest::post(), cookie)
        .uri("/api/auth/change-password")
        .set_json(json!({
            "current_password": current_password,
            "new_password": new_password
        }))
        .to_request();
    let resp = test::call_service(app, req).await;
    assert!(resp.status().is_success());

    let set_cookie = resp
        .headers()
        .get(actix_web::http::header::SET_COOKIE)
        .expect("set-cookie header")
        .to_str()
        .expect("set-cookie str")
        .to_string();

    let cookie_pair = set_cookie
        .split(';')
        .next()
        .expect("cookie pair")
        .to_string();
    actix_web::cookie::Cookie::parse(cookie_pair)
        .expect("parse cookie")
        .into_owned()
}

async fn login_cookie_active(
    app: &impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
    username: &str,
    password: &str,
) -> actix_web::cookie::Cookie<'static> {
    let cookie = login_cookie(app, username, password).await;
    let new_password = format!("{password}-changed");
    change_password_cookie(app, &cookie, password, &new_password).await
}

async fn create_user_as_admin(
    app: &impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
    admin_cookie: &actix_web::cookie::Cookie<'static>,
    username: &str,
    password: &str,
) -> i64 {
    let req = with_cookie(test::TestRequest::post(), admin_cookie)
        .uri("/api/admin/users")
        .set_json(json!({ "username": username, "password": password, "role": "user" }))
        .to_request();
    let resp = test::call_service(app, req).await;
    assert_eq!(resp.status(), 201);
    json_body(resp).await["id"].as_i64().expect("user id")
}

#[actix_web::test]
async fn workout_stats_empty_workouts_returns_empty_arrays() {
    let _env = TestEnv::new();
    let (_db, admin_cookie, app) = setup_test_app().await;

    let username = "user-zero";
    let password = "user-zero-password";
    create_user_as_admin(&app, &admin_cookie, username, password).await;
    let user_cookie = login_cookie_active(&app, username, password).await;

    let req = with_cookie(test::TestRequest::get(), &user_cookie)
        .uri("/api/progress/workout-stats")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body = json_body(resp).await;
    assert!(
        body["popular_hours"].as_array().is_some(),
        "popular_hours should be an array, got: {body}"
    );
    assert!(
        body["duration_distribution"].as_array().is_some(),
        "duration_distribution should be an array, got: {body}"
    );

    assert_eq!(body["popular_hours"].as_array().unwrap().len(), 0);
    assert_eq!(body["duration_distribution"].as_array().unwrap().len(), 0);
}

#[actix_web::test]
async fn health_check_works() {
    let _env = TestEnv::new();
    let (_db, _admin_cookie, app) = setup_test_app().await;

    let req = test::TestRequest::get().uri("/api/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body = json_body(resp).await;
    assert_eq!(body["status"], "healthy");
    assert!(body.get("timestamp").is_some());
}

#[actix_web::test]
async fn unauthenticated_requests_are_rejected() {
    let _env = TestEnv::new();
    let (_db, _admin_cookie, app) = setup_test_app().await;

    let req = test::TestRequest::get().uri("/api/workouts").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn login_me_logout_flow_works() {
    let _env = TestEnv::new();
    let (_db, app) = setup_test_app_raw().await;

    let cookie = login_cookie(&app, ADMIN_USERNAME, ADMIN_PASSWORD).await;

    // Fresh login should be forced into a password change.
    let req = with_cookie(test::TestRequest::get(), &cookie)
        .uri("/api/workouts")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);

    let req = with_cookie(test::TestRequest::get(), &cookie)
        .uri("/api/auth/me")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let me = json_body(resp).await;
    assert_eq!(me["username"], ADMIN_USERNAME);
    assert_eq!(me["role"], "admin");
    assert_eq!(me["must_change_password"], true);

    let cookie =
        change_password_cookie(&app, &cookie, ADMIN_PASSWORD, ADMIN_PASSWORD_CHANGED).await;
    let req = with_cookie(test::TestRequest::get(), &cookie)
        .uri("/api/workouts")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri("/api/auth/logout")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = with_cookie(test::TestRequest::get(), &cookie)
        .uri("/api/workouts")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn admin_only_endpoints_are_forbidden_for_normal_users() {
    let _env = TestEnv::new();
    let (_db, admin_cookie, app) = setup_test_app().await;

    let _user_id = create_user_as_admin(&app, &admin_cookie, "user1", "passwordpassword").await;
    let user_cookie = login_cookie_active(&app, "user1", "passwordpassword").await;

    let req = with_cookie(test::TestRequest::get(), &user_cookie)
        .uri("/api/admin/users")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);

    let req = with_cookie(test::TestRequest::get(), &user_cookie)
        .uri("/api/backups")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);
}

#[actix_web::test]
async fn admin_can_reset_user_password_and_revoke_sessions() {
    let _env = TestEnv::new();
    let (_db, admin_cookie, app) = setup_test_app().await;

    let user_id = create_user_as_admin(&app, &admin_cookie, "resetme", "passwordpassword").await;
    let user_cookie = login_cookie_active(&app, "resetme", "passwordpassword").await;

    let req = with_cookie(test::TestRequest::post(), &admin_cookie)
        .uri(&format!("/api/admin/users/{user_id}/reset-password"))
        .set_json(json!({ "new_password": "newpasswordpassword" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Old session is revoked.
    let req = with_cookie(test::TestRequest::get(), &user_cookie)
        .uri("/api/workouts")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    // Old password no longer works.
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({ "username": "resetme", "password": "passwordpassword" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    // New password works.
    let new_cookie = login_cookie(&app, "resetme", "newpasswordpassword").await;
    let req = with_cookie(test::TestRequest::get(), &new_cookie)
        .uri("/api/workouts")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);

    let changed_cookie =
        change_password_cookie(&app, &new_cookie, "newpasswordpassword", "newpasswordpassword2")
            .await;
    let req = with_cookie(test::TestRequest::get(), &changed_cookie)
        .uri("/api/workouts")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn admin_can_delete_user_and_all_user_data() {
    let _env = TestEnv::new();
    let (db, admin_cookie, app) = setup_test_app().await;

    let user_id = create_user_as_admin(&app, &admin_cookie, "deleteme", "passwordpassword").await;
    let user_cookie = login_cookie_active(&app, "deleteme", "passwordpassword").await;

    let now = chrono::Utc::now();
    let req = with_cookie(test::TestRequest::post(), &user_cookie)
        .uri("/api/workouts")
        .set_json(json!({ "date": now, "start_time": now, "notes": "to be deleted" }))
        .to_request();
    let workout_id = json_body(test::call_service(&app, req).await).await["id"]
        .as_i64()
        .unwrap();

    let req = with_cookie(test::TestRequest::delete(), &admin_cookie)
        .uri(&format!("/api/admin/users/{user_id}"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 204);

    // User can no longer authenticate.
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({ "username": "deleteme", "password": "passwordpassword" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    // Domain data is deleted.
    let pool = db.pool().await;
    let workout_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workouts WHERE id = ?")
        .bind(workout_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(workout_count, 0);
}

#[actix_web::test]
async fn cannot_delete_last_admin() {
    let _env = TestEnv::new();
    let (_db, admin_cookie, app) = setup_test_app().await;

    let req = with_cookie(test::TestRequest::get(), &admin_cookie)
        .uri("/api/auth/me")
        .to_request();
    let me = json_body(test::call_service(&app, req).await).await;
    let admin_id = me["id"].as_i64().unwrap();

    let req = with_cookie(test::TestRequest::delete(), &admin_cookie)
        .uri(&format!("/api/admin/users/{admin_id}"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 409);
}

#[actix_web::test]
async fn user_foreign_keys_cascade_on_delete() {
    let _env = TestEnv::new();
    let (db, _admin_cookie, _app) = setup_test_app().await;

    let pool = db.pool().await;
    for table in [
        "sessions",
        "workouts",
        "exercises",
        "sets",
        "exercise_settings",
    ] {
        let rows = sqlx::query(&format!("PRAGMA foreign_key_list('{table}')"))
            .fetch_all(&pool)
            .await
            .unwrap();

        let mut found = false;
        for row in rows {
            let target: String = row.try_get("table").unwrap();
            let from: String = row.try_get("from").unwrap();
            let on_delete: String = row.try_get("on_delete").unwrap();
            if target == "users" && from == "user_id" {
                found = true;
                assert_eq!(
                    on_delete, "CASCADE",
                    "{table}.user_id -> users.id should be ON DELETE CASCADE"
                );
            }
        }
        assert!(found, "{table} should have a user_id foreign key");
    }
}

#[actix_web::test]
async fn workout_and_exercise_flow_works_for_normal_user() {
    let _env = TestEnv::new();
    let (_db, admin_cookie, app) = setup_test_app().await;

    let _user_id = create_user_as_admin(&app, &admin_cookie, "user2", "passwordpassword").await;
    let cookie = login_cookie_active(&app, "user2", "passwordpassword").await;

    let now = chrono::Utc::now();

    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri("/api/workouts")
        .set_json(json!({
            "date": now,
            "start_time": now,
            "notes": "test workout",
            "timezone_offset_minutes": -60
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let workout_id = json_body(resp).await["id"].as_i64().expect("workout id");

    let req = with_cookie(test::TestRequest::post(), &cookie)
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
    let exercise_id = json_body(resp).await["id"].as_i64().expect("exercise id");

    let req = with_cookie(test::TestRequest::post(), &cookie)
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

    let req = with_cookie(test::TestRequest::put(), &cookie)
        .uri(&format!("/api/exercises/{exercise_id}/end"))
        .set_json(json!({
            "end_time": now + chrono::Duration::minutes(10),
            "notes": "done",
            "split_weight": true,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = with_cookie(test::TestRequest::get(), &cookie)
        .uri(&format!("/api/workouts/{workout_id}"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = json_body(resp).await;

    assert_eq!(body["workout"]["id"], workout_id);
    assert_eq!(body["exercises"].as_array().unwrap().len(), 1);
    let exercise = &body["exercises"][0]["exercise"];
    assert_eq!(exercise["exercise_type"], "Bench Press");
    assert_eq!(exercise["split_weight"], true);
    assert_eq!(exercise["settings"].as_array().unwrap().len(), 2);
}

#[actix_web::test]
async fn data_is_scoped_per_user() {
    let _env = TestEnv::new();
    let (_db, admin_cookie, app) = setup_test_app().await;

    let _ = create_user_as_admin(&app, &admin_cookie, "alice", "passwordpassword").await;
    let _ = create_user_as_admin(&app, &admin_cookie, "bob", "passwordpassword").await;

    let alice = login_cookie_active(&app, "alice", "passwordpassword").await;
    let bob = login_cookie_active(&app, "bob", "passwordpassword").await;

    let now = chrono::Utc::now();
    let req = with_cookie(test::TestRequest::post(), &alice)
        .uri("/api/workouts")
        .set_json(json!({ "date": now, "start_time": now, "notes": "alice workout" }))
        .to_request();
    let workout_id = json_body(test::call_service(&app, req).await).await["id"]
        .as_i64()
        .unwrap();

    let req = with_cookie(test::TestRequest::get(), &bob)
        .uri(&format!("/api/workouts/{workout_id}"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    let req = with_cookie(test::TestRequest::get(), &bob)
        .uri("/api/workouts")
        .to_request();
    let workouts = json_body(test::call_service(&app, req).await).await;
    assert!(workouts.as_array().unwrap().is_empty());
}

#[actix_web::test]
async fn update_workout_times_can_update_notes_and_feedback() {
    let _env = TestEnv::new();
    let (_db, admin_cookie, app) = setup_test_app().await;

    let _user_id = create_user_as_admin(&app, &admin_cookie, "u4", "passwordpassword").await;
    let cookie = login_cookie_active(&app, "u4", "passwordpassword").await;

    let now = chrono::Utc::now();
    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri("/api/workouts")
        .set_json(json!({ "date": now, "start_time": now, "notes": "initial" }))
        .to_request();
    let workout_id = json_body(test::call_service(&app, req).await).await["id"]
        .as_i64()
        .expect("workout id");

    let later = now + chrono::Duration::minutes(10);
    let req = with_cookie(test::TestRequest::put(), &cookie)
        .uri(&format!("/api/workouts/{workout_id}/times"))
        .set_json(json!({
            "start_time": now,
            "end_time": later,
            "notes": "updated notes",
            "feedback": "😐"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = with_cookie(test::TestRequest::get(), &cookie)
        .uri(&format!("/api/workouts/{workout_id}"))
        .to_request();
    let body = json_body(test::call_service(&app, req).await).await;
    assert_eq!(body["workout"]["notes"], "updated notes");
    assert_eq!(body["workout"]["feedback"], "😐");
}

#[actix_web::test]
async fn update_workout_times_preserves_notes_and_feedback_when_omitted() {
    let _env = TestEnv::new();
    let (_db, admin_cookie, app) = setup_test_app().await;

    let _user_id = create_user_as_admin(&app, &admin_cookie, "u5", "passwordpassword").await;
    let cookie = login_cookie_active(&app, "u5", "passwordpassword").await;

    let now = chrono::Utc::now();
    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri("/api/workouts")
        .set_json(json!({ "date": now, "start_time": now, "notes": "initial notes" }))
        .to_request();
    let workout_id = json_body(test::call_service(&app, req).await).await["id"]
        .as_i64()
        .expect("workout id");

    let end_time = now + chrono::Duration::minutes(1);
    let req = with_cookie(test::TestRequest::put(), &cookie)
        .uri(&format!("/api/workouts/{workout_id}/end"))
        .set_json(json!({ "end_time": end_time, "notes": "keep me", "feedback": "😊" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let later = now + chrono::Duration::minutes(10);
    let req = with_cookie(test::TestRequest::put(), &cookie)
        .uri(&format!("/api/workouts/{workout_id}/times"))
        .set_json(json!({
            "start_time": now,
            "end_time": later
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = with_cookie(test::TestRequest::get(), &cookie)
        .uri(&format!("/api/workouts/{workout_id}"))
        .to_request();
    let body = json_body(test::call_service(&app, req).await).await;
    assert_eq!(body["workout"]["notes"], "keep me");
    assert_eq!(body["workout"]["feedback"], "😊");
}

#[actix_web::test]
async fn workouts_are_auto_closed_after_inactivity_and_marked() {
    let _env = TestEnv::new();
    let (db, admin_cookie, app) = setup_test_app().await;

    let _user_id = create_user_as_admin(&app, &admin_cookie, "u6", "passwordpassword").await;
    let cookie = login_cookie_active(&app, "u6", "passwordpassword").await;

    let start = chrono::Utc::now() - chrono::Duration::hours(2);
    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri("/api/workouts")
        .set_json(json!({ "date": start, "start_time": start, "notes": "stale" }))
        .to_request();
    let workout_id = json_body(test::call_service(&app, req).await).await["id"]
        .as_i64()
        .expect("workout id");

    let closed = db.auto_close_stale_workouts(30).await.expect("auto close");
    assert!(closed >= 1);

    let req = with_cookie(test::TestRequest::get(), &cookie)
        .uri(&format!("/api/workouts/{workout_id}"))
        .to_request();
    let body = json_body(test::call_service(&app, req).await).await;

    let workout = &body["workout"];
    assert!(workout.get("auto_closed_at").is_some());
    assert!(workout["auto_closed_at"].is_string());

    let start_time = chrono::DateTime::parse_from_rfc3339(workout["start_time"].as_str().unwrap())
        .unwrap()
        .with_timezone(&chrono::Utc);
    let end_time = chrono::DateTime::parse_from_rfc3339(workout["end_time"].as_str().unwrap())
        .unwrap()
        .with_timezone(&chrono::Utc);
    let delta = end_time - start_time;
    let expected = chrono::Duration::minutes(30);
    assert!(
        (delta - expected).num_seconds().abs() <= 2,
        "expected ~30m inactivity close; start={start_time} end={end_time} delta={delta}"
    );

    let new_end = start_time + chrono::Duration::minutes(45);
    let req = with_cookie(test::TestRequest::put(), &cookie)
        .uri(&format!("/api/workouts/{workout_id}/times"))
        .set_json(json!({
            "start_time": start_time,
            "end_time": new_end
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = with_cookie(test::TestRequest::get(), &cookie)
        .uri(&format!("/api/workouts/{workout_id}"))
        .to_request();
    let body = json_body(test::call_service(&app, req).await).await;
    assert!(body["workout"]["auto_closed_at"].is_null());
}

#[actix_web::test]
async fn logs_endpoints_work_and_enforce_limits() {
    let _env = TestEnv::new();
    let (_db, admin_cookie, app) = setup_test_app().await;

    let _user_id = create_user_as_admin(&app, &admin_cookie, "user3", "passwordpassword").await;
    let cookie = login_cookie_active(&app, "user3", "passwordpassword").await;

    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri("/api/logs")
        .set_json(json!([
            {"level": "info", "msg": "hello"},
            {"level": "warn", "message": "world", "target": "ui"}
        ]))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let too_many = (0..1001).map(|i| json!({ "idx": i })).collect::<Vec<_>>();
    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri("/api/logs")
        .set_json(too_many)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 413);
}

#[actix_web::test]
async fn backups_endpoints_create_list_restore_delete_admin_only() {
    let _env = TestEnv::new();
    let (_db, admin_cookie, app) = setup_test_app().await;
    let now = chrono::Utc::now();

    let req = with_cookie(test::TestRequest::post(), &admin_cookie)
        .uri("/api/workouts")
        .set_json(json!({ "date": now, "start_time": now, "notes": "before" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let req = with_cookie(test::TestRequest::post(), &admin_cookie)
        .uri("/api/backups")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let backup = json_body(resp).await;
    let filename = backup["filename"].as_str().expect("filename").to_string();

    let req = with_cookie(test::TestRequest::get(), &admin_cookie)
        .uri("/api/backups")
        .to_request();
    let backups = json_body(test::call_service(&app, req).await).await;
    assert!(backups
        .as_array()
        .unwrap()
        .iter()
        .any(|b| b["filename"] == filename));

    let req = with_cookie(test::TestRequest::post(), &admin_cookie)
        .uri("/api/workouts")
        .set_json(json!({ "date": now, "start_time": now, "notes": "after" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let req = with_cookie(test::TestRequest::get(), &admin_cookie)
        .uri("/api/workouts")
        .to_request();
    let workouts = json_body(test::call_service(&app, req).await).await;
    assert_eq!(workouts.as_array().unwrap().len(), 2);

    let req = with_cookie(test::TestRequest::post(), &admin_cookie)
        .uri(&format!("/api/backups/{filename}/restore"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = with_cookie(test::TestRequest::get(), &admin_cookie)
        .uri("/api/workouts")
        .to_request();
    let workouts = json_body(test::call_service(&app, req).await).await;
    assert_eq!(workouts.as_array().unwrap().len(), 1);

    let req = with_cookie(test::TestRequest::delete(), &admin_cookie)
        .uri(&format!("/api/backups/{filename}"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn schema_backfill_amsterdam_timezone_offsets_dst_aware() {
    let _env = TestEnv::new();

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite:database/swolemate.db")
        .await
        .expect("connect sqlite");
    schema::setup_schema(&pool).await.expect("setup_schema");

    // Determine default user id (created by schema v5 bootstrap).
    let default_user_id: i64 = sqlx::query_scalar("SELECT id FROM users ORDER BY id ASC LIMIT 1")
        .fetch_one(&pool)
        .await
        .expect("default user id");

    let summer: chrono::DateTime<chrono::Utc> = "2025-07-01T08:00:00Z".parse().unwrap();
    let winter: chrono::DateTime<chrono::Utc> = "2025-01-15T08:00:00Z".parse().unwrap();

    for started_at in [summer, winter] {
        sqlx::query(
            r#"
            INSERT INTO workouts (user_id, date, start_time, end_time, notes, feedback, timezone_offset_minutes)
            VALUES (?, ?, ?, ?, NULL, NULL, NULL)
            "#,
        )
        .bind(default_user_id)
        .bind(started_at)
        .bind(started_at)
        .bind(started_at + chrono::Duration::minutes(1))
        .execute(&pool)
        .await
        .expect("insert workout");
    }

    sqlx::query("DELETE FROM schema_version WHERE version = 4")
        .execute(&pool)
        .await
        .expect("delete schema v4 marker");

    schema::setup_schema(&pool)
        .await
        .expect("setup_schema applies v4 backfill");

    let rows = sqlx::query(
        r#"
        SELECT start_time, timezone_offset_minutes
        FROM workouts
        WHERE start_time IN (?, ?)
        "#,
    )
    .bind(summer)
    .bind(winter)
    .fetch_all(&pool)
    .await
    .expect("select backfilled offsets");

    assert_eq!(rows.len(), 2);
    for row in rows {
        let start_time: chrono::DateTime<chrono::Utc> = row.try_get("start_time").unwrap();
        let offset: i64 = row.try_get("timezone_offset_minutes").unwrap();
        if start_time == summer {
            assert_eq!(offset, -120);
        } else if start_time == winter {
            assert_eq!(offset, -60);
        } else {
            panic!("unexpected start_time: {start_time}");
        }
    }
}

#[actix_web::test]
async fn validation_rejects_invalid_set_payloads() {
    let _env = TestEnv::new();
    let (_db, admin_cookie, app) = setup_test_app().await;

    let _user_id = create_user_as_admin(&app, &admin_cookie, "user4", "passwordpassword").await;
    let cookie = login_cookie_active(&app, "user4", "passwordpassword").await;

    let now = chrono::Utc::now();
    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri("/api/workouts")
        .set_json(json!({
            "date": now,
            "start_time": now,
            "notes": null,
        }))
        .to_request();
    let workout_id = json_body(test::call_service(&app, req).await).await["id"]
        .as_i64()
        .unwrap();

    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri(&format!("/api/workouts/{workout_id}/exercises"))
        .set_json(json!({
            "exercise_type": "Bench Press",
            "start_time": now,
            "notes": null,
        }))
        .to_request();
    let exercise_id = json_body(test::call_service(&app, req).await).await["id"]
        .as_i64()
        .unwrap();

    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri(&format!("/api/exercises/{exercise_id}/sets"))
        .set_json(json!({
            "reps": 10,
            "weight": -1.0,
            "notes": null
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri(&format!("/api/exercises/{exercise_id}/sets"))
        .set_json(json!({
            "reps": 10,
            "weight": 20.0,
            "weight_left": 10.0,
            "notes": null
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn invalid_backup_filenames_are_rejected_before_io() {
    let _env = TestEnv::new();
    let (_db, admin_cookie, app) = setup_test_app().await;

    let req = with_cookie(test::TestRequest::post(), &admin_cookie)
        .uri("/api/backups/..tar.gz/restore")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let req = with_cookie(test::TestRequest::delete(), &admin_cookie)
        .uri("/api/backups/not-a-backup.zip")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn login_is_rate_limited_after_repeated_failed_attempts() {
    let _env = TestEnv::new();
    let (_db, admin_cookie, app) = setup_test_app().await;
    create_user_as_admin(&app, &admin_cookie, "lockme", "lockme-password").await;

    for _ in 0..5 {
        let req = test::TestRequest::post()
            .uri("/api/auth/login")
            .set_json(json!({ "username": "lockme", "password": "wrong-password" }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
    }

    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({ "username": "lockme", "password": "wrong-password" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 429);
}

#[actix_web::test]
async fn admin_disable_user_revokes_existing_session() {
    let _env = TestEnv::new();
    let (_db, admin_cookie, app) = setup_test_app().await;

    let user_id = create_user_as_admin(&app, &admin_cookie, "disableme", "passwordpassword").await;
    let user_cookie = login_cookie_active(&app, "disableme", "passwordpassword").await;

    let req = with_cookie(test::TestRequest::post(), &admin_cookie)
        .uri(&format!("/api/admin/users/{user_id}/disable"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = with_cookie(test::TestRequest::get(), &user_cookie)
        .uri("/api/workouts")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn session_is_rotated_when_near_expiry() {
    let _env = TestEnv::new();
    let mut session_cfg = auth::SessionConfig::for_env("development");
    session_cfg.session_ttl_days = 90;
    session_cfg.rotate_if_expires_within_days = 30;
    let (db, app) = setup_test_app_raw_with_cfg(session_cfg).await;

    let cookie = login_cookie(&app, ADMIN_USERNAME, ADMIN_PASSWORD).await;
    let cookie = change_password_cookie(&app, &cookie, ADMIN_PASSWORD, ADMIN_PASSWORD_CHANGED).await;
    let old_token = cookie.value().to_string();
    let old_hash = auth::hash_session_token(&old_token);

    let pool = db.pool().await;
    sqlx::query(
        r#"
        UPDATE sessions
        SET expires_at = datetime('now', '+1 day')
        WHERE session_hash = ?
        "#,
    )
    .bind(&old_hash)
    .execute(&pool)
    .await
    .expect("set session expiry");

    let req = with_cookie(test::TestRequest::get(), &cookie)
        .uri("/api/workouts")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let set_cookie = resp
        .headers()
        .get(actix_web::http::header::SET_COOKIE)
        .expect("rotation set-cookie")
        .to_str()
        .expect("set-cookie str")
        .to_string();
    let new_cookie = actix_web::cookie::Cookie::parse(
        set_cookie
            .split(';')
            .next()
            .expect("cookie pair")
            .to_string(),
    )
    .expect("parse cookie")
    .into_owned();
    assert_ne!(new_cookie.value(), old_token);

    let req = with_cookie(test::TestRequest::get(), &cookie)
        .uri("/api/workouts")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    let req = with_cookie(test::TestRequest::get(), &new_cookie)
        .uri("/api/workouts")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn api_concurrency_returns_503_when_queue_times_out() {
    let _env = TestEnv::new();
    let _max_inflight = EnvVarGuard::set("API_MAX_INFLIGHT", "1");
    let _timeout_ms = EnvVarGuard::set("API_CONCURRENCY_TIMEOUT_MS", "25");

    let (_db, app) =
        setup_test_app_raw_with_cfg_and_concurrency(auth::SessionConfig::for_env("development"))
            .await;
    let cookie = login_cookie(&app, ADMIN_USERNAME, ADMIN_PASSWORD).await;
    let cookie = change_password_cookie(&app, &cookie, ADMIN_PASSWORD, ADMIN_PASSWORD_CHANGED).await;

    let slow_req = with_cookie(test::TestRequest::get(), &cookie)
        .uri("/api/test/slow")
        .to_request();
    let busy_req = with_cookie(test::TestRequest::get(), &cookie)
        .uri("/api/workouts")
        .to_request();

    let slow_call = test::call_service(&app, slow_req);
    let busy_call = async {
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        test::call_service(&app, busy_req).await
    };

    let (slow_resp, busy_resp) = tokio::join!(slow_call, busy_call);
    assert!(slow_resp.status().is_success());
    assert_eq!(busy_resp.status(), 503);
}

#[actix_web::test]
async fn replace_sets_endpoint_replaces_existing_sets_and_validates_payload() {
    let _env = TestEnv::new();
    let (_db, admin_cookie, app) = setup_test_app().await;

    create_user_as_admin(&app, &admin_cookie, "sets-user", "passwordpassword").await;
    let cookie = login_cookie_active(&app, "sets-user", "passwordpassword").await;
    let now = chrono::Utc::now();

    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri("/api/workouts")
        .set_json(json!({ "date": now, "start_time": now, "notes": "sets workout" }))
        .to_request();
    let workout_id = json_body(test::call_service(&app, req).await).await["id"]
        .as_i64()
        .expect("workout id");

    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri(&format!("/api/workouts/{workout_id}/exercises"))
        .set_json(json!({ "exercise_type": "Squat", "start_time": now }))
        .to_request();
    let exercise_id = json_body(test::call_service(&app, req).await).await["id"]
        .as_i64()
        .expect("exercise id");

    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri(&format!("/api/exercises/{exercise_id}/sets"))
        .set_json(json!({ "reps": 5, "weight": 80.0 }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let req = with_cookie(test::TestRequest::put(), &cookie)
        .uri(&format!("/api/exercises/{exercise_id}/sets"))
        .set_json(json!([
            { "reps": 3, "weight": 90.0, "notes": "heavy" },
            { "reps": 8, "weight": 70.0 }
        ]))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let replaced = json_body(resp).await;
    assert_eq!(replaced.as_array().unwrap().len(), 2);

    let req = with_cookie(test::TestRequest::get(), &cookie)
        .uri(&format!("/api/workouts/{workout_id}"))
        .to_request();
    let workout = json_body(test::call_service(&app, req).await).await;
    let sets = workout["exercises"][0]["sets"].as_array().unwrap();
    assert_eq!(sets.len(), 2);
    assert_eq!(sets[0]["reps"], 3);
    assert_eq!(sets[0]["weight"], 90.0);

    let req = with_cookie(test::TestRequest::put(), &cookie)
        .uri(&format!("/api/exercises/{exercise_id}/sets"))
        .set_json(json!([
            { "reps": 10, "weight": 20.0, "weight_left": 10.0 }
        ]))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}
