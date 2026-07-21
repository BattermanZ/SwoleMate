use super::Database;
use crate::errors::AppError;
use log::error;

/// Server-side idempotency for the two non-idempotent POST creates (workout /
/// exercise). The offline-sync client sends a stable `Idempotency-Key` per
/// offline entity; if a create's HTTP response is lost, the retry carries the
/// same key and we return the already-created resource instead of duplicating it
/// (F-HIGH-3).
///
/// Runtime-checked (non-macro) queries so the change needs no sqlx offline-cache
/// regeneration.
impl Database {
    /// Return the resource id previously created for this (user, kind, key), if any.
    pub async fn lookup_idempotent(
        &self,
        user_id: i64,
        kind: &str,
        key: &str,
    ) -> Result<Option<i64>, AppError> {
        let pool = self.pool().await;
        let existing: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT resource_id
            FROM idempotency_keys
            WHERE user_id = ? AND request_kind = ? AND idempotency_key = ?
            "#,
        )
        .bind(user_id)
        .bind(kind)
        .bind(key)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to look up idempotency key: {}", e);
            AppError::DatabaseError(e)
        })?;
        Ok(existing)
    }

    /// Record that `resource_id` was created for this (user, kind, key). Returns the
    /// authoritative resource id: `resource_id` when we won, or the id stored by a
    /// concurrent request that inserted first (so the caller can drop its duplicate).
    pub async fn record_idempotent(
        &self,
        user_id: i64,
        kind: &str,
        key: &str,
        resource_id: i64,
    ) -> Result<i64, AppError> {
        let pool = self.pool().await;
        let result = sqlx::query(
            r#"
            INSERT OR IGNORE INTO idempotency_keys (user_id, request_kind, idempotency_key, resource_id)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(user_id)
        .bind(kind)
        .bind(key)
        .bind(resource_id)
        .execute(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to record idempotency key: {}", e);
            AppError::DatabaseError(e)
        })?;

        if result.rows_affected() == 1 {
            return Ok(resource_id);
        }

        // A concurrent request won the race for this key — return its resource id.
        let existing: i64 = sqlx::query_scalar(
            r#"
            SELECT resource_id
            FROM idempotency_keys
            WHERE user_id = ? AND request_kind = ? AND idempotency_key = ?
            "#,
        )
        .bind(user_id)
        .bind(kind)
        .bind(key)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to resolve idempotency race: {}", e);
            AppError::DatabaseError(e)
        })?;
        Ok(existing)
    }
}

pub const KIND_WORKOUT: &str = "workout";
pub const KIND_EXERCISE: &str = "exercise";
