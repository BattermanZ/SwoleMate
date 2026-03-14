use crate::db::Database;
use crate::errors::AppError;
use crate::mcp::rate_limit;
use crate::middleware::McpPrincipal;
use crate::models::{
    CreateExerciseRequest, CreateSetRequest, CreateWorkoutRequest, UpdateExerciseRequest,
    UpdateWorkoutRequest,
};
use crate::services::{authz, exercises, progress, workouts};
use actix_web::{post, web, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Value};

const PROTOCOL_VERSION: &str = "2025-11-05";

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

fn rpc_result(id: Option<Value>, result: Value) -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(Value::Null),
        "result": result
    }))
}

fn rpc_error(id: Option<Value>, code: i64, message: &str) -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(Value::Null),
        "error": {
            "code": code,
            "message": message
        }
    }))
}

fn tool_definitions() -> Value {
    json!([
        {
            "name": "list_workouts",
            "description": "List workouts for the authenticated user.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        },
        {
            "name": "get_workout",
            "description": "Fetch one workout with its exercises and sets.",
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
            "name": "get_last_exercise_data",
            "description": "Fetch the last logged exercise instance and sets for an exercise type.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "exercise_type": { "type": "string" }
                },
                "required": ["exercise_type"],
                "additionalProperties": false
            }
        },
        {
            "name": "get_exercise_progress",
            "description": "Fetch progress history for an exercise type.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "exercise_type": { "type": "string" }
                },
                "required": ["exercise_type"],
                "additionalProperties": false
            }
        },
        {
            "name": "get_workout_stats",
            "description": "Fetch aggregate workout statistics for the authenticated user.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        },
        {
            "name": "get_volume_stats",
            "description": "Fetch volume statistics for an exercise type.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "exercise_type": { "type": "string" }
                },
                "required": ["exercise_type"],
                "additionalProperties": false
            }
        },
        {
            "name": "create_workout",
            "description": "Create a workout for the authenticated user.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "date": { "type": "string", "format": "date-time" },
                    "start_time": { "type": "string", "format": "date-time" },
                    "notes": { "type": "string" },
                    "timezone_offset_minutes": { "type": "integer" }
                },
                "required": ["date", "start_time"],
                "additionalProperties": false
            }
        },
        {
            "name": "add_exercise",
            "description": "Add an exercise to an existing workout.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "workout_id": { "type": "integer" },
                    "exercise_type": { "type": "string" },
                    "start_time": { "type": "string", "format": "date-time" },
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
            "description": "Replace all sets for an exercise.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "exercise_id": { "type": "integer" },
                    "sets": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "reps": { "type": "integer" },
                                "weight": { "type": "number" },
                                "notes": { "type": "string" },
                                "weight_left": { "type": "number" },
                                "weight_right": { "type": "number" }
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
            "description": "End an exercise and optionally update notes and settings.",
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
            "description": "End a workout and optionally update notes and feedback.",
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

fn rpc_error_from_app_error(err: AppError) -> (i64, &'static str, &'static str) {
    match err {
        AppError::BadRequest(_) => (-32602, "Invalid params", "invalid_params"),
        AppError::NotFound(_) => (-32004, "Not found", "not_found"),
        AppError::Forbidden | AppError::Unauthorized => (-32001, "Forbidden", "forbidden"),
        AppError::TooManyRequests(_) => (-32029, "Too Many Requests", "rate_limited"),
        AppError::Conflict(_) => (-32009, "Conflict", "conflict"),
        AppError::DatabaseError(_) | AppError::InternalError(_) => {
            (-32000, "Internal error", "internal_error")
        }
    }
}

fn parse_args<T>(args: &Value) -> Result<T, (i64, &'static str, &'static str)>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_value(args.clone()).map_err(|_| (-32602, "Invalid params", "invalid_params"))
}

fn require_scope(
    principal: &McpPrincipal,
    scope: authz::McpScope,
) -> Result<(), (i64, &'static str, &'static str)> {
    if authz::has_scope(&principal.scopes, scope) {
        Ok(())
    } else {
        Err((-32001, "Forbidden", "missing_scope"))
    }
}

