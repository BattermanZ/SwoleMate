use crate::db::Database;
use crate::errors::AppError;
use crate::mcp::rate_limit;
use crate::middleware::McpPrincipal;
use crate::models::{
    CreateExerciseRequest, CreateSetRequest, CreateWorkoutRequest,
    CreateWorkoutTemplateFromWorkoutRequest, CreateWorkoutTemplateRequest,
    DuplicateWorkoutTemplateRequest, StartWorkoutFromTemplateRequest, UpdateExerciseRequest,
    UpdateWorkoutRequest, UpdateWorkoutTemplateRequest,
};
use crate::services::{authz, exercises, progress, templates, workouts};
use actix_web::http::header;
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Value};
use url::Url;

const PROTOCOL_VERSION: &str = "2025-11-25";
const FALLBACK_PROTOCOL_VERSION: &str = "2025-03-26";
const SERVER_INSTRUCTIONS: &str = "SwoleMate exposes one user's workout log. Start read workflows with list_workouts or list_templates, then call get_workout or get_template to find IDs for nested exercises. Use list_exercise_types before exercise-specific progress queries if you are unsure of the exact exercise name. Use ISO 8601 date-time strings; UTC is safest. timezone_offset_minutes is the user's local offset from UTC in minutes when known. Weight values are kilograms. Timed sets use duration_seconds. For split implements, provide weight_left and weight_right together. replace_sets is destructive: it replaces every set on that exercise with exactly the provided array.";

#[derive(Debug, Deserialize)]
struct RpcRequest {
    jsonrpc: String,
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Deserialize)]
struct RpcResponse {
    jsonrpc: String,
    #[serde(default)]
    id: Option<Value>,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct CreateExerciseToolArgs {
    workout_id: i64,
    #[serde(flatten)]
    request: CreateExerciseRequest,
}

#[derive(Debug, Deserialize)]
struct ReplaceSetsToolArgs {
    exercise_id: i64,
    sets: Vec<CreateSetRequest>,
}

#[derive(Debug, Deserialize)]
struct EndExerciseToolArgs {
    id: i64,
    #[serde(flatten)]
    request: UpdateExerciseRequest,
}

#[derive(Debug, Deserialize)]
struct EndWorkoutToolArgs {
    id: i64,
    #[serde(flatten)]
    request: UpdateWorkoutRequest,
}

#[derive(Debug, Deserialize)]
struct GetTemplateToolArgs {
    id: i64,
}

#[derive(Debug, Deserialize)]
struct UpdateTemplateToolArgs {
    id: i64,
    #[serde(flatten)]
    request: UpdateWorkoutTemplateRequest,
}

#[derive(Debug, Deserialize)]
struct DeleteTemplateToolArgs {
    id: i64,
}

#[derive(Debug, Deserialize)]
struct DuplicateTemplateToolArgs {
    id: i64,
    #[serde(flatten)]
    request: DuplicateWorkoutTemplateRequest,
}

#[derive(Debug, Deserialize)]
struct CreateTemplateFromWorkoutToolArgs {
    workout_id: i64,
    #[serde(flatten)]
    request: CreateWorkoutTemplateFromWorkoutRequest,
}

#[derive(Debug, Deserialize)]
struct StartWorkoutFromTemplateToolArgs {
    template_id: i64,
    #[serde(flatten)]
    request: StartWorkoutFromTemplateRequest,
}

fn rpc_result(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn rpc_error(id: Option<Value>, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(Value::Null),
        "error": {
            "code": code,
            "message": message
        }
    })
}

fn rpc_error_with_data(id: Option<Value>, code: i64, message: &str, data: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(Value::Null),
        "error": {
            "code": code,
            "message": message,
            "data": data
        }
    })
}

