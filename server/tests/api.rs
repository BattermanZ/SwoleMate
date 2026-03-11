use actix_web::{test, web, App, HttpResponse};
use base64::Engine;
use once_cell::sync::Lazy;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::Row;
use std::io::Write;
use std::sync::Mutex;
use tempfile::TempDir;

use swolemate_server::{auth, db::Database, mcp, oauth, routes, schema};

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
    let database = setup_test_database().await;
    let session_cfg = auth::SessionConfig::for_env("development");
    let oauth_cfg = oauth::OAuthConfig::from_env();
    let app = test::init_service(
        App::new()
            .wrap(swolemate_server::middleware::SessionAuth::new(
                database.clone(),
                session_cfg.clone(),
            ))
            .app_data(web::Data::new(database.clone()))
            .app_data(web::Data::new(session_cfg))
            .app_data(web::Data::new(oauth_cfg.clone()))
            .configure(routes::config)
            .configure(oauth::routes::config)
            .service(
                web::scope("")
                    .wrap(swolemate_server::middleware::McpBearerAuth::new(
                        database.clone(),
                        oauth_cfg.protected_resource_endpoint.clone(),
                    ))
                    .configure(mcp::routes::config),
            ),
    )
    .await;

    (database, app)
}

async fn setup_test_database() -> Database {
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

    Database::new(pool)
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
    let database = setup_test_database().await;
    let oauth_cfg = oauth::OAuthConfig::from_env();
    let app = test::init_service(
        App::new()
            .wrap(swolemate_server::middleware::SessionAuth::new(
                database.clone(),
                session_cfg.clone(),
            ))
            .app_data(web::Data::new(database.clone()))
            .app_data(web::Data::new(session_cfg))
            .app_data(web::Data::new(oauth_cfg.clone()))
            .configure(routes::config)
            .configure(oauth::routes::config)
            .service(
                web::scope("")
                    .wrap(swolemate_server::middleware::McpBearerAuth::new(
                        database.clone(),
                        oauth_cfg.protected_resource_endpoint.clone(),
                    ))
                    .configure(mcp::routes::config),
            ),
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
    let database = setup_test_database().await;
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
            .service(test_slow_route)
            .service(test_logs_slow)
            .service(test_backups_slow)
            .service(test_restore_slow),
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
    tokio::time::sleep(std::time::Duration::from_millis(350)).await;
    HttpResponse::Ok().json(json!({ "status": "ok" }))
}

#[actix_web::post("/api/logs-slow")]
async fn test_logs_slow(_user: swolemate_server::middleware::CurrentUser) -> HttpResponse {
    tokio::time::sleep(std::time::Duration::from_millis(350)).await;
    HttpResponse::Ok().json(json!({ "status": "ok" }))
}

#[actix_web::post("/api/backups-slow")]
async fn test_backups_slow(_user: swolemate_server::middleware::CurrentUser) -> HttpResponse {
    tokio::time::sleep(std::time::Duration::from_millis(350)).await;
    HttpResponse::Ok().json(json!({ "status": "ok" }))
}

#[actix_web::post("/api/backups-slow/restore")]
async fn test_restore_slow(_user: swolemate_server::middleware::CurrentUser) -> HttpResponse {
    tokio::time::sleep(std::time::Duration::from_millis(350)).await;
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

fn with_same_origin(req: test::TestRequest) -> test::TestRequest {
    req.insert_header((actix_web::http::header::HOST, "app.local"))
        .insert_header((actix_web::http::header::ORIGIN, "http://app.local"))
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

fn pkce_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest)
}

async fn register_oauth_client(
    app: &impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
    scope: &str,
) -> Value {
    let req = test::TestRequest::post()
        .uri("/oauth/register")
        .set_json(json!({
            "client_name": "Test MCP Client",
            "redirect_uris": ["https://client.example/callback"],
            "scope": scope
        }))
        .to_request();
    let resp = test::call_service(app, req).await;
    assert_eq!(resp.status(), 201);
    json_body(resp).await
}

async fn authorize_oauth_code(
    app: &impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
    client_id: &str,
    username: &str,
    password: &str,
    scope: &str,
    verifier: &str,
) -> String {
    let redirect_uri = "https://client.example/callback";
    let challenge = pkce_challenge(verifier);

    let req = test::TestRequest::get()
        .uri(&format!(
            "/oauth/authorize?response_type=code&client_id={}&redirect_uri={}&scope={}&state=test-state&code_challenge={}&code_challenge_method=S256",
            urlencoding::encode(client_id),
            urlencoding::encode(redirect_uri),
            urlencoding::encode(scope),
            urlencoding::encode(&challenge),
        ))
        .to_request();
    let resp = test::call_service(app, req).await;
    assert_eq!(resp.status(), 200);

    let req = test::TestRequest::post()
        .uri("/oauth/authorize")
        .set_form(&[
            ("response_type", "code"),
            ("client_id", client_id),
            ("redirect_uri", redirect_uri),
            ("scope", scope),
            ("state", "test-state"),
            ("code_challenge", challenge.as_str()),
            ("code_challenge_method", "S256"),
            ("username", username),
            ("password", password),
            ("approve", "yes"),
        ])
        .to_request();
    let resp = test::call_service(app, req).await;
    assert_eq!(resp.status(), 302);
    let location = resp
        .headers()
        .get(actix_web::http::header::LOCATION)
        .expect("location header")
        .to_str()
        .expect("location str");
    location
        .split('?')
        .nth(1)
        .expect("query string")
        .split('&')
        .find_map(|pair| pair.split_once('='))
        .filter(|(key, _)| *key == "code")
        .map(|(_, value)| {
            urlencoding::decode(value)
                .expect("decode code")
                .into_owned()
        })
        .expect("authorization code")
}

async fn exchange_token(
    app: &impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
    client_id: &str,
    code: &str,
    verifier: &str,
) -> Value {
    let req = test::TestRequest::post()
        .uri("/oauth/token")
        .insert_header((
            actix_web::http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        ))
        .set_form(&[
            ("grant_type", "authorization_code"),
            ("client_id", client_id),
            ("code", code),
            ("redirect_uri", "https://client.example/callback"),
            ("code_verifier", verifier),
        ])
        .to_request();
    let resp = test::call_service(app, req).await;
    assert_eq!(resp.status(), 200);
    json_body(resp).await
}

async fn mcp_call(
    app: &impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
    access_token: &str,
    id: i64,
    name: &str,
    arguments: Value,
) -> Value {
    let req = test::TestRequest::post()
        .uri("/mcp")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {access_token}"),
        ))
        .set_json(json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": arguments
            }
        }))
        .to_request();
    let resp = test::call_service(app, req).await;
    assert_eq!(resp.status(), 200);
    json_body(resp).await
}