#[post("/mcp")]
pub async fn handle_mcp(
    req: HttpRequest,
    principal: McpPrincipal,
    db: web::Data<Database>,
    body: web::Json<RpcRequest>,
) -> HttpResponse {
    let rpc = body.into_inner();
    if rpc.jsonrpc != "2.0" {
        return rpc_error(rpc.id, -32600, "Invalid Request");
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
        return rpc_error(rpc.id, -32029, "Too Many Requests");
    }

    match rpc.method.as_str() {
        "initialize" => rpc_result(
            rpc.id,
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
                }
            }),
        ),
        "notifications/initialized" => HttpResponse::Accepted().finish(),
        "ping" => rpc_result(rpc.id, json!({})),
        "tools/list" => rpc_result(rpc.id, json!({ "tools": tool_definitions() })),
        "resources/list" => rpc_result(rpc.id, json!({ "resources": [] })),
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

            let response = match call_tool(name, &args, &principal, db.get_ref()).await {
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
                    rpc_result(rpc.id, result)
                }
                Err((code, message, error_code)) => {
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
                    rpc_error(rpc.id, code, message)
                }
            };
            response
        }
        _ => rpc_error(rpc.id, -32601, "Method not found"),
    }
}

async fn call_tool(
    name: &str,
    args: &Value,
    principal: &McpPrincipal,
    db: &Database,
) -> Result<Value, (i64, &'static str, &'static str)> {
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
            let id = args.get("id").and_then(Value::as_i64).ok_or((
                -32602,
                "Invalid params",
                "invalid_params",
            ))?;
            let workout = workouts::get_workout_detail(db, principal.user_id, id)
                .await
                .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(json!(workout)))
        }
        "get_last_exercise_data" => {
            require_scope(principal, authz::McpScope::WorkoutsRead)?;
            let exercise_type = args.get("exercise_type").and_then(Value::as_str).ok_or((
                -32602,
                "Invalid params",
                "invalid_params",
            ))?;
            let data = exercises::get_last_exercise_data(db, principal.user_id, exercise_type)
                .await
                .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(json!(data)))
        }
        "get_exercise_progress" => {
            require_scope(principal, authz::McpScope::ProgressRead)?;
            let exercise_type = args.get("exercise_type").and_then(Value::as_str).ok_or((
                -32602,
                "Invalid params",
                "invalid_params",
            ))?;
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
            let exercise_type = args.get("exercise_type").and_then(Value::as_str).ok_or((
                -32602,
                "Invalid params",
                "invalid_params",
            ))?;
            let data = progress::get_volume_stats(db, principal.user_id, exercise_type)
                .await
                .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(data))
        }
        "create_workout" => {
            require_scope(principal, authz::McpScope::WorkoutsWrite)?;
            let request: CreateWorkoutRequest = parse_args(args)?;
            request
                .validate()
                .map_err(|_| (-32602, "Invalid params", "invalid_params"))?;
            let workout_id = workouts::create_workout(db, principal.user_id, &request)
                .await
                .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(json!({
                "id": workout_id,
                "message": "Workout created successfully"
            })))
        }
        "add_exercise" => {
            require_scope(principal, authz::McpScope::WorkoutsWrite)?;
            let request: CreateExerciseToolArgs = parse_args(args)?;
            request
                .request
                .validate()
                .map_err(|_| (-32602, "Invalid params", "invalid_params"))?;
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
                set.validate()
                    .map_err(|_| (-32602, "Invalid params", "invalid_params"))?;
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
            request
                .request
                .validate()
                .map_err(|_| (-32602, "Invalid params", "invalid_params"))?;
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
            request
                .request
                .validate()
                .map_err(|_| (-32602, "Invalid params", "invalid_params"))?;
            workouts::end_workout(db, principal.user_id, request.id, &request.request)
                .await
                .map_err(rpc_error_from_app_error)?;
            Ok(tool_success(json!({
                "message": "Workout ended successfully"
            })))
        }
        _ => Err((-32601, "Tool not found", "tool_not_found")),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(handle_mcp);
}