fn tool_definitions() -> Value {
    json!([
        {
            "name": "list_workouts",
            "description": "List the authenticated user's workouts. Use this first when you need workout IDs before fetching details or editing a workout.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        },
        {
            "name": "get_workout",
            "description": "Fetch one workout with nested exercises and sets. Use this to find exercise IDs before calling replace_sets or end_exercise.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "integer", "description": "Workout ID from list_workouts or a create_workout response." }
                },
                "required": ["id"],
                "additionalProperties": false
            }
        },
        {
            "name": "list_exercise_types",
            "description": "List exercise names already used by the authenticated user. Use this before exercise-specific progress tools when you need the exact exercise_type string.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        },
        {
            "name": "get_last_exercise_data",
            "description": "Fetch the most recent logged exercise instance and sets for an exact exercise_type. Call list_exercise_types first if unsure of spelling.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "exercise_type": { "type": "string", "description": "Exact exercise name, for example from list_exercise_types." }
                },
                "required": ["exercise_type"],
                "additionalProperties": false
            }
        },
        {
            "name": "get_exercise_progress",
            "description": "Fetch progress history for an exact exercise_type. Requires progress.read scope. Call list_exercise_types first if unsure of spelling.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "exercise_type": { "type": "string", "description": "Exact exercise name, for example from list_exercise_types." }
                },
                "required": ["exercise_type"],
                "additionalProperties": false
            }
        },
        {
            "name": "get_workout_stats",
            "description": "Fetch aggregate workout statistics for the authenticated user. Requires progress.read scope.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        },
        {
            "name": "get_volume_stats",
            "description": "Fetch volume statistics for an exact exercise_type. Requires progress.read scope. Call list_exercise_types first if unsure of spelling.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "exercise_type": { "type": "string", "description": "Exact exercise name, for example from list_exercise_types." }
                },
                "required": ["exercise_type"],
                "additionalProperties": false
            }
        },
        {
            "name": "list_templates",
            "description": "List workout templates for the authenticated user. Use this first when you need template IDs before fetching or starting from a template.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        },
        {
            "name": "get_template",
            "description": "Fetch one workout template with ordered exercises. Use this to inspect template contents before update, duplicate, delete, or start_workout_from_template.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "integer" }
                },
                "required": ["id"],
                "additionalProperties": false
            }
        },
        {
            "name": "create_workout",
            "description": "Create a workout for the authenticated user. Use ISO 8601 date-time strings; UTC is safest. Requires workouts.write scope.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "date": { "type": "string", "format": "date-time", "description": "Workout date/time as ISO 8601." },
                    "start_time": { "type": "string", "format": "date-time", "description": "Workout start time as ISO 8601." },
                    "notes": { "type": "string", "description": "Optional user-visible workout notes." },
                    "timezone_offset_minutes": { "type": "integer", "description": "Optional local offset from UTC in minutes, such as -300 or 60." }
                },
                "required": ["date", "start_time"],
                "additionalProperties": false
            }
        },
        {
            "name": "create_template",
            "description": "Create a workout template. Templates define planned exercises, not logged sets. Requires workouts.write scope.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "exercises": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "exercise_type": { "type": "string" },
                                "notes": { "type": "string" },
                                "per_side_weight": { "type": "boolean" },
                                "split_weight": { "type": "boolean" },
                                "settings": {
                                    "type": "array",
                                    "items": {
                                        "type": "object",
                                        "properties": {
                                            "key": { "type": "string" },
                                            "value": { "type": "string" }
                                        },
                                        "required": ["key", "value"],
                                        "additionalProperties": false
                                    }
                                }
                            },
                            "required": ["exercise_type"],
                            "additionalProperties": false
                        }
                    }
                },
                "required": ["name", "exercises"],
                "additionalProperties": false
            }
        },
        {
            "name": "update_template",
            "description": "Replace a workout template's name and full exercise list. Include every exercise that should remain. Requires workouts.write scope.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "integer" },
                    "name": { "type": "string" },
                    "exercises": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "exercise_type": { "type": "string" },
                                "notes": { "type": "string" },
                                "per_side_weight": { "type": "boolean" },
                                "split_weight": { "type": "boolean" },
                                "settings": {
                                    "type": "array",
                                    "items": {
                                        "type": "object",
                                        "properties": {
                                            "key": { "type": "string" },
                                            "value": { "type": "string" }
                                        },
                                        "required": ["key", "value"],
                                        "additionalProperties": false
                                    }
                                }
                            },
                            "required": ["exercise_type"],
                            "additionalProperties": false
                        }
                    }
                },
                "required": ["id", "name", "exercises"],
                "additionalProperties": false
            }
        },
        {
            "name": "delete_template",
            "description": "Delete a workout template by ID. This does not delete workouts already created from the template. Requires workouts.write scope.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "integer" }
                },
                "required": ["id"],
                "additionalProperties": false
            }
        },
        {
            "name": "duplicate_template",
            "description": "Duplicate a workout template, optionally with a new name. Requires workouts.write scope.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "integer" },
                    "name": { "type": "string" }
                },
                "required": ["id"],
                "additionalProperties": false
            }
        },
        {
            "name": "create_template_from_workout",
            "description": "Create a workout template from an existing workout's exercises. Logged sets are not copied into the template. Requires workouts.write scope.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "workout_id": { "type": "integer" },
                    "name": { "type": "string" }
                },
                "required": ["workout_id", "name"],
                "additionalProperties": false
            }
        },
        {
            "name": "start_workout_from_template",
            "description": "Create a new workout from a template. The new workout receives exercises from the template, usually without sets. Requires workouts.write scope.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "template_id": { "type": "integer" },
                    "date": { "type": "string", "format": "date-time" },
                    "start_time": { "type": "string", "format": "date-time" },
                    "timezone_offset_minutes": { "type": "integer" }
                },
                "required": ["template_id", "date", "start_time"],
                "additionalProperties": false
            }
        },
        {
            "name": "add_exercise",
            "description": "Add an exercise to an existing workout. Use the workout_id from create_workout, list_workouts, or get_workout. Requires workouts.write scope.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "workout_id": { "type": "integer", "description": "Workout ID owned by the authenticated user." },
                    "exercise_type": { "type": "string", "description": "Exercise name, such as Squat or Bench Press." },
                    "start_time": { "type": "string", "format": "date-time", "description": "Exercise start time as ISO 8601." },
                    "notes": { "type": "string" },
                    "per_side_weight": { "type": "boolean" },
                    "split_weight": { "type": "boolean" },
                    "settings": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "key": { "type": "string" },
                                "value": { "type": "string" }
                            },
                            "required": ["key", "value"],
                            "additionalProperties": false
                        }
                    }
                },
                "required": ["workout_id", "exercise_type", "start_time"],
                "additionalProperties": false
            }
        },
        {
            "name": "replace_sets",
            "description": "Destructively replace all sets for an exercise with exactly the provided sets array. Existing sets not included here are deleted. Fetch the workout first to confirm the exercise_id. Requires workouts.write scope.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "exercise_id": { "type": "integer", "description": "Exercise ID from get_workout." },
                    "sets": {
                        "type": "array",
                        "description": "Complete replacement list, maximum 100 sets.",
                        "maxItems": 100,
                        "items": {
                            "type": "object",
                            "properties": {
                                "reps": { "type": "integer", "description": "Repetition count, 0 to 500." },
                                "weight": { "type": "number", "description": "Weight in kilograms." },
                                "duration_seconds": { "type": "integer", "description": "Optional timed-set duration in seconds." },
                                "notes": { "type": "string", "description": "Optional set note." },
                                "weight_left": { "type": "number", "description": "Left-side weight in kilograms; provide together with weight_right." },
                                "weight_right": { "type": "number", "description": "Right-side weight in kilograms; provide together with weight_left." }
                            },
                            "required": ["reps", "weight"],
                            "additionalProperties": false
                        }
                    }
                },
                "required": ["exercise_id", "sets"],
                "additionalProperties": false
            }
        },
        {
            "name": "end_exercise",
            "description": "End an exercise and optionally update notes, split/per-side flags, and settings. Fetch the workout first to confirm the exercise ID. Requires workouts.write scope.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "integer" },
                    "end_time": { "type": "string", "format": "date-time" },
                    "notes": { "type": "string" },
                    "per_side_weight": { "type": "boolean" },
                    "split_weight": { "type": "boolean" },
                    "settings": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "key": { "type": "string" },
                                "value": { "type": "string" }
                            },
                            "required": ["key", "value"],
                            "additionalProperties": false
                        }
                    }
                },
                "required": ["id", "end_time"],
                "additionalProperties": false
            }
        },
        {
            "name": "end_workout",
            "description": "End a workout and optionally update notes and feedback. feedback must be one of 😊, 😐, or 😞. Requires workouts.write scope.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "integer" },
                    "end_time": { "type": "string", "format": "date-time" },
                    "notes": { "type": "string" },
                    "feedback": { "type": "string" }
                },
                "required": ["id", "end_time"],
                "additionalProperties": false
            }
        }
    ])
}

