use crate::auth::password;
use crate::auth::rate_limit::{
    clear_ip_failures, is_ip_rate_limited, record_ip_failure, request_ip,
};
use crate::auth::{generate_session_token, hash_session_token};
use crate::db::Database;
use crate::oauth::OAuthConfig;
use crate::services::authz::normalize_scopes;
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use base64::Engine;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
pub struct OAuthRegisterRequest {
    pub client_name: String,
    pub redirect_uris: Vec<String>,
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Debug, Serialize)]
struct OAuthRegisterResponse {
    client_id: String,
    client_name: String,
    redirect_uris: Vec<String>,
    grant_types: Vec<String>,
    response_types: Vec<String>,
    token_endpoint_auth_method: String,
    scope: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OAuthAuthorizeQuery {
    response_type: String,
    client_id: String,
    redirect_uri: String,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    code_challenge: Option<String>,
    #[serde(default)]
    code_challenge_method: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthAuthorizeForm {
    response_type: String,
    client_id: String,
    redirect_uri: String,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    code_challenge: Option<String>,
    #[serde(default)]
    code_challenge_method: Option<String>,
    username: String,
    password: String,
    approve: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthTokenForm {
    grant_type: String,
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    redirect_uri: Option<String>,
    #[serde(default)]
    client_id: Option<String>,
    #[serde(default)]
    code_verifier: Option<String>,
    #[serde(default)]
    refresh_token: Option<String>,
}

fn validate_client_redirect_and_scopes(
    client: &crate::db::oauth::OAuthClient,
    redirect_uri: &str,
    requested_scopes: &[String],
) -> Result<(), HttpResponse> {
    if client.disabled_at.is_some() {
        return Err(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "invalid_client"
        })));
    }
    if !client.redirect_uris.iter().any(|uri| uri == redirect_uri) {
        return Err(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "invalid_request",
            "error_description": "redirect_uri is not registered"
        })));
    }
    if requested_scopes
        .iter()
        .any(|scope| !client.scopes.iter().any(|allowed| allowed == scope))
    {
        return Err(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "invalid_scope"
        })));
    }
    Ok(())
}

