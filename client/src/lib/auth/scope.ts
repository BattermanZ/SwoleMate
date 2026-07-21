const ACTIVE_USER_ID_KEY = 'auth.activeUserId';

function isBrowser(): boolean {
	return typeof window !== 'undefined';
}

export function getActiveUserId(): string | null {
	if (!isBrowser()) return null;
	try {
		const value = localStorage.getItem(ACTIVE_USER_ID_KEY);
		return value && value.trim().length > 0 ? value : null;
	} catch {
		return null;
	}
}

export function setActiveUserId(userId: number | null): void {
	if (!isBrowser()) return;
	try {
		if (userId == null) {
			localStorage.removeItem(ACTIVE_USER_ID_KEY);
			return;
		}
		localStorage.setItem(ACTIVE_USER_ID_KEY, String(userId));
	} catch {
		// ignore
	}
}

/** Whether a concrete active-user scope is currently known. */
export function hasActiveUserScope(): boolean {
	return getActiveUserId() !== null;
}

export function scopedKey(base: string): string {
	const userId = getActiveUserId();
	// When the active user id is unknown (cold-start race, or localStorage
	// unreadable) use an explicit `anon:` namespace rather than the bare base key.
	// This keeps such keys from ever colliding with a real user's `u<id>:` reads,
	// and — combined with saveOfflineSession refusing to write without a scope —
	// stops an in-progress offline session from being orphaned or exposed to
	// another account on a shared device (F-LOW-2).
	if (!userId) return `anon:${base}`;
	return `u${userId}:${base}`;
}