fn tool_success(payload: Value) -> Value {
    let text = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string());
    json!({
        "content": [
            {
                "type": "text",
                "text": text
            }
        ],
        "structuredContent": payload
    })
}

fn summarize_args(args: &Value) -> Value {
    match args {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| (key.clone(), summarize_audit_value(value)))
                .collect(),
        ),
        _ => json!({}),
    }
}

fn summarize_audit_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(nested_key, nested_value)| {
                    (nested_key.clone(), summarize_audit_value(nested_value))
                })
                .collect(),
        ),
        Value::Array(items) => json!({
            "count": items.len(),
            "items": items.iter().take(3).map(summarize_audit_value).collect::<Vec<_>>(),
        }),
        Value::String(text) => json!({
            "type": "string",
            "length": text.chars().count(),
        }),
        Value::Number(_) | Value::Bool(_) | Value::Null => value.clone(),
    }
}

fn rpc_error_from_app_error(err: AppError) -> (i64, &'static str, &'static str, Option<String>) {
    match err {
        AppError::BadRequest(detail) => (-32602, "Invalid params", "invalid_params", Some(detail)),
        AppError::NotFound(detail) => (-32004, "Not found", "not_found", Some(detail)),
        AppError::Forbidden | AppError::Unauthorized => (-32001, "Forbidden", "forbidden", None),
        AppError::TooManyRequests(detail) => {
            (-32029, "Too Many Requests", "rate_limited", Some(detail))
        }
        AppError::Conflict(detail) => (-32009, "Conflict", "conflict", Some(detail)),
        AppError::DatabaseError(_) | AppError::InternalError(_) => {
            (-32000, "Internal error", "internal_error", None)
        }
    }
}

