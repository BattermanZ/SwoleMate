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

#[derive(Debug, Clone)]
pub struct OAuthRefreshToken {
    pub id: i64,
    pub client_id: String,
    pub user_id: i64,
    pub scopes: Vec<String>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub user_disabled_at: Option<DateTime<Utc>>,
    pub user_must_change_password: bool,
    pub client_disabled_at: Option<DateTime<Utc>>,
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

    pub async fn get_oauth_authorization_code(
        &self,
        code_hash: &str,
    ) -> Result<Option<OAuthAuthorizationCode>, AppError> {
        let pool = self.pool().await;
        let row = sqlx::query(
            r#"
            SELECT id, code_hash, client_id, user_id, redirect_uri, scopes_json, pkce_code_challenge, pkce_method, expires_at, used_at
            FROM oauth_authorization_codes
            WHERE code_hash = ?
            LIMIT 1
            "#,
        )
        .bind(code_hash)
        .fetch_optional(&pool)
        .await
        .map_err(AppError::DatabaseError)?;

        let Some(row) = row else {
            return Ok(None);
        };

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

    pub async fn mark_oauth_authorization_code_used(&self, id: i64) -> Result<bool, AppError> {
        let pool = self.pool().await;
        let result = sqlx::query(
            r#"
            UPDATE oauth_authorization_codes
            SET used_at = CURRENT_TIMESTAMP
            WHERE id = ? AND used_at IS NULL
            "#,
        )
        .bind(id)
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;
        Ok(result.rows_affected() == 1)
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
                oc.disabled_at as client_disabled_at,
                u.id as user_id,
                u.username,
                u.role,
                u.must_change_password,
                u.disabled_at
            FROM oauth_access_tokens oat
            JOIN oauth_clients oc ON oc.client_id = oat.client_id
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
        let client_disabled_at: Option<DateTime<Utc>> = row
            .try_get("client_disabled_at")
            .map_err(AppError::DatabaseError)?;
        if disabled_at.is_some() || client_disabled_at.is_some() {
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
    ) -> Result<Option<OAuthRefreshToken>, AppError> {
        let pool = self.pool().await;
        let row = sqlx::query(
            r#"
            SELECT
                ort.id,
                ort.client_id,
                ort.user_id,
                ort.scopes_json,
                ort.expires_at,
                ort.revoked_at,
                u.disabled_at as user_disabled_at,
                u.must_change_password,
                oc.disabled_at as client_disabled_at
            FROM oauth_refresh_tokens ort
            JOIN users u ON u.id = ort.user_id
            JOIN oauth_clients oc ON oc.client_id = ort.client_id
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
        Ok(Some(OAuthRefreshToken {
            id: row.try_get("id").map_err(AppError::DatabaseError)?,
            client_id: row.try_get("client_id").map_err(AppError::DatabaseError)?,
            user_id: row.try_get("user_id").map_err(AppError::DatabaseError)?,
            scopes: parse_scopes(&scopes)?,
            expires_at: row.try_get("expires_at").map_err(AppError::DatabaseError)?,
            revoked_at: row.try_get("revoked_at").map_err(AppError::DatabaseError)?,
            user_disabled_at: row
                .try_get("user_disabled_at")
                .map_err(AppError::DatabaseError)?,
            user_must_change_password: row
                .try_get::<i64, _>("must_change_password")
                .map_err(AppError::DatabaseError)?
                != 0,
            client_disabled_at: row
                .try_get("client_disabled_at")
                .map_err(AppError::DatabaseError)?,
        }))
    }

    pub async fn revoke_oauth_refresh_token(&self, id: i64) -> Result<bool, AppError> {
        let pool = self.pool().await;
        let result = sqlx::query(
            r#"
            UPDATE oauth_refresh_tokens
            SET revoked_at = CURRENT_TIMESTAMP
            WHERE id = ? AND revoked_at IS NULL
            "#,
        )
        .bind(id)
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn rotate_oauth_refresh_token(
        &self,
        current_refresh_token_id: i64,
        next_access_token_hash: &str,
        next_refresh_token_hash: &str,
        client_id: &str,
        user_id: i64,
        scopes_json: &str,
        access_expires_at: DateTime<Utc>,
        refresh_expires_at: DateTime<Utc>,
    ) -> Result<bool, AppError> {
        let pool = self.pool().await;
        let mut tx = pool.begin().await.map_err(AppError::DatabaseError)?;

        let revoked = sqlx::query(
            r#"
            UPDATE oauth_refresh_tokens
            SET revoked_at = CURRENT_TIMESTAMP
            WHERE id = ? AND revoked_at IS NULL
            "#,
        )
        .bind(current_refresh_token_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::DatabaseError)?
        .rows_affected()
            == 1;

        if !revoked {
            tx.rollback().await.map_err(AppError::DatabaseError)?;
            return Ok(false);
        }

        sqlx::query(
            r#"
            INSERT INTO oauth_access_tokens (token_hash, client_id, user_id, scopes_json, expires_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(next_access_token_hash)
        .bind(client_id)
        .bind(user_id)
        .bind(scopes_json)
        .bind(access_expires_at)
        .execute(&mut *tx)
        .await
        .map_err(AppError::DatabaseError)?;

        sqlx::query(
            r#"
            INSERT INTO oauth_refresh_tokens (token_hash, client_id, user_id, scopes_json, expires_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(next_refresh_token_hash)
        .bind(client_id)
        .bind(user_id)
        .bind(scopes_json)
        .bind(refresh_expires_at)
        .execute(&mut *tx)
        .await
        .map_err(AppError::DatabaseError)?;

        tx.commit().await.map_err(AppError::DatabaseError)?;
        Ok(true)
    }

    pub async fn revoke_oauth_access_token_by_hash(
        &self,
        token_hash: &str,
        client_id: &str,
    ) -> Result<bool, AppError> {
        let pool = self.pool().await;
        let result = sqlx::query(
            r#"
            UPDATE oauth_access_tokens
            SET revoked_at = CURRENT_TIMESTAMP
            WHERE token_hash = ? AND client_id = ? AND revoked_at IS NULL
            "#,
        )
        .bind(token_hash)
        .bind(client_id)
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn revoke_oauth_refresh_token_by_hash(
        &self,
        token_hash: &str,
        client_id: &str,
    ) -> Result<bool, AppError> {
        let pool = self.pool().await;
        let result = sqlx::query(
            r#"
            UPDATE oauth_refresh_tokens
            SET revoked_at = CURRENT_TIMESTAMP
            WHERE token_hash = ? AND client_id = ? AND revoked_at IS NULL
            "#,
        )
        .bind(token_hash)
        .bind(client_id)
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;
        Ok(result.rows_affected() == 1)
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