async fn latest_mcp_audit_entries(
    db: &Database,
    limit: i64,
) -> Vec<(String, bool, Option<String>)> {
    let pool = db.pool().await;
    let rows = sqlx::query(
        r#"
        SELECT tool_name, success, error_code
        FROM mcp_audit_log
        ORDER BY id DESC
        LIMIT ?
        "#,
    )
    .bind(limit)
    .fetch_all(&pool)
    .await
    .expect("fetch mcp audit log");

    rows.into_iter()
        .map(|row| {
            (
                row.get::<String, _>("tool_name"),
                row.get::<i64, _>("success") != 0,
                row.get::<Option<String>, _>("error_code"),
            )
        })
        .collect()
}

async fn latest_mcp_audit_payload(db: &Database) -> Value {
    let pool = db.pool().await;
    let row = sqlx::query(
        r#"
        SELECT input_summary_json
        FROM mcp_audit_log
        ORDER BY id DESC
        LIMIT 1
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("fetch mcp audit payload");
    let payload = row
        .get::<Option<String>, _>("input_summary_json")
        .unwrap_or_default();
    serde_json::from_str(&payload).expect("parse mcp audit payload")
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

    let changed_cookie = change_password_cookie(
        &app,
        &new_cookie,
        "newpasswordpassword",
        "newpasswordpassword2",
    )
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
async fn schema_creates_oauth_and_mcp_foundation_tables() {
    let _env = TestEnv::new();

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite:database/swolemate.db")
        .await
        .expect("connect sqlite");
    schema::setup_schema(&pool).await.expect("setup_schema");

    let version_exists: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM schema_version WHERE version = 9")
            .fetch_one(&pool)
            .await
            .expect("schema version 9 marker");
    assert_eq!(version_exists, 1);

    for table_name in [
        "oauth_clients",
        "oauth_authorization_codes",
        "oauth_access_tokens",
        "oauth_refresh_tokens",
        "oauth_consents",
        "mcp_audit_log",
    ] {
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name = ?",
        )
        .bind(table_name)
        .fetch_one(&pool)
        .await
        .expect("table existence");
        assert_eq!(exists, 1, "missing table {table_name}");
    }

    let version_exists: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM schema_version WHERE version = 10")
            .fetch_one(&pool)
            .await
            .expect("schema version 10 marker");
    assert_eq!(version_exists, 1);

    for table_name in [
        "oauth_authorization_codes",
        "oauth_access_tokens",
        "oauth_refresh_tokens",
        "oauth_consents",
    ] {
        let fks = sqlx::query(&format!("PRAGMA foreign_key_list('{table_name}')"))
            .fetch_all(&pool)
            .await
            .expect("foreign key list");

        let has_client_fk = fks.iter().any(|row| {
            let foreign_table: String = row.try_get("table").expect("fk table");
            let from_column: String = row.try_get("from").expect("fk from");
            foreign_table == "oauth_clients" && from_column == "client_id"
        });

        assert!(
            has_client_fk,
            "missing oauth_clients(client_id) foreign key on {table_name}"
        );
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
async fn login_is_rate_limited_by_ip_across_usernames() {
    let _env = TestEnv::new();
    let _attempts_guard = EnvVarGuard::set("LOGIN_RATE_LIMIT_ATTEMPTS", "3");
    let _window_guard = EnvVarGuard::set("LOGIN_RATE_LIMIT_WINDOW_SECONDS", "600");
    let (_db, app) = setup_test_app_raw().await;

    for i in 0..3 {
        let req = test::TestRequest::post()
            .uri("/api/auth/login")
            .insert_header((actix_web::http::header::X_FORWARDED_FOR, "203.0.113.50"))
            .set_json(json!({ "username": format!("nouser-{i}"), "password": "wrong-password" }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
    }

    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .insert_header((actix_web::http::header::X_FORWARDED_FOR, "203.0.113.50"))
        .set_json(json!({ "username": "another-user", "password": "wrong-password" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 429);
}

#[actix_web::test]
async fn csrf_blocks_mutating_authenticated_requests_without_origin_in_production_mode() {
    let _env = TestEnv::new();
    let session_cfg = auth::SessionConfig::for_env("production");
    let (_db, app) = setup_test_app_raw_with_cfg(session_cfg).await;

    let cookie = login_cookie(&app, ADMIN_USERNAME, ADMIN_PASSWORD).await;

    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri("/api/auth/change-password")
        .set_json(json!({
            "current_password": ADMIN_PASSWORD,
            "new_password": ADMIN_PASSWORD_CHANGED
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);

    let req = with_same_origin(with_cookie(test::TestRequest::post(), &cookie))
        .uri("/api/auth/change-password")
        .set_json(json!({
            "current_password": ADMIN_PASSWORD,
            "new_password": ADMIN_PASSWORD_CHANGED
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
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
    let cookie =
        change_password_cookie(&app, &cookie, ADMIN_PASSWORD, ADMIN_PASSWORD_CHANGED).await;
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
    let _timeout_ms = EnvVarGuard::set("API_CONCURRENCY_TIMEOUT_MS", "15");

    let (_db, app) =
        setup_test_app_raw_with_cfg_and_concurrency(auth::SessionConfig::for_env("development"))
            .await;
    let cookie = login_cookie(&app, ADMIN_USERNAME, ADMIN_PASSWORD).await;
    let cookie =
        change_password_cookie(&app, &cookie, ADMIN_PASSWORD, ADMIN_PASSWORD_CHANGED).await;

    let slow_call = async {
        let req = with_cookie(test::TestRequest::get(), &cookie)
            .uri("/api/test/slow")
            .to_request();
        test::call_service(&app, req).await
    };
    let busy_call = async {
        let mut statuses = Vec::new();
        for delay in [10u64, 40, 80, 120] {
            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            let req = with_cookie(test::TestRequest::get(), &cookie)
                .uri("/api/workouts")
                .to_request();
            statuses.push(test::call_service(&app, req).await.status().as_u16());
        }
        statuses
    };

    let (slow_resp, busy_statuses) = tokio::join!(slow_call, busy_call);
    assert!(slow_resp.status().is_success());
    assert!(
        busy_statuses.iter().any(|s| *s == 503),
        "expected at least one 503, got statuses: {:?}",
        busy_statuses
    );
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

#[actix_web::test]
async fn exercise_and_progress_endpoints_are_covered() {
    let _env = TestEnv::new();
    let (_db, admin_cookie, app) = setup_test_app().await;

    create_user_as_admin(&app, &admin_cookie, "phase2-user", "passwordpassword").await;
    let cookie = login_cookie_active(&app, "phase2-user", "passwordpassword").await;
    let now = chrono::Utc::now();

    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri("/api/workouts")
        .set_json(json!({ "date": now, "start_time": now, "notes": "phase2" }))
        .to_request();
    let workout_id = json_body(test::call_service(&app, req).await).await["id"]
        .as_i64()
        .expect("workout id");

    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri(&format!("/api/workouts/{workout_id}/exercises"))
        .set_json(json!({
            "exercise_type": "Incline Bench",
            "start_time": now,
            "notes": "first"
        }))
        .to_request();
    let exercise_id = json_body(test::call_service(&app, req).await).await["id"]
        .as_i64()
        .expect("exercise id");

    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri(&format!("/api/exercises/{exercise_id}/sets"))
        .set_json(json!({ "reps": 6, "weight": 70.0 }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let req = with_cookie(test::TestRequest::get(), &cookie)
        .uri("/api/exercises/types")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let types = json_body(resp).await;
    assert!(types
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t.as_str() == Some("Incline Bench")));

    let req = with_cookie(test::TestRequest::get(), &cookie)
        .uri("/api/exercises/last/Incline%20Bench")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let last = json_body(resp).await;
    assert_eq!(last["exercise"]["id"], exercise_id);
    assert_eq!(last["sets"].as_array().unwrap().len(), 1);

    let req = with_cookie(test::TestRequest::get(), &cookie)
        .uri("/api/progress/exercise/Incline%20Bench")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let progress = json_body(resp).await;
    assert!(!progress.as_array().unwrap().is_empty());

    let req = with_cookie(test::TestRequest::get(), &cookie)
        .uri("/api/progress/volume?exercise_type=Incline%20Bench")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let volume = json_body(resp).await;
    assert!(volume["weekly_volume"].is_array());
    assert!(volume["monthly_volume"].is_array());
    assert!(volume["personal_records"].is_object());

    let req = with_cookie(test::TestRequest::delete(), &cookie)
        .uri(&format!("/api/exercises/{exercise_id}"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = with_cookie(test::TestRequest::get(), &cookie)
        .uri(&format!("/api/workouts/{workout_id}"))
        .to_request();
    let workout = json_body(test::call_service(&app, req).await).await;
    assert_eq!(workout["exercises"].as_array().unwrap().len(), 0);

    let req = with_cookie(test::TestRequest::delete(), &cookie)
        .uri(&format!("/api/workouts/{workout_id}"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = with_cookie(test::TestRequest::get(), &cookie)
        .uri(&format!("/api/workouts/{workout_id}"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn validation_boundaries_reject_out_of_range_and_oversized_payloads() {
    let _env = TestEnv::new();
    let (_db, admin_cookie, app) = setup_test_app().await;

    create_user_as_admin(&app, &admin_cookie, "bounds-user", "passwordpassword").await;
    let cookie = login_cookie_active(&app, "bounds-user", "passwordpassword").await;
    let now = chrono::Utc::now();

    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri("/api/workouts")
        .set_json(json!({
            "date": now,
            "start_time": now,
            "timezone_offset_minutes": 900
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri("/api/workouts")
        .set_json(json!({
            "date": now,
            "start_time": now,
            "timezone_offset_minutes": -841
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri("/api/workouts")
        .set_json(json!({
            "date": now,
            "start_time": now,
            "notes": "N".repeat(2001)
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri("/api/workouts")
        .set_json(json!({ "date": now, "start_time": now, "notes": "ok" }))
        .to_request();
    let workout_id = json_body(test::call_service(&app, req).await).await["id"]
        .as_i64()
        .expect("workout id");

    let req = with_cookie(test::TestRequest::put(), &cookie)
        .uri(&format!("/api/workouts/{workout_id}/end"))
        .set_json(json!({
            "end_time": now + chrono::Duration::minutes(1),
            "feedback": "amazing"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let req = with_cookie(test::TestRequest::put(), &cookie)
        .uri(&format!("/api/workouts/{workout_id}/times"))
        .set_json(json!({
            "start_time": now,
            "end_time": now + chrono::Duration::minutes(2),
            "feedback": "excellent"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let long_type = "X".repeat(81);
    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri(&format!("/api/workouts/{workout_id}/exercises"))
        .set_json(json!({
            "exercise_type": long_type,
            "start_time": now
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let long_key = "K".repeat(65);
    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri(&format!("/api/workouts/{workout_id}/exercises"))
        .set_json(json!({
            "exercise_type": "Rows",
            "start_time": now,
            "settings": [{ "key": long_key, "value": "1" }]
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let long_value = "V".repeat(129);
    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri(&format!("/api/workouts/{workout_id}/exercises"))
        .set_json(json!({
            "exercise_type": "Rows",
            "start_time": now,
            "settings": [{ "key": "pin", "value": long_value }]
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri(&format!("/api/workouts/{workout_id}/exercises"))
        .set_json(json!({
            "exercise_type": "Rows",
            "start_time": now,
            "notes": "N".repeat(2001)
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri(&format!("/api/workouts/{workout_id}/exercises"))
        .set_json(json!({
            "exercise_type": "Rows",
            "start_time": now
        }))
        .to_request();
    let exercise_id = json_body(test::call_service(&app, req).await).await["id"]
        .as_i64()
        .expect("exercise id");

    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri(&format!("/api/exercises/{exercise_id}/sets"))
        .set_json(json!({ "reps": 501, "weight": 80.0 }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri(&format!("/api/exercises/{exercise_id}/sets"))
        .set_json(json!({ "reps": 8, "weight": 2001.0 }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn backup_failure_paths_are_handled_without_crashing() {
    let _env = TestEnv::new();
    let (_db, admin_cookie, app) = setup_test_app().await;

    let malformed_name = "swolemate_2099-01-02_00-00_manual.tar.gz";
    let malformed_path = std::env::current_dir()
        .expect("cwd")
        .join("backups")
        .join(malformed_name);
    let mut f = std::fs::File::create(&malformed_path).expect("create malformed backup");
    f.write_all(b"not-a-valid-tar-gz")
        .expect("write malformed backup");

    let req = with_cookie(test::TestRequest::get(), &admin_cookie)
        .uri("/api/backups")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let backups = json_body(resp).await;
    assert!(backups.is_array());

    let req = with_cookie(test::TestRequest::delete(), &admin_cookie)
        .uri("/api/backups/swolemate_2099-01-03_00-00_manual.tar.gz")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn malformed_backup_restore_fails_safely() {
    let _env = TestEnv::new();
    let (_db, admin_cookie, app) = setup_test_app().await;

    let malformed_name = "swolemate_2099-01-02_00-00_manual.tar.gz";
    let malformed_path = std::env::current_dir()
        .expect("cwd")
        .join("backups")
        .join(malformed_name);
    let mut f = std::fs::File::create(&malformed_path).expect("create malformed backup");
    f.write_all(b"not-a-valid-tar-gz")
        .expect("write malformed backup");

    let req = with_cookie(test::TestRequest::post(), &admin_cookie)
        .uri(&format!("/api/backups/{malformed_name}/restore"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 500);
}

#[actix_web::test]
async fn missing_backup_restore_fails_safely() {
    let _env = TestEnv::new();
    let (_db, admin_cookie, app) = setup_test_app().await;

    let missing_name = "swolemate_2099-01-01_00-00_manual.tar.gz";
    let req = with_cookie(test::TestRequest::post(), &admin_cookie)
        .uri(&format!("/api/backups/{missing_name}/restore"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 500);
}

#[actix_web::test]
async fn auth_negative_paths_are_rejected() {
    let _env = TestEnv::new();
    let (_db, admin_cookie, app) = setup_test_app().await;

    create_user_as_admin(&app, &admin_cookie, "auth-neg", "passwordpassword").await;
    let cookie = login_cookie_active(&app, "auth-neg", "passwordpassword").await;

    let req = with_cookie(test::TestRequest::post(), &cookie)
        .uri("/api/auth/change-password")
        .set_json(json!({
            "current_password": "wrong-password",
            "new_password": "passwordpassword-changed"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    let req = test::TestRequest::post()
        .uri("/api/auth/logout")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn admin_list_users_returns_expected_shape_and_members() {
    let _env = TestEnv::new();
    let (_db, admin_cookie, app) = setup_test_app().await;

    let _ = create_user_as_admin(&app, &admin_cookie, "list-user-a", "passwordpassword").await;
    let _ = create_user_as_admin(&app, &admin_cookie, "list-user-b", "passwordpassword").await;

    let req = with_cookie(test::TestRequest::get(), &admin_cookie)
        .uri("/api/admin/users")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let users = json_body(resp).await;
    let arr = users.as_array().expect("users array");
    assert!(arr
        .iter()
        .any(|u| u["username"] == "admin" && u["role"] == "admin"));
    assert!(arr
        .iter()
        .any(|u| u["username"] == "list-user-a" && u["role"] == "user"));
    assert!(arr
        .iter()
        .any(|u| u["username"] == "list-user-b" && u["role"] == "user"));
    assert!(arr.iter().all(|u| {
        u.get("id").is_some()
            && u.get("username").is_some()
            && u.get("role").is_some()
            && u.get("disabled_at").is_some()
    }));
}

#[actix_web::test]
async fn negative_contract_paths_return_expected_errors() {
    let _env = TestEnv::new();
    let (_db, admin_cookie, app) = setup_test_app().await;

    create_user_as_admin(&app, &admin_cookie, "neg-user", "passwordpassword").await;
    let cookie = login_cookie_active(&app, "neg-user", "passwordpassword").await;

    let req = with_cookie(test::TestRequest::get(), &cookie)
        .uri("/api/exercises/last/NoSuchExercise")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = json_body(resp).await;
    assert!(body.is_null());

    let req = with_cookie(test::TestRequest::delete(), &cookie)
        .uri("/api/exercises/999999")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    let req = with_cookie(test::TestRequest::delete(), &cookie)
        .uri("/api/workouts/999999")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    let req = with_cookie(test::TestRequest::get(), &cookie)
        .uri("/api/progress/volume")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn api_concurrency_enforces_logs_backups_and_restore_class_limits() {
    let _env = TestEnv::new();
    let _global = EnvVarGuard::set("API_MAX_INFLIGHT", "8");
    let _logs = EnvVarGuard::set("API_MAX_INFLIGHT_LOGS", "1");
    let _backups = EnvVarGuard::set("API_MAX_INFLIGHT_BACKUPS", "1");
    let _restore = EnvVarGuard::set("API_MAX_INFLIGHT_BACKUP_RESTORE", "1");
    let _timeout = EnvVarGuard::set("API_CONCURRENCY_TIMEOUT_MS", "15");

    let (_db, app) =
        setup_test_app_raw_with_cfg_and_concurrency(auth::SessionConfig::for_env("development"))
            .await;
    let cookie = login_cookie(&app, ADMIN_USERNAME, ADMIN_PASSWORD).await;
    let cookie =
        change_password_cookie(&app, &cookie, ADMIN_PASSWORD, ADMIN_PASSWORD_CHANGED).await;

    for (slow_uri, same_class_uri) in [
        ("/api/logs-slow", "/api/logs-slow"),
        ("/api/backups-slow", "/api/backups-slow"),
        ("/api/backups-slow/restore", "/api/backups-slow/restore"),
    ] {
        let hold = async {
            let req = with_cookie(test::TestRequest::post(), &cookie)
                .uri(slow_uri)
                .to_request();
            test::call_service(&app, req).await
        };

        let contended = async {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            let req = with_cookie(test::TestRequest::post(), &cookie)
                .uri(same_class_uri)
                .to_request();
            test::call_service(&app, req).await
        };

        let unaffected_global = async {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            let req = with_cookie(test::TestRequest::get(), &cookie)
                .uri("/api/workouts")
                .to_request();
            test::call_service(&app, req).await
        };

        let (hold_resp, contended_resp, global_resp) =
            tokio::join!(hold, contended, unaffected_global);
        assert!(hold_resp.status().is_success());
        assert_eq!(contended_resp.status(), 503, "uri={same_class_uri}");
        assert!(global_resp.status().is_success(), "uri={same_class_uri}");
    }
}

#[actix_web::test]
async fn oauth_metadata_and_client_registration_work() {
    let _env = TestEnv::new();
    let _registration_guard = EnvVarGuard::set("OAUTH_ALLOW_DYNAMIC_CLIENT_REGISTRATION", "true");
    let (_db, _admin_cookie, app) = setup_test_app().await;

    let req = test::TestRequest::get()
        .uri("/.well-known/oauth-authorization-server")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let metadata = json_body(resp).await;
    assert!(metadata["authorization_endpoint"]
        .as_str()
        .unwrap()
        .ends_with("/oauth/authorize"));
    assert!(metadata["token_endpoint"]
        .as_str()
        .unwrap()
        .ends_with("/oauth/token"));

    let req = test::TestRequest::get()
        .uri("/.well-known/oauth-protected-resource")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let resource = json_body(resp).await;
    assert!(resource["resource"].as_str().unwrap().ends_with("/mcp"));

    let registered = register_oauth_client(&app, "workouts.read progress.read").await;
    assert_eq!(registered["token_endpoint_auth_method"], "none");
    assert_eq!(registered["response_types"][0], "code");
}

#[actix_web::test]
async fn oauth_registration_rejects_non_loopback_http_redirect_uris() {
    let _env = TestEnv::new();
    let _registration_guard = EnvVarGuard::set("OAUTH_ALLOW_DYNAMIC_CLIENT_REGISTRATION", "true");
    let (_db, _admin_cookie, app) = setup_test_app().await;

    let req = test::TestRequest::post()
        .uri("/oauth/register")
        .set_json(json!({
            "client_name": "Bad Client",
            "redirect_uris": ["http://evil.example/callback"],
            "scope": "workouts.read"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
    let body = json_body(resp).await;
    assert_eq!(body["error"], "invalid_redirect_uri");
}

#[actix_web::test]
async fn oauth_authorize_page_displays_redirect_host() {
    let _env = TestEnv::new();
    let _registration_guard = EnvVarGuard::set("OAUTH_ALLOW_DYNAMIC_CLIENT_REGISTRATION", "true");
    let (_db, _admin_cookie, app) = setup_test_app().await;

    let registered = register_oauth_client(&app, "workouts.read").await;
    let client_id = registered["client_id"].as_str().unwrap();
    let challenge = pkce_challenge("display-host-verifier");

    let req = test::TestRequest::get()
        .uri(&format!(
            "/oauth/authorize?response_type=code&client_id={}&redirect_uri={}&scope={}&state=test-state&code_challenge={}&code_challenge_method=S256",
            urlencoding::encode(client_id),
            urlencoding::encode("https://client.example/callback"),
            urlencoding::encode("workouts.read"),
            urlencoding::encode(&challenge),
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body = String::from_utf8(test::read_body(resp).await.to_vec()).expect("html body");
    assert!(body.contains("Redirect host:</strong> client.example"));
}

#[actix_web::test]
async fn oauth_token_flow_and_read_only_mcp_tools_work() {
    let _env = TestEnv::new();
    let _registration_guard = EnvVarGuard::set("OAUTH_ALLOW_DYNAMIC_CLIENT_REGISTRATION", "true");
    let (_db, admin_cookie, app) = setup_test_app().await;

    create_user_as_admin(&app, &admin_cookie, "mcp-user", "passwordpassword").await;
    let user_cookie = login_cookie_active(&app, "mcp-user", "passwordpassword").await;
    let user_password = "passwordpassword-changed";

    let req = with_cookie(test::TestRequest::post(), &user_cookie)
        .uri("/api/workouts")
        .set_json(json!({
            "date": chrono::Utc::now(),
            "start_time": chrono::Utc::now(),
            "notes": "mcp session"
        }))
        .to_request();
    let workout_resp = test::call_service(&app, req).await;
    assert_eq!(workout_resp.status(), 201);
    let workout_id = json_body(workout_resp).await["id"].as_i64().unwrap();

    let registered = register_oauth_client(&app, "workouts.read progress.read").await;
    let client_id = registered["client_id"].as_str().unwrap();
    let verifier = "batch2-test-verifier";
    let code = authorize_oauth_code(
        &app,
        client_id,
        "mcp-user",
        user_password,
        "workouts.read progress.read",
        verifier,
    )
    .await;
    let tokens = exchange_token(&app, client_id, &code, verifier).await;
    let access_token = tokens["access_token"].as_str().unwrap();

    let req = test::TestRequest::post()
        .uri("/mcp")
        .set_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
    assert!(resp
        .headers()
        .contains_key(actix_web::http::header::WWW_AUTHENTICATE));

    let req = test::TestRequest::post()
        .uri("/mcp")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {access_token}"),
        ))
        .set_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let initialize = json_body(resp).await;
    assert_eq!(initialize["result"]["serverInfo"]["name"], "swolemate");

    let req = test::TestRequest::post()
        .uri("/mcp")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {access_token}"),
        ))
        .set_json(json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let listed = json_body(resp).await;
    assert!(listed["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .any(|tool| tool["name"] == "list_workouts"));

    let req = test::TestRequest::post()
        .uri("/mcp")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {access_token}"),
        ))
        .set_json(json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "get_workout",
                "arguments": { "id": workout_id }
            }
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let called = json_body(resp).await;
    assert_eq!(
        called["result"]["structuredContent"]["workout"]["id"],
        workout_id
    );

    let registered = register_oauth_client(&app, "workouts.read").await;
    let limited_client_id = registered["client_id"].as_str().unwrap();
    let limited_code = authorize_oauth_code(
        &app,
        limited_client_id,
        "mcp-user",
        user_password,
        "workouts.read",
        "limited-verifier",
    )
    .await;
    let limited_tokens =
        exchange_token(&app, limited_client_id, &limited_code, "limited-verifier").await;
    let limited_access_token = limited_tokens["access_token"].as_str().unwrap();

    let req = test::TestRequest::post()
        .uri("/mcp")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {limited_access_token}"),
        ))
        .set_json(json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "get_workout_stats",
                "arguments": {}
            }
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body = json_body(resp).await;
    assert_eq!(body["error"]["message"], "Forbidden");
}

#[actix_web::test]
async fn oauth_registration_is_disabled_by_default() {
    let _env = TestEnv::new();
    let (_db, _admin_cookie, app) = setup_test_app().await;

    let req = test::TestRequest::post()
        .uri("/oauth/register")
        .set_json(json!({
            "client_name": "Blocked Client",
            "redirect_uris": ["https://client.example/callback"],
            "scope": "workouts.read"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn oauth_authorize_is_rate_limited_by_ip() {
    let _env = TestEnv::new();
    let _registration_guard = EnvVarGuard::set("OAUTH_ALLOW_DYNAMIC_CLIENT_REGISTRATION", "true");
    let _attempts_guard = EnvVarGuard::set("LOGIN_RATE_LIMIT_ATTEMPTS", "2");
    let _window_guard = EnvVarGuard::set("LOGIN_RATE_LIMIT_WINDOW_SECONDS", "600");
    let (_db, admin_cookie, app) = setup_test_app().await;

    create_user_as_admin(&app, &admin_cookie, "oauth-lock", "passwordpassword").await;
    let _cookie = login_cookie_active(&app, "oauth-lock", "passwordpassword").await;
    let registered = register_oauth_client(&app, "workouts.read").await;
    let client_id = registered["client_id"].as_str().unwrap();
    let redirect_uri = "https://client.example/callback";
    let challenge = pkce_challenge("oauth-rate-limit-verifier");

    for _ in 0..2 {
        let req = test::TestRequest::post()
            .uri("/oauth/authorize")
            .insert_header((actix_web::http::header::X_FORWARDED_FOR, "203.0.113.60"))
            .set_form(&[
                ("response_type", "code"),
                ("client_id", client_id),
                ("redirect_uri", redirect_uri),
                ("scope", "workouts.read"),
                ("state", "oauth-rate-limit"),
                ("code_challenge", challenge.as_str()),
                ("code_challenge_method", "S256"),
                ("username", "oauth-lock"),
                ("password", "wrong-password"),
                ("approve", "yes"),
            ])
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
    }

    let req = test::TestRequest::post()
        .uri("/oauth/authorize")
        .insert_header((actix_web::http::header::X_FORWARDED_FOR, "203.0.113.60"))
        .set_form(&[
            ("response_type", "code"),
            ("client_id", client_id),
            ("redirect_uri", redirect_uri),
            ("scope", "workouts.read"),
            ("state", "oauth-rate-limit"),
            ("code_challenge", challenge.as_str()),
            ("code_challenge_method", "S256"),
            ("username", "oauth-lock"),
            ("password", "wrong-password"),
            ("approve", "yes"),
        ])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 429);
}

#[actix_web::test]
async fn invalid_oauth_token_exchange_does_not_burn_authorization_code() {
    let _env = TestEnv::new();
    let _registration_guard = EnvVarGuard::set("OAUTH_ALLOW_DYNAMIC_CLIENT_REGISTRATION", "true");
    let (_db, admin_cookie, app) = setup_test_app().await;

    create_user_as_admin(&app, &admin_cookie, "oauth-code", "passwordpassword").await;
    let _cookie = login_cookie_active(&app, "oauth-code", "passwordpassword").await;
    let registered = register_oauth_client(&app, "workouts.read").await;
    let client_id = registered["client_id"].as_str().unwrap();
    let verifier = "oauth-good-verifier";
    let code = authorize_oauth_code(
        &app,
        client_id,
        "oauth-code",
        "passwordpassword-changed",
        "workouts.read",
        verifier,
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/oauth/token")
        .insert_header((
            actix_web::http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        ))
        .set_form(&[
            ("grant_type", "authorization_code"),
            ("client_id", client_id),
            ("code", code.as_str()),
            ("redirect_uri", "https://client.example/callback"),
            ("code_verifier", "wrong-verifier"),
        ])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let tokens = exchange_token(&app, client_id, &code, verifier).await;
    assert!(tokens["access_token"].as_str().is_some());
}

#[actix_web::test]
async fn refresh_tokens_rotate_and_old_refresh_token_is_rejected() {
    let _env = TestEnv::new();
    let _registration_guard = EnvVarGuard::set("OAUTH_ALLOW_DYNAMIC_CLIENT_REGISTRATION", "true");
    let (_db, admin_cookie, app) = setup_test_app().await;

    create_user_as_admin(&app, &admin_cookie, "oauth-refresh", "passwordpassword").await;
    let _cookie = login_cookie_active(&app, "oauth-refresh", "passwordpassword").await;
    let registered = register_oauth_client(&app, "workouts.read").await;
    let client_id = registered["client_id"].as_str().unwrap();
    let verifier = "refresh-rotate-verifier";
    let code = authorize_oauth_code(
        &app,
        client_id,
        "oauth-refresh",
        "passwordpassword-changed",
        "workouts.read",
        verifier,
    )
    .await;
    let tokens = exchange_token(&app, client_id, &code, verifier).await;
    let refresh_token = tokens["refresh_token"].as_str().unwrap();

    let req = test::TestRequest::post()
        .uri("/oauth/token")
        .insert_header((
            actix_web::http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        ))
        .set_form(&[
            ("grant_type", "refresh_token"),
            ("client_id", client_id),
            ("refresh_token", refresh_token),
        ])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let refreshed = json_body(resp).await;
    let next_refresh = refreshed["refresh_token"].as_str().unwrap();
    assert_ne!(next_refresh, refresh_token);

    let req = test::TestRequest::post()
        .uri("/oauth/token")
        .insert_header((
            actix_web::http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        ))
        .set_form(&[
            ("grant_type", "refresh_token"),
            ("client_id", client_id),
            ("refresh_token", refresh_token),
        ])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
    let body = json_body(resp).await;
    assert_eq!(body["error"], "invalid_grant");
}

#[actix_web::test]
async fn oauth_revoke_revokes_access_token_for_mcp() {
    let _env = TestEnv::new();
    let _registration_guard = EnvVarGuard::set("OAUTH_ALLOW_DYNAMIC_CLIENT_REGISTRATION", "true");
    let (_db, admin_cookie, app) = setup_test_app().await;

    create_user_as_admin(&app, &admin_cookie, "oauth-revoke", "passwordpassword").await;
    let _cookie = login_cookie_active(&app, "oauth-revoke", "passwordpassword").await;
    let registered = register_oauth_client(&app, "workouts.read").await;
    let client_id = registered["client_id"].as_str().unwrap();
    let verifier = "revoke-verifier";
    let code = authorize_oauth_code(
        &app,
        client_id,
        "oauth-revoke",
        "passwordpassword-changed",
        "workouts.read",
        verifier,
    )
    .await;
    let tokens = exchange_token(&app, client_id, &code, verifier).await;
    let access_token = tokens["access_token"].as_str().unwrap();

    let req = test::TestRequest::post()
        .uri("/oauth/revoke")
        .insert_header((
            actix_web::http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        ))
        .set_form(&[
            ("token", access_token),
            ("client_id", client_id),
            ("token_type_hint", "access_token"),
        ])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let req = test::TestRequest::post()
        .uri("/mcp")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {access_token}"),
        ))
        .set_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn disabled_oauth_client_cannot_refresh_or_access_mcp() {
    let _env = TestEnv::new();
    let _registration_guard = EnvVarGuard::set("OAUTH_ALLOW_DYNAMIC_CLIENT_REGISTRATION", "true");
    let (db, admin_cookie, app) = setup_test_app().await;

    create_user_as_admin(&app, &admin_cookie, "oauth-disabled", "passwordpassword").await;
    let _cookie = login_cookie_active(&app, "oauth-disabled", "passwordpassword").await;
    let registered = register_oauth_client(&app, "workouts.read").await;
    let client_id = registered["client_id"].as_str().unwrap().to_string();
    let verifier = "disabled-client-verifier";
    let code = authorize_oauth_code(
        &app,
        &client_id,
        "oauth-disabled",
        "passwordpassword-changed",
        "workouts.read",
        verifier,
    )
    .await;
    let tokens = exchange_token(&app, &client_id, &code, verifier).await;
    let access_token = tokens["access_token"].as_str().unwrap().to_string();
    let refresh_token = tokens["refresh_token"].as_str().unwrap().to_string();

    let pool = db.pool().await;
    sqlx::query("UPDATE oauth_clients SET disabled_at = CURRENT_TIMESTAMP WHERE client_id = ?")
        .bind(&client_id)
        .execute(&pool)
        .await
        .expect("disable oauth client");

    let req = test::TestRequest::post()
        .uri("/oauth/token")
        .insert_header((
            actix_web::http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        ))
        .set_form(&[
            ("grant_type", "refresh_token"),
            ("client_id", client_id.as_str()),
            ("refresh_token", refresh_token.as_str()),
        ])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
    let body = json_body(resp).await;
    assert_eq!(body["error"], "invalid_grant");

    let req = test::TestRequest::post()
        .uri("/mcp")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {access_token}"),
        ))
        .set_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn oauth_token_flow_and_write_mcp_tools_work() {
    let _env = TestEnv::new();
    let _registration_guard = EnvVarGuard::set("OAUTH_ALLOW_DYNAMIC_CLIENT_REGISTRATION", "true");
    let (_db, admin_cookie, app) = setup_test_app().await;

    create_user_as_admin(&app, &admin_cookie, "mcp-write", "passwordpassword").await;
    let _user_cookie = login_cookie_active(&app, "mcp-write", "passwordpassword").await;

    let registered =
        register_oauth_client(&app, "workouts.read progress.read workouts.write").await;
    let client_id = registered["client_id"].as_str().unwrap();
    let verifier = "batch3-write-verifier";
    let code = authorize_oauth_code(
        &app,
        client_id,
        "mcp-write",
        "passwordpassword-changed",
        "workouts.read progress.read workouts.write",
        verifier,
    )
    .await;
    let tokens = exchange_token(&app, client_id, &code, verifier).await;
    let access_token = tokens["access_token"].as_str().unwrap();

    let now = chrono::Utc::now();
    let created = mcp_call(
        &app,
        access_token,
        1,
        "create_workout",
        json!({
            "date": now,
            "start_time": now,
            "notes": "mcp write batch"
        }),
    )
    .await;
    let workout_id = created["result"]["structuredContent"]["id"]
        .as_i64()
        .unwrap();

    let add_exercise = mcp_call(
        &app,
        access_token,
        2,
        "add_exercise",
        json!({
            "workout_id": workout_id,
            "exercise_type": "bench press",
            "start_time": now,
            "notes": "heavy day",
            "settings": [{"key": "bench", "value": "flat"}]
        }),
    )
    .await;
    let exercise_id = add_exercise["result"]["structuredContent"]["id"]
        .as_i64()
        .unwrap();

    let replaced = mcp_call(
        &app,
        access_token,
        3,
        "replace_sets",
        json!({
            "exercise_id": exercise_id,
            "sets": [
                { "reps": 5, "weight": 100.0 },
                { "reps": 5, "weight": 102.5, "notes": "top set" }
            ]
        }),
    )
    .await;
    assert_eq!(
        replaced["result"]["structuredContent"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let ended_exercise = mcp_call(
        &app,
        access_token,
        4,
        "end_exercise",
        json!({
            "id": exercise_id,
            "end_time": now + chrono::Duration::minutes(30),
            "notes": "done"
        }),
    )
    .await;
    assert_eq!(
        ended_exercise["result"]["structuredContent"]["message"],
        "Exercise ended successfully"
    );

    let ended_workout = mcp_call(
        &app,
        access_token,
        5,
        "end_workout",
        json!({
            "id": workout_id,
            "end_time": now + chrono::Duration::minutes(45),
            "notes": "wrapped",
            "feedback": "😊"
        }),
    )
    .await;
    assert_eq!(
        ended_workout["result"]["structuredContent"]["message"],
        "Workout ended successfully"
    );

    let fetched = mcp_call(
        &app,
        access_token,
        6,
        "get_workout",
        json!({ "id": workout_id }),
    )
    .await;
    assert_eq!(
        fetched["result"]["structuredContent"]["exercises"][0]["sets"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[actix_web::test]
async fn read_only_mcp_token_cannot_write() {
    let _env = TestEnv::new();
    let _registration_guard = EnvVarGuard::set("OAUTH_ALLOW_DYNAMIC_CLIENT_REGISTRATION", "true");
    let (_db, admin_cookie, app) = setup_test_app().await;

    create_user_as_admin(&app, &admin_cookie, "mcp-readonly", "passwordpassword").await;
    let _user_cookie = login_cookie_active(&app, "mcp-readonly", "passwordpassword").await;

    let registered = register_oauth_client(&app, "workouts.read progress.read").await;
    let client_id = registered["client_id"].as_str().unwrap();
    let verifier = "batch3-readonly-verifier";
    let code = authorize_oauth_code(
        &app,
        client_id,
        "mcp-readonly",
        "passwordpassword-changed",
        "workouts.read progress.read",
        verifier,
    )
    .await;
    let tokens = exchange_token(&app, client_id, &code, verifier).await;
    let access_token = tokens["access_token"].as_str().unwrap();

    let body = mcp_call(
        &app,
        access_token,
        1,
        "create_workout",
        json!({
            "date": chrono::Utc::now(),
            "start_time": chrono::Utc::now()
        }),
    )
    .await;
    assert_eq!(body["error"]["message"], "Forbidden");
}

#[actix_web::test]
async fn write_mcp_token_cannot_mutate_another_users_workout() {
    let _env = TestEnv::new();
    let _registration_guard = EnvVarGuard::set("OAUTH_ALLOW_DYNAMIC_CLIENT_REGISTRATION", "true");
    let (_db, admin_cookie, app) = setup_test_app().await;

    create_user_as_admin(&app, &admin_cookie, "mcp-owner-a", "passwordpassword").await;
    create_user_as_admin(&app, &admin_cookie, "mcp-owner-b", "passwordpassword").await;
    let user_a_cookie = login_cookie_active(&app, "mcp-owner-a", "passwordpassword").await;
    let user_b_cookie = login_cookie_active(&app, "mcp-owner-b", "passwordpassword").await;

    let req = with_cookie(test::TestRequest::post(), &user_b_cookie)
        .uri("/api/workouts")
        .set_json(json!({
            "date": chrono::Utc::now(),
            "start_time": chrono::Utc::now(),
            "notes": "belongs to b"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let other_workout_id = json_body(resp).await["id"].as_i64().unwrap();

    let registered = register_oauth_client(&app, "workouts.read workouts.write").await;
    let client_id = registered["client_id"].as_str().unwrap();
    let verifier = "batch3-owner-verifier";
    let code = authorize_oauth_code(
        &app,
        client_id,
        "mcp-owner-a",
        "passwordpassword-changed",
        "workouts.read workouts.write",
        verifier,
    )
    .await;
    let tokens = exchange_token(&app, client_id, &code, verifier).await;
    let access_token = tokens["access_token"].as_str().unwrap();

    let own_workout_resp = with_cookie(test::TestRequest::post(), &user_a_cookie)
        .uri("/api/workouts")
        .set_json(json!({
            "date": chrono::Utc::now(),
            "start_time": chrono::Utc::now(),
            "notes": "belongs to a"
        }))
        .to_request();
    let own_workout_resp = test::call_service(&app, own_workout_resp).await;
    assert_eq!(own_workout_resp.status(), 201);

    let body = mcp_call(
        &app,
        access_token,
        1,
        "add_exercise",
        json!({
            "workout_id": other_workout_id,
            "exercise_type": "squat",
            "start_time": chrono::Utc::now()
        }),
    )
    .await;
    assert_eq!(body["error"]["message"], "Not found");
}

#[actix_web::test]
async fn write_mcp_tools_are_audited_for_success_and_failure() {
    let _env = TestEnv::new();
    let _registration_guard = EnvVarGuard::set("OAUTH_ALLOW_DYNAMIC_CLIENT_REGISTRATION", "true");
    let (db, admin_cookie, app) = setup_test_app().await;

    create_user_as_admin(&app, &admin_cookie, "mcp-audit", "passwordpassword").await;
    let _user_cookie = login_cookie_active(&app, "mcp-audit", "passwordpassword").await;

    let registered = register_oauth_client(&app, "workouts.read workouts.write").await;
    let client_id = registered["client_id"].as_str().unwrap();
    let verifier = "batch3-audit-verifier";
    let code = authorize_oauth_code(
        &app,
        client_id,
        "mcp-audit",
        "passwordpassword-changed",
        "workouts.read workouts.write",
        verifier,
    )
    .await;
    let tokens = exchange_token(&app, client_id, &code, verifier).await;
    let access_token = tokens["access_token"].as_str().unwrap();

    let _created = mcp_call(
        &app,
        access_token,
        1,
        "create_workout",
        json!({
            "date": chrono::Utc::now(),
            "start_time": chrono::Utc::now()
        }),
    )
    .await;
    let failed = mcp_call(
        &app,
        access_token,
        2,
        "end_workout",
        json!({
            "id": 999999,
            "end_time": chrono::Utc::now()
        }),
    )
    .await;
    assert_eq!(failed["error"]["message"], "Not found");

    let entries = latest_mcp_audit_entries(&db, 2).await;
    assert_eq!(entries[0].0, "end_workout");
    assert!(!entries[0].1);
    assert_eq!(entries[0].2.as_deref(), Some("not_found"));
    assert_eq!(entries[1].0, "create_workout");
    assert!(entries[1].1);
}

#[actix_web::test]
async fn mcp_audit_log_redacts_notes_and_feedback() {
    let _env = TestEnv::new();
    let _registration_guard = EnvVarGuard::set("OAUTH_ALLOW_DYNAMIC_CLIENT_REGISTRATION", "true");
    let (db, admin_cookie, app) = setup_test_app().await;

    create_user_as_admin(&app, &admin_cookie, "mcp-redact", "passwordpassword").await;
    let _user_cookie = login_cookie_active(&app, "mcp-redact", "passwordpassword").await;

    let registered = register_oauth_client(&app, "workouts.write").await;
    let client_id = registered["client_id"].as_str().unwrap();
    let verifier = "redact-verifier";
    let code = authorize_oauth_code(
        &app,
        client_id,
        "mcp-redact",
        "passwordpassword-changed",
        "workouts.write",
        verifier,
    )
    .await;
    let tokens = exchange_token(&app, client_id, &code, verifier).await;
    let access_token = tokens["access_token"].as_str().unwrap();

    let _ = mcp_call(
        &app,
        access_token,
        1,
        "create_workout",
        json!({
            "date": chrono::Utc::now(),
            "start_time": chrono::Utc::now(),
            "notes": "secret note",
            "feedback": "should not appear"
        }),
    )
    .await;

    let payload = latest_mcp_audit_payload(&db).await;
    assert_eq!(payload["notes"], "<redacted>");
    assert_eq!(payload["feedback"], "<redacted>");
}

#[actix_web::test]
async fn invalid_write_mcp_payload_returns_json_rpc_error() {
    let _env = TestEnv::new();
    let _registration_guard = EnvVarGuard::set("OAUTH_ALLOW_DYNAMIC_CLIENT_REGISTRATION", "true");
    let (_db, admin_cookie, app) = setup_test_app().await;

    create_user_as_admin(&app, &admin_cookie, "mcp-invalid", "passwordpassword").await;
    let _user_cookie = login_cookie_active(&app, "mcp-invalid", "passwordpassword").await;

    let registered = register_oauth_client(&app, "workouts.write").await;
    let client_id = registered["client_id"].as_str().unwrap();
    let verifier = "batch3-invalid-verifier";
    let code = authorize_oauth_code(
        &app,
        client_id,
        "mcp-invalid",
        "passwordpassword-changed",
        "workouts.write",
        verifier,
    )
    .await;
    let tokens = exchange_token(&app, client_id, &code, verifier).await;
    let access_token = tokens["access_token"].as_str().unwrap();

    let body = mcp_call(
        &app,
        access_token,
        1,
        "replace_sets",
        json!({
            "exercise_id": 123,
            "sets": [
                { "reps": 5, "weight": 20.0, "weight_left": 10.0 }
            ]
        }),
    )
    .await;
    assert_eq!(body["error"]["code"], -32602);
    assert_eq!(body["error"]["message"], "Invalid params");
}

#[actix_web::test]
async fn mcp_write_rate_limit_returns_json_rpc_error() {
    let _env = TestEnv::new();
    let _registration_guard = EnvVarGuard::set("OAUTH_ALLOW_DYNAMIC_CLIENT_REGISTRATION", "true");
    let _rate_guard = EnvVarGuard::set("MCP_RATE_LIMIT_PER_MINUTE", "2");
    let (_db, admin_cookie, app) = setup_test_app().await;

    create_user_as_admin(&app, &admin_cookie, "mcp-throttle", "passwordpassword").await;
    let _user_cookie = login_cookie_active(&app, "mcp-throttle", "passwordpassword").await;

    let registered = register_oauth_client(&app, "workouts.write").await;
    let client_id = registered["client_id"].as_str().unwrap();
    let verifier = "batch3-throttle-verifier";
    let code = authorize_oauth_code(
        &app,
        client_id,
        "mcp-throttle",
        "passwordpassword-changed",
        "workouts.write",
        verifier,
    )
    .await;
    let tokens = exchange_token(&app, client_id, &code, verifier).await;
    let access_token = tokens["access_token"].as_str().unwrap();

    for id in 1..=2 {
        let body = mcp_call(
            &app,
            access_token,
            id,
            "create_workout",
            json!({
                "date": chrono::Utc::now(),
                "start_time": chrono::Utc::now()
            }),
        )
        .await;
        assert!(body.get("result").is_some());
    }

    let body = mcp_call(
        &app,
        access_token,
        3,
        "create_workout",
        json!({
            "date": chrono::Utc::now(),
            "start_time": chrono::Utc::now()
        }),
    )
    .await;
    assert_eq!(body["error"]["code"], -32029);
    assert_eq!(body["error"]["message"], "Too Many Requests");
}

#[actix_web::test]
async fn mcp_non_tool_requests_are_rate_limited_too() {
    let _env = TestEnv::new();
    let _registration_guard = EnvVarGuard::set("OAUTH_ALLOW_DYNAMIC_CLIENT_REGISTRATION", "true");
    let _rate_guard = EnvVarGuard::set("MCP_RATE_LIMIT_PER_MINUTE", "2");
    let (_db, admin_cookie, app) = setup_test_app().await;

    create_user_as_admin(&app, &admin_cookie, "mcp-init-throttle", "passwordpassword").await;
    let _user_cookie = login_cookie_active(&app, "mcp-init-throttle", "passwordpassword").await;

    let registered = register_oauth_client(&app, "workouts.read").await;
    let client_id = registered["client_id"].as_str().unwrap();
    let verifier = "batch3-init-throttle-verifier";
    let code = authorize_oauth_code(
        &app,
        client_id,
        "mcp-init-throttle",
        "passwordpassword-changed",
        "workouts.read",
        verifier,
    )
    .await;
    let tokens = exchange_token(&app, client_id, &code, verifier).await;
    let access_token = tokens["access_token"].as_str().unwrap();

    for id in 1..=2 {
        let req = test::TestRequest::post()
            .uri("/mcp")
            .insert_header((
                actix_web::http::header::AUTHORIZATION,
                format!("Bearer {access_token}"),
            ))
            .set_json(json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": "tools/list",
                "params": {}
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        let body = json_body(resp).await;
        assert!(body.get("result").is_some());
    }

    let req = test::TestRequest::post()
        .uri("/mcp")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {access_token}"),
        ))
        .set_json(json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "ping",
            "params": {}
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body = json_body(resp).await;
    assert_eq!(body["error"]["code"], -32029);
    assert_eq!(body["error"]["message"], "Too Many Requests");
}
