use super::Database;
use crate::errors::AppError;
use chrono::{DateTime, Utc};
use sqlx::Row;

#[derive(Debug, Clone)]
pub struct McpTokenRow {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub user_disabled_at: Option<DateTime<Utc>>,
    pub user_must_change_password: bool,
}

fn parse_scopes(raw: &str) -> Result<Vec<String>, AppError> {
    serde_json::from_str::<Vec<String>>(raw)
        .map_err(|e| AppError::InternalError(format!("invalid stored mcp scopes json: {e}")))
}

impl Database {
    pub async fn create_mcp_token(
        &self,
        user_id: i64,
        name: &str,
        token_hash: &str,
        scopes_json: &str,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<i64, AppError> {
        let pool = self.pool().await;
        let result = sqlx::query(
            r#"
            INSERT INTO mcp_tokens (user_id, name, token_hash, scopes_json, expires_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(user_id)
        .bind(name)
        .bind(token_hash)
        .bind(scopes_json)
        .bind(expires_at)
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;
        Ok(result.last_insert_rowid())
    }

    pub async fn list_mcp_tokens_for_user(
        &self,
        user_id: i64,
    ) -> Result<Vec<McpTokenRow>, AppError> {
        let pool = self.pool().await;
        let rows = sqlx::query(
            r#"
            SELECT
                mt.id,
                mt.user_id,
                mt.name,
                mt.scopes_json,
                mt.expires_at,
                mt.revoked_at,
                mt.last_used_at,
                mt.created_at,
                u.disabled_at as user_disabled_at,
                u.must_change_password
            FROM mcp_tokens mt
            JOIN users u ON u.id = mt.user_id
            WHERE mt.user_id = ?
            ORDER BY mt.created_at DESC, mt.id DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&pool)
        .await
        .map_err(AppError::DatabaseError)?;

        rows.into_iter()
            .map(Self::mcp_token_from_row)
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn get_mcp_token_by_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<McpTokenRow>, AppError> {
        let pool = self.pool().await;
        let row = sqlx::query(
            r#"
            SELECT
                mt.id,
                mt.user_id,
                mt.name,
                mt.scopes_json,
                mt.expires_at,
                mt.revoked_at,
                mt.last_used_at,
                mt.created_at,
                u.disabled_at as user_disabled_at,
                u.must_change_password
            FROM mcp_tokens mt
            JOIN users u ON u.id = mt.user_id
            WHERE mt.token_hash = ?
            LIMIT 1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&pool)
        .await
        .map_err(AppError::DatabaseError)?;

        row.map(Self::mcp_token_from_row).transpose()
    }

    pub async fn revoke_mcp_token_for_user(
        &self,
        token_id: i64,
        user_id: i64,
    ) -> Result<bool, AppError> {
        let pool = self.pool().await;
        let result = sqlx::query(
            r#"
            UPDATE mcp_tokens
            SET revoked_at = CURRENT_TIMESTAMP
            WHERE id = ? AND user_id = ? AND revoked_at IS NULL
            "#,
        )
        .bind(token_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn touch_mcp_token_last_used(&self, token_id: i64) -> Result<(), AppError> {
        let pool = self.pool().await;
        sqlx::query(
            r#"
            UPDATE mcp_tokens
            SET last_used_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(token_id)
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;
        Ok(())
    }

    fn mcp_token_from_row(row: sqlx::sqlite::SqliteRow) -> Result<McpTokenRow, AppError> {
        let scopes: String = row
            .try_get("scopes_json")
            .map_err(AppError::DatabaseError)?;

        Ok(McpTokenRow {
            id: row.try_get("id").map_err(AppError::DatabaseError)?,
            user_id: row.try_get("user_id").map_err(AppError::DatabaseError)?,
            name: row.try_get("name").map_err(AppError::DatabaseError)?,
            scopes: parse_scopes(&scopes)?,
            expires_at: row.try_get("expires_at").map_err(AppError::DatabaseError)?,
            revoked_at: row.try_get("revoked_at").map_err(AppError::DatabaseError)?,
            last_used_at: row
                .try_get("last_used_at")
                .map_err(AppError::DatabaseError)?,
            created_at: row.try_get("created_at").map_err(AppError::DatabaseError)?,
            user_disabled_at: row
                .try_get("user_disabled_at")
                .map_err(AppError::DatabaseError)?,
            user_must_change_password: row
                .try_get::<i64, _>("must_change_password")
                .map_err(AppError::DatabaseError)?
                != 0,
        })
    }
}
