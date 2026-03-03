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

export function scopedKey(base: string): string {
	const userId = getActiveUserId();
	if (!userId) return base;
	return `u${userId}:${base}`;
}
