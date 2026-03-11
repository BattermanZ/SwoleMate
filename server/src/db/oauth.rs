use super::Database;
use crate::auth::{hash_session_token, AuthUser, Role};
use crate::errors::AppError;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::Row;

#[derive(Debug, Clone)]
pub struct OAuthClient {
    pub client_id: String,
    pub client_secret_hash: Option<String>,
    pub client_name: String,
    pub redirect_uris: Vec<String>,
    pub scopes: Vec<String>,
    pub disabled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct OAuthAuthorizationCode {
    pub id: i64,
    pub code_hash: String,
    pub client_id: String,
    pub user_id: i64,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub pkce_code_challenge: Option<String>,
    pub pkce_method: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct OAuthAccessToken {
    pub id: i64,
    pub client_id: String,
    pub user: AuthUser,
    pub scopes: Vec<String>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

fn parse_scopes(raw: &str) -> Result<Vec<String>, AppError> {
    serde_json::from_str::<Vec<String>>(raw)
        .map_err(|e| AppError::InternalError(format!("invalid stored scopes json: {e}")))
}

fn parse_redirect_uris(raw: &str) -> Result<Vec<String>, AppError> {
    serde_json::from_str::<Vec<String>>(raw)
        .map_err(|e| AppError::InternalError(format!("invalid stored redirect_uris json: {e}")))
}

impl Database {
    pub async fn create_oauth_client(
        &self,
        client_id: &str,
        client_secret_hash: Option<&str>,
        client_name: &str,
        redirect_uris_json: &str,
        scopes_json: &str,
    ) -> Result<(), AppError> {
        let pool = self.pool().await;
        sqlx::query(
            r#"
            INSERT INTO oauth_clients (client_id, client_secret_hash, client_name, redirect_uris_json, scopes_json)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(client_id)
        .bind(client_secret_hash)
        .bind(client_name)
        .bind(redirect_uris_json)
        .bind(scopes_json)
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;
        Ok(())
    }

    pub async fn get_oauth_client(&self, client_id: &str) -> Result<Option<OAuthClient>, AppError> {
        let pool = self.pool().await;
        let row = sqlx::query(
            r#"
            SELECT client_id, client_secret_hash, client_name, redirect_uris_json, scopes_json, disabled_at
            FROM oauth_clients
            WHERE client_id = ?
            LIMIT 1
            "#,
        )
        .bind(client_id)
        .fetch_optional(&pool)
        .await
        .map_err(AppError::DatabaseError)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let redirect_uris: String = row
            .try_get("redirect_uris_json")
            .map_err(AppError::DatabaseError)?;
        let scopes: String = row
            .try_get("scopes_json")
            .map_err(AppError::DatabaseError)?;

        Ok(Some(OAuthClient {
            client_id: row.try_get("client_id").map_err(AppError::DatabaseError)?,
            client_secret_hash: row
                .try_get("client_secret_hash")
                .map_err(AppError::DatabaseError)?,
            client_name: row
                .try_get("client_name")
                .map_err(AppError::DatabaseError)?,
            redirect_uris: parse_redirect_uris(&redirect_uris)?,
            scopes: parse_scopes(&scopes)?,
            disabled_at: row
                .try_get("disabled_at")
                .map_err(AppError::DatabaseError)?,
        }))
    }

    pub async fn create_oauth_authorization_code(
        &self,
        code_hash: &str,
        client_id: &str,
        user_id: i64,
        redirect_uri: &str,
        scopes_json: &str,
        pkce_code_challenge: Option<&str>,
        pkce_method: Option<&str>,
        expires_at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let pool = self.pool().await;
        sqlx::query(
            r#"
            INSERT INTO oauth_authorization_codes (
                code_hash, client_id, user_id, redirect_uri, scopes_json, pkce_code_challenge, pkce_method, expires_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(code_hash)
        .bind(client_id)
        .bind(user_id)
        .bind(redirect_uri)
        .bind(scopes_json)
        .bind(pkce_code_challenge)
        .bind(pkce_method)
        .bind(expires_at)
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;
        Ok(())
    }

    pub async fn consume_oauth_authorization_code(
        &self,
        code_hash: &str,
    ) -> Result<Option<OAuthAuthorizationCode>, AppError> {
        let pool = self.pool().await;
        let mut tx = pool.begin().await.map_err(AppError::DatabaseError)?;
        let row = sqlx::query(
            r#"
            SELECT id, code_hash, client_id, user_id, redirect_uri, scopes_json, pkce_code_challenge, pkce_method, expires_at, used_at
            FROM oauth_authorization_codes
            WHERE code_hash = ?
            LIMIT 1
            "#,
        )
        .bind(code_hash)
        .fetch_optional(&mut *tx)
        .await
        .map_err(AppError::DatabaseError)?;

        let Some(row) = row else {
            tx.commit().await.map_err(AppError::DatabaseError)?;
            return Ok(None);
        };

        sqlx::query(
            r#"
            UPDATE oauth_authorization_codes
            SET used_at = COALESCE(used_at, CURRENT_TIMESTAMP)
            WHERE id = ?
            "#,
        )
        .bind(
            row.try_get::<i64, _>("id")
                .map_err(AppError::DatabaseError)?,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::DatabaseError)?;

        tx.commit().await.map_err(AppError::DatabaseError)?;

        let scopes: String = row
            .try_get("scopes_json")
            .map_err(AppError::DatabaseError)?;
        Ok(Some(OAuthAuthorizationCode {
            id: row.try_get("id").map_err(AppError::DatabaseError)?,
            code_hash: row.try_get("code_hash").map_err(AppError::DatabaseError)?,
            client_id: row.try_get("client_id").map_err(AppError::DatabaseError)?,
            user_id: row.try_get("user_id").map_err(AppError::DatabaseError)?,
            redirect_uri: row
                .try_get("redirect_uri")
                .map_err(AppError::DatabaseError)?,
            scopes: parse_scopes(&scopes)?,
            pkce_code_challenge: row
                .try_get("pkce_code_challenge")
                .map_err(AppError::DatabaseError)?,
            pkce_method: row
                .try_get("pkce_method")
                .map_err(AppError::DatabaseError)?,
            expires_at: row.try_get("expires_at").map_err(AppError::DatabaseError)?,
            used_at: row.try_get("used_at").map_err(AppError::DatabaseError)?,
        }))
    }

    pub async fn create_oauth_access_token(
        &self,
        token_hash: &str,
        client_id: &str,
        user_id: i64,
        scopes_json: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let pool = self.pool().await;
        sqlx::query(
            r#"
            INSERT INTO oauth_access_tokens (token_hash, client_id, user_id, scopes_json, expires_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(token_hash)
        .bind(client_id)
        .bind(user_id)
        .bind(scopes_json)
        .bind(expires_at)
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;
        Ok(())
    }

    pub async fn create_oauth_refresh_token(
        &self,
        token_hash: &str,
        client_id: &str,
        user_id: i64,
        scopes_json: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let pool = self.pool().await;
        sqlx::query(
            r#"
            INSERT INTO oauth_refresh_tokens (token_hash, client_id, user_id, scopes_json, expires_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(token_hash)
        .bind(client_id)
        .bind(user_id)
        .bind(scopes_json)
        .bind(expires_at)
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;
        Ok(())
    }

    pub async fn get_oauth_access_token_by_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<OAuthAccessToken>, AppError> {
        let pool = self.pool().await;
        let row = sqlx::query(
            r#"
            SELECT
                oat.id,
                oat.client_id,
                oat.scopes_json,
                oat.expires_at,
                oat.revoked_at,
                u.id as user_id,
                u.username,
                u.role,
                u.must_change_password,
                u.disabled_at
            FROM oauth_access_tokens oat
            JOIN users u ON u.id = oat.user_id
            WHERE oat.token_hash = ?
            LIMIT 1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&pool)
        .await
        .map_err(AppError::DatabaseError)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let disabled_at: Option<DateTime<Utc>> = row
            .try_get("disabled_at")
            .map_err(AppError::DatabaseError)?;
        if disabled_at.is_some() {
            return Ok(None);
        }

        let scopes: String = row
            .try_get("scopes_json")
            .map_err(AppError::DatabaseError)?;
        let role: String = row.try_get("role").map_err(AppError::DatabaseError)?;

        Ok(Some(OAuthAccessToken {
            id: row.try_get("id").map_err(AppError::DatabaseError)?,
            client_id: row.try_get("client_id").map_err(AppError::DatabaseError)?,
            user: AuthUser {
                id: row.try_get("user_id").map_err(AppError::DatabaseError)?,
                username: row.try_get("username").map_err(AppError::DatabaseError)?,
                role: if role == "admin" {
                    Role::Admin
                } else {
                    Role::User
                },
                must_change_password: row
                    .try_get::<i64, _>("must_change_password")
                    .map_err(AppError::DatabaseError)?
                    != 0,
            },
            scopes: parse_scopes(&scopes)?,
            expires_at: row.try_get("expires_at").map_err(AppError::DatabaseError)?,
            revoked_at: row.try_get("revoked_at").map_err(AppError::DatabaseError)?,
        }))
    }

    pub async fn get_oauth_refresh_token(
        &self,
        token_hash: &str,
    ) -> Result<
        Option<(
            String,
            i64,
            Vec<String>,
            DateTime<Utc>,
            Option<DateTime<Utc>>,
        )>,
        AppError,
    > {
        let pool = self.pool().await;
        let row = sqlx::query(
            r#"
            SELECT client_id, user_id, scopes_json, expires_at, revoked_at
            FROM oauth_refresh_tokens
            WHERE token_hash = ?
            LIMIT 1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&pool)
        .await
        .map_err(AppError::DatabaseError)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let scopes: String = row
            .try_get("scopes_json")
            .map_err(AppError::DatabaseError)?;
        Ok(Some((
            row.try_get("client_id").map_err(AppError::DatabaseError)?,
            row.try_get("user_id").map_err(AppError::DatabaseError)?,
            parse_scopes(&scopes)?,
            row.try_get("expires_at").map_err(AppError::DatabaseError)?,
            row.try_get("revoked_at").map_err(AppError::DatabaseError)?,
        )))
    }

    pub async fn store_oauth_consent(
        &self,
        user_id: i64,
        client_id: &str,
        scopes_json: &str,
    ) -> Result<(), AppError> {
        let pool = self.pool().await;
        sqlx::query(
            r#"
            INSERT INTO oauth_consents (user_id, client_id, scopes_json, granted_at, revoked_at)
            VALUES (?, ?, ?, CURRENT_TIMESTAMP, NULL)
            "#,
        )
        .bind(user_id)
        .bind(client_id)
        .bind(scopes_json)
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;
        Ok(())
    }

    pub async fn write_mcp_audit_log(
        &self,
        user_id: Option<i64>,
        client_id: Option<&str>,
        tool_name: &str,
        success: bool,
        error_code: Option<&str>,
        input_summary_json: Option<&Value>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<(), AppError> {
        let pool = self.pool().await;
        let input_summary = input_summary_json.map(|value| value.to_string());
        sqlx::query(
            r#"
            INSERT INTO mcp_audit_log (
                user_id, client_id, tool_name, success, error_code, input_summary_json, ip_address, user_agent
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(user_id)
        .bind(client_id)
        .bind(tool_name)
        .bind(if success { 1 } else { 0 })
        .bind(error_code)
        .bind(input_summary)
        .bind(ip_address)
        .bind(user_agent)
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;
        Ok(())
    }

    pub async fn insert_test_access_token(
        &self,
        raw_token: &str,
        client_id: &str,
        user_id: i64,
        scopes: &[String],
        expires_at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let hash = hash_session_token(raw_token);
        let scopes_json = serde_json::to_string(scopes)
            .map_err(|e| AppError::InternalError(format!("failed to encode scopes: {e}")))?;
        self.create_oauth_access_token(&hash, client_id, user_id, &scopes_json, expires_at)
            .await
    }
}
