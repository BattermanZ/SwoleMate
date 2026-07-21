use super::Database;
use crate::errors::AppError;
use chrono::{DateTime, Utc};
use sqlx::Row;

/// Expiry for a rotated MCP token: re-apply the original token's lifetime
/// (expires_at - created_at) from `now`, so a token rotated late in its window
/// still gets its full duration back instead of inheriting a nearly-elapsed
/// absolute expiry (B-LOW-9). A non-expiring token (None) stays non-expiring.
fn rotated_expiry(
    original_expiry: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Option<DateTime<Utc>> {
    original_expiry.map(|expiry| now + (expiry - created_at))
}

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

    /// Revoke every live MCP token for a user. Used when a password changes so a
    /// leaked bearer token cannot outlive the credential rotation (B-MED-4).
    pub async fn revoke_all_mcp_tokens_for_user(&self, user_id: i64) -> Result<(), AppError> {
        let pool = self.pool().await;
        sqlx::query(
            r#"
            UPDATE mcp_tokens
            SET revoked_at = CURRENT_TIMESTAMP
            WHERE user_id = ? AND revoked_at IS NULL
            "#,
        )
        .bind(user_id)
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;
        Ok(())
    }

    pub async fn rotate_mcp_token_for_user(
        &self,
        token_id: i64,
        user_id: i64,
        token_hash: &str,
    ) -> Result<Option<(i64, String, Vec<String>, Option<DateTime<Utc>>)>, AppError> {
        let pool = self.pool().await;
        let mut tx = pool.begin().await.map_err(AppError::DatabaseError)?;

        let existing = sqlx::query(
            r#"
            SELECT name, scopes_json, expires_at, created_at
            FROM mcp_tokens
            WHERE id = ? AND user_id = ? AND revoked_at IS NULL
            LIMIT 1
            "#,
        )
        .bind(token_id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(AppError::DatabaseError)?;

        let Some(existing) = existing else {
            tx.rollback().await.map_err(AppError::DatabaseError)?;
            return Ok(None);
        };

        let name: String = existing.try_get("name").map_err(AppError::DatabaseError)?;
        let scopes_json: String = existing
            .try_get("scopes_json")
            .map_err(AppError::DatabaseError)?;
        let scopes = parse_scopes(&scopes_json)?;
        let expires_at: Option<DateTime<Utc>> = existing
            .try_get("expires_at")
            .map_err(AppError::DatabaseError)?;
        let created_at: DateTime<Utc> = existing
            .try_get("created_at")
            .map_err(AppError::DatabaseError)?;

        // Rotation hands out a durable replacement credential, so re-apply the
        // original token's lifetime from *now* rather than copying its (possibly
        // nearly-elapsed) absolute expiry — otherwise a token rotated late in its
        // window would expire almost immediately (B-LOW-9). A non-expiring token
        // (NULL expiry) stays non-expiring.
        let new_expires_at = rotated_expiry(expires_at, created_at, Utc::now());

        let result = sqlx::query(
            r#"
            INSERT INTO mcp_tokens (user_id, name, token_hash, scopes_json, expires_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(user_id)
        .bind(&name)
        .bind(token_hash)
        .bind(&scopes_json)
        .bind(new_expires_at)
        .execute(&mut *tx)
        .await
        .map_err(AppError::DatabaseError)?;

        sqlx::query(
            r#"
            UPDATE mcp_tokens
            SET revoked_at = CURRENT_TIMESTAMP
            WHERE id = ? AND user_id = ? AND revoked_at IS NULL
            "#,
        )
        .bind(token_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::DatabaseError)?;

        tx.commit().await.map_err(AppError::DatabaseError)?;

        Ok(Some((
            result.last_insert_rowid(),
            name,
            scopes,
            new_expires_at,
        )))
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

#[cfg(test)]
mod tests {
    use super::rotated_expiry;
    use chrono::{Duration, Utc};

    #[test]
    fn rotation_reapplies_full_lifetime_from_now() {
        // A 30-day token created 29 days ago (expiry 1 day out). Rotating it must
        // grant a fresh ~30-day window from now, NOT the near-elapsed 1 day.
        let now = Utc::now();
        let created_at = now - Duration::days(29);
        let original_expiry = Some(created_at + Duration::days(30)); // ~1 day from now

        let rotated = rotated_expiry(original_expiry, created_at, now).unwrap();
        let remaining = rotated - now;
        assert!(
            (remaining - Duration::days(30)).num_seconds().abs() < 5,
            "rotated token should have ~30 days left, got {remaining}"
        );
    }

    #[test]
    fn rotation_keeps_non_expiring_tokens_non_expiring() {
        assert!(rotated_expiry(None, Utc::now(), Utc::now()).is_none());
    }
}
