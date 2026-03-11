use crate::services::authz::{normalize_scopes, McpScope};
use chrono::Duration;

pub mod routes;

#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub registration_endpoint: String,
    pub protected_resource_endpoint: String,
    pub resource: String,
    pub access_token_ttl: Duration,
    pub refresh_token_ttl: Duration,
    pub default_scopes: Vec<String>,
    pub allow_dynamic_client_registration: bool,
}

impl OAuthConfig {
    pub fn from_env() -> Self {
        let base = std::env::var("MCP_PUBLIC_BASE_URL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| std::env::var("FRONTEND_URL").ok())
            .unwrap_or_else(|| "http://localhost:2470".to_string())
            .trim_end_matches('/')
            .to_string();

        let default_scopes = std::env::var("OAUTH_DEFAULT_SCOPES")
            .ok()
            .map(|value| normalize_scopes(&value))
            .filter(|scopes| !scopes.is_empty())
            .unwrap_or_else(|| {
                vec![
                    McpScope::WorkoutsRead.as_str().to_string(),
                    McpScope::ProgressRead.as_str().to_string(),
                ]
            });

        let access_token_ttl = std::env::var("OAUTH_ACCESS_TOKEN_TTL_SECONDS")
            .ok()
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or(3600);
        let refresh_token_ttl = std::env::var("OAUTH_REFRESH_TOKEN_TTL_SECONDS")
            .ok()
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or(30 * 24 * 60 * 60);
        let allow_dynamic_client_registration =
            std::env::var("OAUTH_ALLOW_DYNAMIC_CLIENT_REGISTRATION")
                .ok()
                .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
                .unwrap_or(false);

        Self {
            issuer: base.clone(),
            authorization_endpoint: format!("{base}/oauth/authorize"),
            token_endpoint: format!("{base}/oauth/token"),
            registration_endpoint: format!("{base}/oauth/register"),
            protected_resource_endpoint: format!("{base}/.well-known/oauth-protected-resource"),
            resource: format!("{base}/mcp"),
            access_token_ttl: Duration::seconds(access_token_ttl.max(300)),
            refresh_token_ttl: Duration::seconds(refresh_token_ttl.max(3600)),
            default_scopes,
            allow_dynamic_client_registration,
        }
    }
}