fn parse_args<T>(args: &Value) -> Result<T, (i64, &'static str, &'static str, Option<String>)>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_value(args.clone()).map_err(|err| {
        (
            -32602,
            "Invalid params",
            "invalid_params",
            Some(err.to_string()),
        )
    })
}

fn require_scope(
    principal: &McpPrincipal,
    scope: authz::McpScope,
) -> Result<(), (i64, &'static str, &'static str, Option<String>)> {
    if authz::has_scope(&principal.scopes, scope) {
        Ok(())
    } else {
        Err((
            -32001,
            "Forbidden",
            "missing_scope",
            Some(format!("Required scope: {}", scope.as_str())),
        ))
    }
}

fn invalid_params(detail: impl ToString) -> (i64, &'static str, &'static str, Option<String>) {
    (
        -32602,
        "Invalid params",
        "invalid_params",
        Some(detail.to_string()),
    )
}

fn valid_request_id(id: &Value) -> bool {
    match id {
        Value::String(_) => true,
        Value::Number(number) => number.as_i64().is_some() || number.as_u64().is_some(),
        _ => false,
    }
}

fn configured_origin(raw: Option<String>) -> Option<String> {
    raw.and_then(|value| {
        let trimmed = value.trim().trim_end_matches('/').to_string();
        if trimmed.is_empty() {
            return None;
        }
        if let Ok(url) = Url::parse(&trimmed) {
            if let Some(host) = url.host_str() {
                let mut origin = format!("{}://{}", url.scheme(), host);
                if let Some(port) = url.port() {
                    origin.push_str(&format!(":{port}"));
                }
                return Some(origin);
            }
        } else {
            return Some(trimmed);
        }
        None
    })
}

