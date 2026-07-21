import { browser } from '$app/environment';
import {
	authChangePassword,
	authLogin,
	authLogout,
	authMe,
	isApiError,
	setUnauthorizedHandler,
	type PublicUser
} from '$lib/api';
import { kvDelete, kvListKeys } from '$lib/offline/storage';
import { getActiveUserId, setActiveUserId } from './scope';
import { writable } from 'svelte/store';

type AuthStatus = 'unknown' | 'authenticated' | 'unauthenticated';

export type AuthState = {
	status: AuthStatus;
	user: PublicUser | null;
	offline: boolean;
};

const STORAGE_KEY = 'auth.lastUser';
const CACHE_STORAGE_PREFIX = 'swolemate-cache';

type ClearOptions = {
	// When true, in-progress / pending_sync workout session records are preserved.
	// These records hold logged sets that have not yet reached the server, so a
	// routine 401 (session TTL lapse, redeploy) or a user switch must NOT destroy
	// them. Only read-only cache + session metadata is cleared. See audit
	// docs/audits/frontend/05-auth-scope-guard.md (F-CRIT-1, F-HIGH-4).
	preserveWorkoutSessions?: boolean;
	// When set, only keys scoped to this user id (`u{id}:` prefix) are cleared, so a
	// switch to another account never wipes the departing user's data — nor a third
	// user's data on a shared device. When null, keys for all users are cleared.
	userId?: string | null;
};

// Normalise a raw localStorage key to its plain kv key. kv-backed entries are
// stored as `swolemate:<plainKey>`; other keys (legacy pointers, cache) pass
// through unchanged.
function stripLsPrefix(rawKey: string): string {
	return rawKey.startsWith('swolemate:') ? rawKey.slice('swolemate:'.length) : rawKey;
}

async function clearClientSensitiveData(opts: ClearOptions = {}): Promise<void> {
	if (!browser) return;

	const { preserveWorkoutSessions = false, userId = null } = opts;

	const inUserScope = (plainKey: string): boolean => {
		if (!userId) return true;
		return plainKey.startsWith(`u${userId}:`);
	};

	// Decide whether a (plain) key should be removed under the current options.
	const shouldRemove = (plainKey: string): boolean => {
		// Read-only cached API responses — always safe to drop, but respect scope.
		if (plainKey.startsWith('swolemate-cache:') || plainKey.includes(':swolemate-cache:')) {
			return inUserScope(plainKey);
		}
		// Unsynced workout session records — hold logged sets not yet on the server.
		if (plainKey.includes('offline.today.session.')) {
			if (preserveWorkoutSessions) return false;
			return inUserScope(plainKey);
		}
		// Read-only cached views / planned template — safe to drop within scope.
		if (
			plainKey.includes('offline.today.recentSessions') ||
			plainKey.includes('today.plannedTemplate')
		) {
			return inUserScope(plainKey);
		}
		// Legacy unscoped pointers — only clear when doing an all-users wipe.
		if (plainKey.includes('currentWorkoutId') || plainKey.includes('currentWorkoutState')) {
			return !userId;
		}
		return false;
	};

	try {
		for (let i = localStorage.length - 1; i >= 0; i--) {
			const k = localStorage.key(i);
			if (!k) continue;
			if (shouldRemove(stripLsPrefix(k))) localStorage.removeItem(k);
		}
	} catch {
		// ignore
	}

	try {
		const allKeys = await kvListKeys('');
		const sensitiveKeys = allKeys.filter((key) => shouldRemove(key));
		await Promise.all(sensitiveKeys.map((key) => kvDelete(key)));
	} catch {
		// ignore
	}

	try {
		if (typeof caches !== 'undefined') {
			const cacheNames = await caches.keys();
			await Promise.all(
				cacheNames
					.filter((name) => name.startsWith(CACHE_STORAGE_PREFIX))
					.map((name) => caches.delete(name))
			);
		}
	} catch {
		// ignore
	}
}

