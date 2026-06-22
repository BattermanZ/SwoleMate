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

async function clearClientSensitiveData(): Promise<void> {
	if (!browser) return;

	try {
		for (let i = localStorage.length - 1; i >= 0; i--) {
			const k = localStorage.key(i);
			if (!k) continue;
			if (k.startsWith('swolemate-cache:') || k.includes(':swolemate-cache:')) {
				localStorage.removeItem(k);
				continue;
			}
			if (
				k.includes('offline.today.session.') ||
				k.includes('offline.today.recentSessions') ||
				k.includes('today.plannedTemplate')
			) {
				localStorage.removeItem(k);
				continue;
			}
			if (k.includes('currentWorkoutId') || k.includes('swolemate:currentWorkoutState')) {
				localStorage.removeItem(k);
			}
		}
	} catch {
		// ignore
	}

	try {
		const allKeys = await kvListKeys('');
		const sensitiveKeys = allKeys.filter(
			(key) =>
				key.includes('offline.today.session.') ||
				key.includes('offline.today.recentSessions') ||
				key.includes('today.plannedTemplate') ||
				key.includes('currentWorkoutId') ||
				key.includes('swolemate:currentWorkoutState')
		);
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
	const message = e instanceof Error ? e.message : String(e);
	return /failed to fetch|networkerror|load failed|connection|timeout/i.test(message);
}

function createAuthStore() {
	const initialUser = readStoredUser();
	setActiveUserId(initialUser?.id ?? null);

	const state = writable<AuthState>({
		status: 'unknown',
		user: initialUser,
		offline: false
	});

	function handleUnauthorized() {
		state.set({ status: 'unauthenticated', user: null, offline: false });
		persistUser(null);
		setActiveUserId(null);
		void clearClientSensitiveData();
	}

	if (browser) setUnauthorizedHandler(handleUnauthorized);

	async function refresh(fetcher: typeof fetch = fetch) {
		try {
			const user = await authMe(fetcher);
			const previousUserId = getActiveUserId();
			if (previousUserId && previousUserId !== String(user.id)) {
				await clearClientSensitiveData();
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
			await clearClientSensitiveData();
		}
		setActiveUserId(user.id);
		state.set({ status: 'authenticated', user, offline: false });
		persistUser(user);
	}

	async function logout(fetcher: typeof fetch = fetch) {
		try {
			await authLogout(fetcher);
		} finally {
			handleUnauthorized();
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
