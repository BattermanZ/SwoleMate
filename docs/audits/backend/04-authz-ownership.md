# Authorization / resource ownership (IDOR & privilege escalation)

**Summary:** 1 confirmed (1 medium), 0 refuted.

## Confirmed findings

### MEDIUM (correctness): disable_user lacks the last-admin guard that delete_user has, and no re-enable route exists — an admin can irreversibly brick all admin capability

- **Attack / trigger:**
  An authenticated admin issues `POST /api/admin/users/{id}/disable` targeting the only admin account (which may be their own id). The request succeeds, the admin's sessions are revoked, and every subsequent admin-gated call (`/api/admin/*`, `/api/backups*`) returns 401/403 for everyone. No API path can restore an admin.

- **Location:**
  `server/src/routes/admin.rs:68–80` (disable_user); compared against `admin.rs:98–120` (delete_user); also `session_auth.rs:202–208` (SessionAuth rejection), `routes.rs:612–616` (admin endpoints)

- **What happens:**
  `disable_user` accepts any authenticated admin and an `id` parameter, calls `db.disable_user(*id)` and `revoke_all_sessions_for_user(*id)` unconditionally with no admin-count check. Once the last admin is disabled, `SessionAuth` rejects any session for that user (disabled_at is set), `AdminUser` extractor fails because no live admin session exists, and all admin-gated routes return 401/403. The admin API exposes only list/create/disable/reset-password/delete users — there is NO re-enable endpoint — so this state is not recoverable through the API and requires direct database surgery.

- **Why:**
  It is a real, user-impacting correctness/availability gap with an obvious intended-but-missing guard: the sibling `delete_user` path guards exactly this condition (`role.is_admin() && count_active_admins() <= 1 => Conflict`), showing the invariant "at least one active admin must always exist" was meant to hold. It is not a privilege escalation (only a trusted admin can trigger it) and is not remotely exploitable, so it is not high/critical, but the irreversibility via API elevates it above a trivial footgun.

- **Fix sketch:**
  Before disabling, if the target user's role is admin, call `db.count_active_admins()` and reject with `Conflict` when it is ≤ 1 (mirror the guard in `delete_user`). Optionally also refuse to disable one's own account, and/or add an `enable_user` route so the state is recoverable.

## Refuted (not real / already handled)

(None)