function readStoredUser(): PublicUser | null {
	if (!browser) return null;
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		if (!raw) return null;
		const parsed = JSON.parse(raw) as PublicUser;
		if (!parsed || typeof parsed !== 'object') return null;
		if (typeof parsed.id !== 'number') return null;
		if (typeof parsed.username !== 'string') return null;
		if (parsed.role !== 'admin' && parsed.role !== 'user') return null;
		const must_change_password =
			typeof (parsed as { must_change_password?: unknown }).must_change_password === 'boolean'
				? (parsed as { must_change_password: boolean }).must_change_password
				: false;
		return { ...parsed, must_change_password };
	} catch {
		return null;
	}
}

function persistUser(user: PublicUser | null) {
	if (!browser) return;
	try {
		if (user) localStorage.setItem(STORAGE_KEY, JSON.stringify(user));
		else localStorage.removeItem(STORAGE_KEY);
	} catch {
		// ignore
	}
}

function isNetworkFailure(e: unknown): boolean {
	if (typeof navigator !== 'undefined' && navigator.onLine === false) return true;
	if (e instanceof TypeError) return true;
	// A request timeout / abort (see the request timeout in api.ts) is a
	// connectivity problem, not an auth failure (F-MED-7).
	if (
		typeof DOMException !== 'undefined' &&
		e instanceof DOMException &&
		(e.name === 'TimeoutError' || e.name === 'AbortError')
	) {
		return true;
	}
	const message = e instanceof Error ? e.message : String(e);
	return /failed to fetch|networkerror|load failed|connection|timeout|timed out|aborted/i.test(
		message
	);
}

function createAuthStore() {
	const initialUser = readStoredUser();
	setActiveUserId(initialUser?.id ?? null);

	const state = writable<AuthState>({
		status: 'unknown',
		user: initialUser,
		offline: false
	});

	// `fullWipe` is only set on explicit logout. A 401 (the default path) preserves
	// unsynced workout sessions so a routine session lapse mid-workout does not
	// silently destroy logged sets (F-CRIT-1).
	function handleUnauthorized(opts: { fullWipe?: boolean } = {}) {
		const activeId = getActiveUserId();
		state.set({ status: 'unauthenticated', user: null, offline: false });
		persistUser(null);
		setActiveUserId(null);
		void clearClientSensitiveData({
			preserveWorkoutSessions: !opts.fullWipe,
			userId: activeId
		});
	}

	if (browser) setUnauthorizedHandler(() => handleUnauthorized());

	async function refresh(fetcher: typeof fetch = fetch) {
		try {
			const user = await authMe(fetcher);
			const previousUserId = getActiveUserId();
			if (previousUserId && previousUserId !== String(user.id)) {
				// Switching accounts: clear only the departing user's read-only data and
				// preserve their unsynced sessions (F-HIGH-4).
				await clearClientSensitiveData({
					preserveWorkoutSessions: true,
					userId: previousUserId
				});
			}
			setActiveUserId(user.id);
			state.set({ status: 'authenticated', user, offline: false });
			persistUser(user);
		} catch (e) {
			if (isNetworkFailure(e)) {
				state.update((current) => ({
					...current,
					offline: true
				}));
				return;
			}
			if (isApiError(e) && e.status === 401) {
				handleUnauthorized();
			} else {
				state.update((current) => ({ ...current, offline: false }));
			}
		}
	}

	async function login(username: string, password: string, fetcher: typeof fetch = fetch) {
		const user = await authLogin(username, password, fetcher);
		const previousUserId = getActiveUserId();
		if (previousUserId && previousUserId !== String(user.id)) {
			// Switching accounts on a shared device: scope the wipe to the departing
			// user and keep their unsynced sessions (F-HIGH-4).
			await clearClientSensitiveData({
				preserveWorkoutSessions: true,
				userId: previousUserId
			});
		}
		setActiveUserId(user.id);
		state.set({ status: 'authenticated', user, offline: false });
		persistUser(user);
	}

	async function logout(fetcher: typeof fetch = fetch) {
		try {
			await authLogout(fetcher);
		} finally {
			// Explicit logout is the one path allowed to destroy unsynced sessions.
			handleUnauthorized({ fullWipe: true });
		}
	}

	async function changePassword(
		currentPassword: string,
		newPassword: string,
		fetcher: typeof fetch = fetch
	) {
		await authChangePassword(currentPassword, newPassword, fetcher);
		await refresh(fetcher);
	}

	return { state, refresh, login, logout, changePassword };
}

export const auth = createAuthStore();