fn build_authorize_html(query: &OAuthAuthorizeQuery, client_name: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>SwoleMate OAuth Authorization</title>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <style>
    body {{ font-family: sans-serif; background: #f5f2e9; color: #1c1b18; margin: 0; padding: 2rem; }}
    main {{ max-width: 32rem; margin: 0 auto; background: white; border-radius: 16px; padding: 2rem; box-shadow: 0 12px 40px rgba(0,0,0,0.08); }}
    label {{ display: block; margin-top: 1rem; font-weight: 600; }}
    input {{ width: 100%; padding: 0.75rem; margin-top: 0.35rem; border: 1px solid #d0c7b8; border-radius: 10px; box-sizing: border-box; }}
    button {{ margin-top: 1.25rem; width: 100%; padding: 0.85rem; border: 0; border-radius: 999px; background: #0a6847; color: white; font-weight: 700; }}
    .scopes {{ background: #f7f7f3; padding: 0.85rem 1rem; border-radius: 12px; margin-top: 1rem; }}
  </style>
</head>
<body>
  <main>
    <h1>Authorize {client_name}</h1>
    <p>This client is requesting access to your SwoleMate MCP server.</p>
    <div class="scopes"><strong>Requested scopes:</strong> {scopes}</div>
    <form method="post" action="/oauth/authorize">
      <input type="hidden" name="response_type" value="{response_type}">
      <input type="hidden" name="client_id" value="{client_id}">
      <input type="hidden" name="redirect_uri" value="{redirect_uri}">
      <input type="hidden" name="scope" value="{scope}">
      <input type="hidden" name="state" value="{state}">
      <input type="hidden" name="code_challenge" value="{code_challenge}">
      <input type="hidden" name="code_challenge_method" value="{code_challenge_method}">
      <input type="hidden" name="approve" value="yes">
      <label>Username <input name="username" autocomplete="username" required></label>
      <label>Password <input name="password" type="password" autocomplete="current-password" required></label>
      <button type="submit">Approve and Continue</button>
    </form>
  </main>
</body>
</html>"#,
        client_name = escape_html(client_name),
        scopes = escape_html(query.scope.as_deref().unwrap_or("default scopes")),
        response_type = escape_html(&query.response_type),
        client_id = escape_html(&query.client_id),
        redirect_uri = escape_html(&query.redirect_uri),
        scope = escape_html(query.scope.as_deref().unwrap_or("")),
        state = escape_html(query.state.as_deref().unwrap_or("")),
        code_challenge = escape_html(query.code_challenge.as_deref().unwrap_or("")),
        code_challenge_method = escape_html(query.code_challenge_method.as_deref().unwrap_or("")),
    )
}

fn escape_html(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn code_challenge_for(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest)
}

#[post("/oauth/register")]
pub async fn register_client(
    db: web::Data<Database>,
    cfg: web::Data<OAuthConfig>,
    body: web::Json<OAuthRegisterRequest>,
) -> HttpResponse {
    if !cfg.allow_dynamic_client_registration {
        return HttpResponse::NotFound().finish();
    }

    if body.client_name.trim().is_empty() || body.redirect_uris.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "invalid_client_metadata"
        }));
    }

    let scopes = body
        .scope
        .as_deref()
        .map(normalize_scopes)
        .filter(|scopes| !scopes.is_empty())
        .unwrap_or_else(|| cfg.default_scopes.clone());

    let redirect_uris_json = match serde_json::to_string(&body.redirect_uris) {
        Ok(value) => value,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "server_error"
            }))
        }
    };
    let scopes_json = match serde_json::to_string(&scopes) {
        Ok(value) => value,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "server_error"
            }))
        }
    };

    let client_id = format!("swole_{}", generate_session_token());
    match db
        .create_oauth_client(
            &client_id,
            None,
            body.client_name.trim(),
            &redirect_uris_json,
            &scopes_json,
        )
        .await
    {
        Ok(()) => HttpResponse::Created().json(OAuthRegisterResponse {
            client_id,
            client_name: body.client_name.trim().to_string(),
            redirect_uris: body.redirect_uris.clone(),
            grant_types: vec![
                "authorization_code".to_string(),
                "refresh_token".to_string(),
            ],
            response_types: vec!["code".to_string()],
            token_endpoint_auth_method: "none".to_string(),
            scope: scopes.join(" "),
        }),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "server_error"
        })),
    }
}

#[get("/oauth/authorize")]
pub async fn authorize_get(
    db: web::Data<Database>,
    query: web::Query<OAuthAuthorizeQuery>,
    cfg: web::Data<OAuthConfig>,
) -> HttpResponse {
    let query = query.into_inner();
    if query.response_type != "code" {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "unsupported_response_type"
        }));
    }

    let client = match db.get_oauth_client(&query.client_id).await {
        Ok(Some(client)) => client,
        Ok(None) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_client"
            }))
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "server_error"
            }))
        }
    };

    let scopes = query
        .scope
        .as_deref()
        .map(normalize_scopes)
        .filter(|scopes| !scopes.is_empty())
        .unwrap_or_else(|| cfg.default_scopes.clone());

    if let Err(resp) = validate_client_redirect_and_scopes(&client, &query.redirect_uri, &scopes) {
        return resp;
    }

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(build_authorize_html(&query, &client.client_name))
}

