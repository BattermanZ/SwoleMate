use super::Database;
use crate::auth::{normalize_username, Role};
use crate::errors::AppError;
use chrono::{DateTime, Utc};
use log::{debug, error};

#[derive(Debug, Clone)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub role: Role,
    pub disabled_at: Option<DateTime<Utc>>,
    pub failed_login_count: i64,
    pub locked_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct SessionRow {
    pub id: i64,
    pub user_id: i64,
    pub session_hash: String,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub rotated_from_session_id: Option<i64>,
}

impl Database {
    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<UserRow>, AppError> {
        let pool = self.pool().await;
        let username = normalize_username(username);
        let row = sqlx::query!(
            r#"
            SELECT
                id as "id!: i64",
                username,
                password_hash,
                role as "role!: String",
                disabled_at as "disabled_at: DateTime<Utc>",
                failed_login_count as "failed_login_count!: i64",
                locked_until as "locked_until: DateTime<Utc>"
            FROM users
            WHERE LOWER(username) = LOWER(?)
            LIMIT 1
            "#,
            username
        )
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch user by username: {}", e);
            AppError::DatabaseError(e)
        })?;

        Ok(row.map(|r| UserRow {
            id: r.id,
            username: r.username,
            password_hash: r.password_hash,
            role: if r.role == "admin" { Role::Admin } else { Role::User },
            disabled_at: r.disabled_at,
            failed_login_count: r.failed_login_count,
            locked_until: r.locked_until,
        }))
    }

    pub async fn get_user_by_id(&self, user_id: i64) -> Result<Option<UserRow>, AppError> {
        let pool = self.pool().await;
        let row = sqlx::query!(
            r#"
            SELECT
                id as "id!: i64",
                username,
                password_hash,
                role as "role!: String",
                disabled_at as "disabled_at: DateTime<Utc>",
                failed_login_count as "failed_login_count!: i64",
                locked_until as "locked_until: DateTime<Utc>"
            FROM users
            WHERE id = ?
            LIMIT 1
            "#,
            user_id
        )
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            error!(target: "database", "Failed to fetch user by id: {}", e);
            AppError::DatabaseError(e)
        })?;

        Ok(row.map(|r| UserRow {
            id: r.id,
            username: r.username,
            password_hash: r.password_hash,
            role: if r.role == "admin" { Role::Admin } else { Role::User },
            disabled_at: r.disabled_at,
            failed_login_count: r.failed_login_count,
            locked_until: r.locked_until,
        }))
    }

    pub async fn create_user(
        &self,
        username: &str,
        password_hash: &str,
        role: Role,
    ) -> Result<i64, AppError> {
        let pool = self.pool().await;
        let username = normalize_username(username);
        let role = if role.is_admin() { "admin" } else { "user" };
        let row = sqlx::query!(
            r#"
            INSERT INTO users (username, password_hash, role)
            VALUES (?, ?, ?)
            RETURNING id as "id!: i64"
            "#,
            username,
            password_hash,
            role
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
        Ok(row.id)
    }

    pub async fn list_users(&self) -> Result<Vec<(i64, String, Role, Option<DateTime<Utc>>)>, AppError>
    {
        let pool = self.pool().await;
        let rows = sqlx::query!(
            r#"
            SELECT id as "id!: i64", username, role as "role!: String", disabled_at as "disabled_at: DateTime<Utc>"
            FROM users
            ORDER BY id ASC
            "#
        )
        .fetch_all(&pool)
        .await
        .map_err(AppError::DatabaseError)?;
        Ok(rows
            .into_iter()
            .map(|r| {
                (
                    r.id,
                    r.username,
                    if r.role == "admin" { Role::Admin } else { Role::User },
                    r.disabled_at,
                )
            })
            .collect())
    }

    pub async fn disable_user(&self, user_id: i64) -> Result<(), AppError> {
        let pool = self.pool().await;
        sqlx::query!(
            r#"
            UPDATE users
            SET disabled_at = COALESCE(disabled_at, CURRENT_TIMESTAMP)
            WHERE id = ?
            "#,
            user_id
        )
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;
        Ok(())
    }

    pub async fn update_password_hash(&self, user_id: i64, password_hash: &str) -> Result<(), AppError> {
        let pool = self.pool().await;
        sqlx::query!(
            r#"
            UPDATE users
            SET password_hash = ?, failed_login_count = 0, locked_until = NULL
            WHERE id = ?
            "#,
            password_hash,
            user_id
        )
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;
        Ok(())
    }

    pub async fn record_failed_login(&self, user_id: i64) -> Result<(), AppError> {
        let pool = self.pool().await;

        // Lockout policy: 5 failed attempts -> 5 minutes lock.
        sqlx::query!(
            r#"
            UPDATE users
            SET failed_login_count = failed_login_count + 1,
                locked_until = CASE
                    WHEN failed_login_count + 1 >= 5 THEN datetime('now', '+5 minutes')
                    ELSE locked_until
                END
            WHERE id = ?
            "#,
            user_id
        )
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;
        Ok(())
    }

    pub async fn reset_login_failures(&self, user_id: i64) -> Result<(), AppError> {
        let pool = self.pool().await;
        sqlx::query!(
            r#"
            UPDATE users
            SET failed_login_count = 0, locked_until = NULL
            WHERE id = ?
            "#,
            user_id
        )
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;
        Ok(())
    }

    pub async fn create_session(
        &self,
        user_id: i64,
        session_hash: &str,
        expires_at: DateTime<Utc>,
        rotated_from_session_id: Option<i64>,
        user_agent: Option<String>,
        ip: Option<String>,
    ) -> Result<i64, AppError> {
        let pool = self.pool().await;
        let row = sqlx::query!(
            r#"
            INSERT INTO sessions (user_id, session_hash, expires_at, rotated_from_session_id, user_agent, ip)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id as "id!: i64"
            "#,
            user_id,
            session_hash,
            expires_at,
            rotated_from_session_id,
            user_agent,
            ip
        )
        .fetch_one(&pool)
        .await
        .map_err(AppError::DatabaseError)?;
        Ok(row.id)
    }

    pub async fn revoke_session_by_hash(&self, session_hash: &str) -> Result<(), AppError> {
        let pool = self.pool().await;
        sqlx::query!(
            r#"
            UPDATE sessions
            SET revoked_at = COALESCE(revoked_at, CURRENT_TIMESTAMP)
            WHERE session_hash = ?
            "#,
            session_hash
        )
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;
        Ok(())
    }

    pub async fn revoke_all_sessions_for_user(&self, user_id: i64) -> Result<(), AppError> {
        let pool = self.pool().await;
        sqlx::query!(
            r#"
            UPDATE sessions
            SET revoked_at = COALESCE(revoked_at, CURRENT_TIMESTAMP)
            WHERE user_id = ?
            "#,
            user_id
        )
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;
        Ok(())
    }

    pub async fn get_session_and_user_by_hash(
        &self,
        session_hash: &str,
    ) -> Result<Option<(SessionRow, UserRow)>, AppError> {
        let pool = self.pool().await;
        let row = sqlx::query!(
            r#"
            SELECT
                s.id as "session_id!: i64",
                s.user_id as "user_id!: i64",
                s.session_hash as "session_hash!: String",
                s.expires_at as "expires_at: DateTime<Utc>",
                s.revoked_at as "revoked_at: DateTime<Utc>",
                s.rotated_from_session_id as "rotated_from_session_id: i64",
                u.username as "username!: String",
                u.password_hash as "password_hash!: String",
                u.role as "role!: String",
                u.disabled_at as "disabled_at: DateTime<Utc>",
                u.failed_login_count as "failed_login_count!: i64",
                u.locked_until as "locked_until: DateTime<Utc>"
            FROM sessions s
            JOIN users u ON u.id = s.user_id
            WHERE s.session_hash = ?
            LIMIT 1
            "#,
            session_hash
        )
        .fetch_optional(&pool)
        .await
        .map_err(AppError::DatabaseError)?;

        let Some(r) = row else {
            return Ok(None);
        };

        let session = SessionRow {
            id: r.session_id,
            user_id: r.user_id,
            session_hash: r.session_hash,
            expires_at: r.expires_at,
            revoked_at: r.revoked_at,
            rotated_from_session_id: r.rotated_from_session_id,
        };

        let user = UserRow {
            id: r.user_id,
            username: r.username,
            password_hash: r.password_hash,
            role: if r.role == "admin" { Role::Admin } else { Role::User },
            disabled_at: r.disabled_at,
            failed_login_count: r.failed_login_count,
            locked_until: r.locked_until,
        };

        Ok(Some((session, user)))
    }

    pub async fn touch_session(&self, session_id: i64) -> Result<(), AppError> {
        let pool = self.pool().await;
        sqlx::query!(
            r#"
            UPDATE sessions
            SET last_seen_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            session_id
        )
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;
        Ok(())
    }

    pub async fn rotate_session(
        &self,
        old_session_id: i64,
        old_session_hash: &str,
        user_id: i64,
        new_session_hash: &str,
        new_expires_at: DateTime<Utc>,
        user_agent: Option<String>,
        ip: Option<String>,
    ) -> Result<(), AppError> {
        debug!(target: "database", "Rotating session {} for user {}", old_session_id, user_id);
        let pool = self.pool().await;
        let mut tx = pool.begin().await.map_err(AppError::DatabaseError)?;

        sqlx::query!(
            r#"
            UPDATE sessions
            SET revoked_at = COALESCE(revoked_at, CURRENT_TIMESTAMP)
            WHERE id = ? AND session_hash = ?
            "#,
            old_session_id,
            old_session_hash
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::DatabaseError)?;

        sqlx::query!(
            r#"
            INSERT INTO sessions (user_id, session_hash, expires_at, rotated_from_session_id, user_agent, ip)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            user_id,
            new_session_hash,
            new_expires_at,
            old_session_id,
            user_agent,
            ip
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::DatabaseError)?;

        tx.commit().await.map_err(AppError::DatabaseError)?;
        Ok(())
    }
}