fn allowed_origin_values() -> Vec<String> {
    let mut origins = std::env::var("CORS_ALLOWED_ORIGINS")
        .ok()
        .map(|raw| {
            raw.split(',')
                .filter_map(|value| configured_origin(Some(value.to_string())))
                .collect::<Vec<_>>()
        })
        .filter(|values| !values.is_empty())
        .unwrap_or_else(|| {
            configured_origin(
                std::env::var("FRONTEND_URL")
                    .ok()
                    .or_else(|| Some("http://localhost:2470".to_string())),
            )
            .into_iter()
            .collect()
        });

    if let Some(origin) = configured_origin(std::env::var("MCP_PUBLIC_BASE_URL").ok()) {
        origins.push(origin);
    }
    if cfg!(debug_assertions) {
        origins.push("http://localhost:2470".to_string());
        origins.push("http://127.0.0.1:2470".to_string());
    }
    origins.sort();
    origins.dedup();
    origins
}

fn origin_is_allowed(req: &HttpRequest) -> bool {
    let Some(origin) = req
        .headers()
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| configured_origin(Some(value.to_string())))
    else {
        return true;
    };

    allowed_origin_values()
        .iter()
        .any(|allowed| allowed == &origin)
}

fn protocol_version_is_supported(req: &HttpRequest) -> bool {
    let Some(version) = req
        .headers()
        .get("MCP-Protocol-Version")
        .and_then(|value| value.to_str().ok())
    else {
        return true;
    };

    matches!(version, PROTOCOL_VERSION | FALLBACK_PROTOCOL_VERSION)
}

fn accepts_json_or_event_stream(req: &HttpRequest) -> bool {
    let Some(accept) = req
        .headers()
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
    else {
        return true;
    };

    accept
        .split(',')
        .map(|item| item.split(';').next().unwrap_or("").trim())
        .any(|item| {
            matches!(
                item,
                "*/*" | "application/*" | "application/json" | "text/event-stream"
            )
        })
}

#[post("/mcp")]
pub async fn handle_mcp(
    req: HttpRequest,
    principal: McpPrincipal,
    db: web::Data<Database>,
    body: web::Bytes,
) -> HttpResponse {
    if !origin_is_allowed(&req) {
        return HttpResponse::Forbidden().json(rpc_error(None, -32001, "Forbidden"));
    }
    if !protocol_version_is_supported(&req) {
        return HttpResponse::BadRequest().json(rpc_error(None, -32600, "Invalid Request"));
    }
    if !accepts_json_or_event_stream(&req) {
        return HttpResponse::NotAcceptable().json(rpc_error(None, -32600, "Invalid Request"));
    }

    let payload: Value = match serde_json::from_slice(&body) {
        Ok(payload) => payload,
        Err(_) => {
            return HttpResponse::Ok().json(rpc_error(None, -32700, "Parse error"));
        }
    };

    match payload {
        Value::Array(items) => {
            if items.is_empty() {
                return HttpResponse::Ok().json(rpc_error(None, -32600, "Invalid Request"));
            }

            let mut responses = Vec::new();
            for item in items {
                if let Some(response) =
                    handle_mcp_message(&req, &principal, db.get_ref(), item).await
                {
                    responses.push(response);
                }
            }

            if responses.is_empty() {
                HttpResponse::Accepted().finish()
            } else {
                HttpResponse::Ok().json(responses)
            }
        }
        item => match handle_mcp_message(&req, &principal, db.get_ref(), item).await {
            Some(response) => HttpResponse::Ok().json(response),
            None => HttpResponse::Accepted().finish(),
        },
    }
}