#[post("/oauth/authorize")]
pub async fn authorize_post(
    db: web::Data<Database>,
    req: HttpRequest,
    form: web::Form<OAuthAuthorizeForm>,
    cfg: web::Data<OAuthConfig>,
) -> HttpResponse {
    let form = form.into_inner();
    let now = Utc::now();
    let client_ip = request_ip(&req);
    if form.approve.as_deref() != Some("yes") {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "access_denied"
        }));
    }
    if form.response_type != "code" {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "unsupported_response_type"
        }));
    }

    let client = match db.get_oauth_client(&form.client_id).await {
        Ok(Some(client)) => client,
        Ok(None) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_client"
            }))
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "server_error"
            }))
        }
    };

    let scopes = form
        .scope
        .as_deref()
        .map(normalize_scopes)
        .filter(|scopes| !scopes.is_empty())
        .unwrap_or_else(|| cfg.default_scopes.clone());

    if let Err(resp) = validate_client_redirect_and_scopes(&client, &form.redirect_uri, &scopes) {
        return resp;
    }

    if form.code_challenge_method.as_deref() != Some("S256") || form.code_challenge.is_none() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "invalid_request",
            "error_description": "PKCE with S256 is required"
        }));
    }

    if let Some(ip) = client_ip.as_deref() {
        if is_ip_rate_limited(ip, now) {
            return HttpResponse::TooManyRequests().json(serde_json::json!({
                "error": "too_many_requests",
                "error_description": "Too many login attempts from this IP. Try again later."
            }));
        }
    }

    let user = match db.get_user_by_username(&form.username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            if let Some(ip) = client_ip.as_deref() {
                record_ip_failure(ip, now);
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "access_denied"
            }));
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "server_error"
            }))
        }
    };

    if user.disabled_at.is_some() || user.must_change_password {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "access_denied"
        }));
    }

    let password_ok = match password::verify_password(&user.password_hash, &form.password) {
        Ok(result) => result,
        Err(_) => false,
    };
    if !password_ok {
        let _ = db.record_failed_login(user.id).await;
        if let Some(ip) = client_ip.as_deref() {
            record_ip_failure(ip, now);
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "access_denied"
        }));
    }
    let _ = db.reset_login_failures(user.id).await;
    if let Some(ip) = client_ip.as_deref() {
        clear_ip_failures(ip);
    }

    let code = generate_session_token();
    let code_hash = hash_session_token(&code);
    let scopes_json = match serde_json::to_string(&scopes) {
        Ok(value) => value,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "server_error"
            }))
        }
    };

    if db
        .create_oauth_authorization_code(
            &code_hash,
            &form.client_id,
            user.id,
            &form.redirect_uri,
            &scopes_json,
            form.code_challenge.as_deref(),
            form.code_challenge_method.as_deref(),
            Utc::now() + chrono::Duration::minutes(10),
        )
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "server_error"
        }));
    }

    let _ = db
        .store_oauth_consent(user.id, &form.client_id, &scopes_json)
        .await;

    let mut location = format!("{}?code={}", form.redirect_uri, urlencoding::encode(&code));
    if let Some(state) = form.state.as_deref() {
        if !state.is_empty() {
            location.push_str("&state=");
            location.push_str(&urlencoding::encode(state));
        }
    }

    HttpResponse::Found()
        .insert_header((actix_web::http::header::LOCATION, location))
        .finish()
}

#[post("/oauth/token")]
pub async fn token(
    db: web::Data<Database>,
    cfg: web::Data<OAuthConfig>,
    form: web::Form<OAuthTokenForm>,
) -> HttpResponse {
    let form = form.into_inner();
    match form.grant_type.as_str() {
        "authorization_code" => exchange_authorization_code(db, cfg, form).await,
        "refresh_token" => exchange_refresh_token(db, cfg, form).await,
        _ => HttpResponse::BadRequest().json(serde_json::json!({
            "error": "unsupported_grant_type"
        })),
    }
}