async fn handle_mcp_message(
    req: &HttpRequest,
    principal: &McpPrincipal,
    db: &Database,
    payload: Value,
) -> Option<Value> {
    let explicit_null_id = payload
        .as_object()
        .and_then(|map| map.get("id"))
        .is_some_and(Value::is_null);
    if explicit_null_id {
        return Some(rpc_error(Some(Value::Null), -32600, "Invalid Request"));
    }

    if let Ok(response) = serde_json::from_value::<RpcResponse>(payload.clone()) {
        if response.jsonrpc == "2.0"
            && response.id.is_some()
            && (response.result.is_some() || response.error.is_some())
        {
            return None;
        }
    }

    let rpc: RpcRequest = match serde_json::from_value(payload) {
        Ok(rpc) => rpc,
        Err(_) => return Some(rpc_error(None, -32600, "Invalid Request")),
    };

    if rpc.jsonrpc != "2.0" {
        return Some(rpc_error(rpc.id, -32600, "Invalid Request"));
    }

    if rpc.id.is_none() {
        return None;
    } else {
        match rpc.id.as_ref() {
            Some(id) if valid_request_id(id) => {}
            _ => return Some(rpc_error(rpc.id, -32600, "Invalid Request")),
        }
    }

    let now = Utc::now();
    let rate_limit_key = format!("{}:{}", principal.client_id, principal.user_id);
    if !rate_limit::admit_request(&rate_limit_key, now) {
        let name = if rpc.method == "tools/call" {
            rpc.params
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("tools/call")
        } else {
            rpc.method.as_str()
        };
        let args = if rpc.method == "tools/call" {
            rpc.params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}))
        } else {
            rpc.params.clone()
        };
        let audit_args = summarize_args(&args);
        let ip = req
            .connection_info()
            .realip_remote_addr()
            .map(str::to_string);
        let user_agent = req
            .headers()
            .get(actix_web::http::header::USER_AGENT)
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);
        let _ = db
            .write_mcp_audit_log(
                Some(principal.user_id),
                Some(&principal.client_id),
                name,
                false,
                Some("rate_limited"),
                Some(&audit_args),
                ip.as_deref(),
                user_agent.as_deref(),
            )
            .await;
        return Some(rpc_error(rpc.id, -32029, "Too Many Requests"));
    }

    match rpc.method.as_str() {
        "initialize" => Some(rpc_result(
            rpc.id.expect("validated request id"),
            json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {
                    "tools": {
                        "listChanged": false
                    }
                },
                "serverInfo": {
                    "name": "swolemate",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "instructions": SERVER_INSTRUCTIONS
            }),
        )),
        "notifications/initialized" => None,
        "ping" => Some(rpc_result(rpc.id.expect("validated request id"), json!({}))),
        "tools/list" => Some(rpc_result(
            rpc.id.expect("validated request id"),
            json!({ "tools": tool_definitions() }),
        )),
        "resources/list" => Some(rpc_result(
            rpc.id.expect("validated request id"),
            json!({ "resources": [] }),
        )),
        "tools/call" => {
            let name = rpc
                .params
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let args = rpc
                .params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            let audit_args = summarize_args(&args);
            let ip = req
                .connection_info()
                .realip_remote_addr()
                .map(str::to_string);
            let user_agent = req
                .headers()
                .get(actix_web::http::header::USER_AGENT)
                .and_then(|value| value.to_str().ok())
                .map(str::to_string);

            let response = match call_tool(name, &args, principal, db).await {
                Ok(result) => {
                    let _ = db
                        .write_mcp_audit_log(
                            Some(principal.user_id),
                            Some(&principal.client_id),
                            name,
                            true,
                            None,
                            Some(&audit_args),
                            ip.as_deref(),
                            user_agent.as_deref(),
                        )
                        .await;
                    rpc_result(rpc.id.expect("validated request id"), result)
                }
                Err((code, message, error_code, detail)) => {
                    let _ = db
                        .write_mcp_audit_log(
                            Some(principal.user_id),
                            Some(&principal.client_id),
                            name,
                            false,
                            Some(error_code),
                            Some(&audit_args),
                            ip.as_deref(),
                            user_agent.as_deref(),
                        )
                        .await;
                    let mut data = json!({ "code": error_code });
                    if let Some(detail) = detail {
                        data["detail"] = json!(detail);
                    }
                    rpc_error_with_data(rpc.id, code, message, data)
                }
            };
            Some(response)
        }
        _ => Some(rpc_error(rpc.id, -32601, "Method not found")),
    }
}