async fn exchange_authorization_code(
    db: web::Data<Database>,
    cfg: web::Data<OAuthConfig>,
    form: OAuthTokenForm,
) -> HttpResponse {
    let Some(code) = form.code.as_deref() else {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "invalid_request" }));
    };
    let Some(client_id) = form.client_id.as_deref() else {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "invalid_client" }));
    };
    let Some(redirect_uri) = form.redirect_uri.as_deref() else {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "invalid_request" }));
    };
    let Some(code_verifier) = form.code_verifier.as_deref() else {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "invalid_request" }));
    };

    let client = match db.get_oauth_client(client_id).await {
        Ok(Some(client)) => client,
        _ => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "error": "invalid_client" }))
        }
    };

    let code_hash = hash_session_token(code);
    let Some(stored) = (match db.get_oauth_authorization_code(&code_hash).await {
        Ok(value) => value,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "server_error" }))
        }
    }) else {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "invalid_grant" }));
    };

    if stored.used_at.is_some()
        || stored.client_id != client.client_id
        || stored.redirect_uri != redirect_uri
        || stored.expires_at <= Utc::now()
    {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "invalid_grant" }));
    }

    let Some(challenge) = stored.pkce_code_challenge.as_deref() else {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "invalid_grant" }));
    };
    if stored.pkce_method.as_deref() != Some("S256")
        || code_challenge_for(code_verifier) != challenge
    {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "invalid_grant" }));
    }

    match db.mark_oauth_authorization_code_used(stored.id).await {
        Ok(true) => {}
        Ok(false) => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "error": "invalid_grant" }));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "server_error" }));
        }
    }

    let access_token = generate_session_token();
    let refresh_token = generate_session_token();
    let access_token_hash = hash_session_token(&access_token);
    let refresh_token_hash = hash_session_token(&refresh_token);
    let scopes_json = match serde_json::to_string(&stored.scopes) {
        Ok(value) => value,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "server_error" }))
        }
    };

    if db
        .create_oauth_access_token(
            &access_token_hash,
            &stored.client_id,
            stored.user_id,
            &scopes_json,
            Utc::now() + cfg.access_token_ttl,
        )
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": "server_error" }));
    }
    if db
        .create_oauth_refresh_token(
            &refresh_token_hash,
            &stored.client_id,
            stored.user_id,
            &scopes_json,
            Utc::now() + cfg.refresh_token_ttl,
        )
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": "server_error" }));
    }

    HttpResponse::Ok().json(serde_json::json!({
        "access_token": access_token,
        "token_type": "Bearer",
        "expires_in": cfg.access_token_ttl.num_seconds(),
        "scope": stored.scopes.join(" "),
        "refresh_token": refresh_token
    }))
}

async fn exchange_refresh_token(
    db: web::Data<Database>,
    cfg: web::Data<OAuthConfig>,
    form: OAuthTokenForm,
) -> HttpResponse {
    let Some(refresh_token) = form.refresh_token.as_deref() else {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "invalid_request" }));
    };
    let Some(client_id) = form.client_id.as_deref() else {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "invalid_client" }));
    };

    let token_hash = hash_session_token(refresh_token);
    let Some((stored_client_id, user_id, scopes, expires_at, revoked_at)) =
        (match db.get_oauth_refresh_token(&token_hash).await {
            Ok(value) => value,
            Err(_) => {
                return HttpResponse::InternalServerError()
                    .json(serde_json::json!({ "error": "server_error" }))
            }
        })
    else {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "invalid_grant" }));
    };

    if stored_client_id != client_id || revoked_at.is_some() || expires_at <= Utc::now() {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "invalid_grant" }));
    }

    let access_token = generate_session_token();
    let access_token_hash = hash_session_token(&access_token);
    let scopes_json = match serde_json::to_string(&scopes) {
        Ok(value) => value,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "server_error" }))
        }
    };
    if db
        .create_oauth_access_token(
            &access_token_hash,
            client_id,
            user_id,
            &scopes_json,
            Utc::now() + cfg.access_token_ttl,
        )
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": "server_error" }));
    }

    HttpResponse::Ok().json(serde_json::json!({
        "access_token": access_token,
        "token_type": "Bearer",
        "expires_in": cfg.access_token_ttl.num_seconds(),
        "scope": scopes.join(" ")
    }))
}

#[get("/.well-known/oauth-authorization-server")]
pub async fn authorization_server_metadata(cfg: web::Data<OAuthConfig>) -> HttpResponse {
    let mut metadata = serde_json::json!({
        "issuer": cfg.issuer,
        "authorization_endpoint": cfg.authorization_endpoint,
        "token_endpoint": cfg.token_endpoint,
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code", "refresh_token"],
        "code_challenge_methods_supported": ["S256"],
        "token_endpoint_auth_methods_supported": ["none"]
    });
    if cfg.allow_dynamic_client_registration {
        metadata["registration_endpoint"] = serde_json::json!(cfg.registration_endpoint);
    }
    HttpResponse::Ok().json(metadata)
}

#[get("/.well-known/oauth-protected-resource")]
pub async fn protected_resource_metadata(cfg: web::Data<OAuthConfig>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "resource": cfg.resource,
        "authorization_servers": [cfg.issuer],
        "scopes_supported": ["workouts.read", "progress.read", "workouts.write"]
    }))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(register_client)
        .service(authorize_get)
        .service(authorize_post)
        .service(token)
        .service(authorization_server_metadata)
        .service(protected_resource_metadata);
}