async fn call_tool(
    name: &str,
    args: &Value,
    principal: &McpPrincipal,
    db: &Database,
) -> Result<Value, (i64, &'static str, &'static str, Option<String>)> {
    match name {
        "list_workouts" => {
            require_scope(principal, authz::McpScope::WorkoutsRead)?;
            let workouts = workouts::list_workouts(db, principal.user_id)
                .await
                .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(json!(workouts)))
        }
        "get_workout" => {
            require_scope(principal, authz::McpScope::WorkoutsRead)?;
            let id = args
                .get("id")
                .and_then(Value::as_i64)
                .ok_or_else(|| invalid_params("id is required and must be an integer"))?;
            let workout = workouts::get_workout_detail(db, principal.user_id, id)
                .await
                .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(json!(workout)))
        }
        "list_exercise_types" => {
            require_scope(principal, authz::McpScope::WorkoutsRead)?;
            let types = exercises::list_exercise_types(db, principal.user_id)
                .await
                .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(json!(types)))
        }
        "list_templates" => {
            require_scope(principal, authz::McpScope::WorkoutsRead)?;
            let templates = templates::list_templates(db, principal.user_id)
                .await
                .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(json!(templates)))
        }
        "get_template" => {
            require_scope(principal, authz::McpScope::WorkoutsRead)?;
            let request: GetTemplateToolArgs = parse_args(args)?;
            let template = templates::get_template(db, principal.user_id, request.id)
                .await
                .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(json!(template)))
        }
        "get_last_exercise_data" => {
            require_scope(principal, authz::McpScope::WorkoutsRead)?;
            let exercise_type = args
                .get("exercise_type")
                .and_then(Value::as_str)
                .ok_or_else(|| invalid_params("exercise_type is required and must be a string"))?;
            let data = exercises::get_last_exercise_data(db, principal.user_id, exercise_type)
                .await
                .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(json!(data)))
        }
        "get_exercise_progress" => {
            require_scope(principal, authz::McpScope::ProgressRead)?;
            let exercise_type = args
                .get("exercise_type")
                .and_then(Value::as_str)
                .ok_or_else(|| invalid_params("exercise_type is required and must be a string"))?;
            let data = progress::get_exercise_progress(db, principal.user_id, exercise_type)
                .await
                .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(json!(data)))
        }
        "get_workout_stats" => {
            require_scope(principal, authz::McpScope::ProgressRead)?;
            let data = progress::get_workout_stats(db, principal.user_id)
                .await
                .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(data))
        }
        "get_volume_stats" => {
            require_scope(principal, authz::McpScope::ProgressRead)?;
            let exercise_type = args
                .get("exercise_type")
                .and_then(Value::as_str)
                .ok_or_else(|| invalid_params("exercise_type is required and must be a string"))?;
            let data = progress::get_volume_stats(db, principal.user_id, exercise_type)
                .await
                .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(data))
        }
        "create_workout" => {
            require_scope(principal, authz::McpScope::WorkoutsWrite)?;
            let request: CreateWorkoutRequest = parse_args(args)?;
            request.validate().map_err(invalid_params)?;
            let workout_id = workouts::create_workout(db, principal.user_id, &request)
                .await
                .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(json!({
                "id": workout_id,
                "message": "Workout created successfully"
            })))
        }
        "create_template" => {
            require_scope(principal, authz::McpScope::WorkoutsWrite)?;
            let request: CreateWorkoutTemplateRequest = parse_args(args)?;
            request.validate().map_err(invalid_params)?;
            let template = templates::create_template(db, principal.user_id, &request)
                .await
                .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(json!(template)))
        }
        "update_template" => {
            require_scope(principal, authz::McpScope::WorkoutsWrite)?;
            let request: UpdateTemplateToolArgs = parse_args(args)?;
            request.request.validate().map_err(invalid_params)?;
            let template =
                templates::update_template(db, principal.user_id, request.id, &request.request)
                    .await
                    .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(json!(template)))
        }
        "delete_template" => {
            require_scope(principal, authz::McpScope::WorkoutsWrite)?;
            let request: DeleteTemplateToolArgs = parse_args(args)?;
            templates::delete_template(db, principal.user_id, request.id)
                .await
                .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(json!({
                "message": "Template deleted successfully"
            })))
        }
        "duplicate_template" => {
            require_scope(principal, authz::McpScope::WorkoutsWrite)?;
            let request: DuplicateTemplateToolArgs = parse_args(args)?;
            request.request.validate().map_err(invalid_params)?;
            let template =
                templates::duplicate_template(db, principal.user_id, request.id, &request.request)
                    .await
                    .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(json!(template)))
        }
        "create_template_from_workout" => {
            require_scope(principal, authz::McpScope::WorkoutsWrite)?;
            let request: CreateTemplateFromWorkoutToolArgs = parse_args(args)?;
            request.request.validate().map_err(invalid_params)?;
            let template = templates::create_template_from_workout(
                db,
                principal.user_id,
                request.workout_id,
                &request.request,
            )
            .await
            .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(json!(template)))
        }
        "start_workout_from_template" => {
            require_scope(principal, authz::McpScope::WorkoutsWrite)?;
            let request: StartWorkoutFromTemplateToolArgs = parse_args(args)?;
            request.request.validate().map_err(invalid_params)?;
            let workout_id = templates::start_workout_from_template(
                db,
                principal.user_id,
                request.template_id,
                &request.request,
            )
            .await
            .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(json!({
                "id": workout_id,
                "message": "Workout started from template"
            })))
        }
        "add_exercise" => {
            require_scope(principal, authz::McpScope::WorkoutsWrite)?;
            let request: CreateExerciseToolArgs = parse_args(args)?;
            request.request.validate().map_err(invalid_params)?;
            let exercise_id = exercises::create_exercise(
                db,
                principal.user_id,
                request.workout_id,
                &request.request,
            )
            .await
            .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(json!({
                "id": exercise_id,
                "message": "Exercise created successfully"
            })))
        }
        "replace_sets" => {
            require_scope(principal, authz::McpScope::WorkoutsWrite)?;
            let request: ReplaceSetsToolArgs = parse_args(args)?;
            for set in &request.sets {
                set.validate().map_err(invalid_params)?;
            }
            let sets =
                exercises::replace_sets(db, principal.user_id, request.exercise_id, &request.sets)
                    .await
                    .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(json!(sets)))
        }
        "end_exercise" => {
            require_scope(principal, authz::McpScope::WorkoutsWrite)?;
            let request: EndExerciseToolArgs = parse_args(args)?;
            request.request.validate().map_err(invalid_params)?;
            exercises::end_exercise(db, principal.user_id, request.id, &request.request)
                .await
                .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(json!({
                "message": "Exercise ended successfully"
            })))
        }
        "end_workout" => {
            require_scope(principal, authz::McpScope::WorkoutsWrite)?;
            let request: EndWorkoutToolArgs = parse_args(args)?;
            request.request.validate().map_err(invalid_params)?;
            workouts::end_workout(db, principal.user_id, request.id, &request.request)
                .await
                .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(json!({
                "message": "Workout ended successfully"
            })))
        }
        _ => Err((-32601, "Tool not found", "tool_not_found", None)),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(handle_mcp).service(handle_mcp_get);
}

#[get("/mcp")]
pub async fn handle_mcp_get(req: HttpRequest) -> HttpResponse {
    if !origin_is_allowed(&req) {
        return HttpResponse::Forbidden().finish();
    }
    if !protocol_version_is_supported(&req) {
        return HttpResponse::BadRequest().finish();
    }

    HttpResponse::MethodNotAllowed()
        .insert_header((header::ALLOW, "POST"))
        .finish()
}
