import { render, waitFor } from '@testing-library/svelte';
import { writable } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';

type AuthState = {
	status: 'unknown' | 'authenticated' | 'unauthenticated';
	user: { id: number; username: string; role: 'admin' | 'user'; must_change_password: boolean } | null;
	offline: boolean;
};

const pageStore = writable({ url: new URL('http://localhost/') });

const authStateStore = writable<AuthState>({
	status: 'authenticated',
	user: { id: 1, username: 'admin', role: 'admin', must_change_password: false },
	offline: false
});

const gotoMock = vi.hoisted(() => vi.fn(async () => undefined));
const authMocks = vi.hoisted(() => ({
	refresh: vi.fn(async () => undefined),
	logout: vi.fn(async () => undefined)
}));
const loggerMocks = vi.hoisted(() => ({
	setRemoteEnabled: vi.fn(),
	debug: vi.fn(),
	info: vi.fn(),
	warn: vi.fn(),
	error: vi.fn()
}));

vi.mock('$app/navigation', () => ({ goto: gotoMock }));
vi.mock('$app/environment', () => ({ browser: true }));

vi.mock('$app/stores', () => ({
	page: pageStore
}));

vi.mock('$lib/auth', () => ({
	auth: {
		state: authStateStore,
		refresh: authMocks.refresh,
		logout: authMocks.logout
	}
}));

vi.mock('$lib/logger', () => ({
	logger: loggerMocks
}));

describe('layout behavior', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		localStorage.clear();
		pageStore.set({ url: new URL('http://localhost/') });
		authStateStore.set({
			status: 'authenticated',
			user: { id: 1, username: 'admin', role: 'admin', must_change_password: false },
			offline: false
		});
	});

	it('redirects unauthenticated users to login when not on login route', async () => {
		pageStore.set({ url: new URL('http://localhost/workouts') });
		authStateStore.set({ status: 'unauthenticated', user: null, offline: false });

		const { default: Layout } = await import('../routes/+layout.svelte');
		render(Layout as never);

		await waitFor(() => expect(gotoMock).toHaveBeenCalledWith('/login'));
	}, 15_000);

	it('redirects authenticated users away from login and enforces password-change route', async () => {
		pageStore.set({ url: new URL('http://localhost/login') });
		authStateStore.set({
			status: 'authenticated',
			user: { id: 1, username: 'admin', role: 'admin', must_change_password: true },
			offline: false
		});

		const { default: Layout } = await import('../routes/+layout.svelte');
		render(Layout as never);
		await waitFor(() => expect(gotoMock).toHaveBeenCalledWith('/settings'));
	});

	it('hides admin/backups nav when offline and shows offline banner', async () => {
		const { default: Layout } = await import('../routes/+layout.svelte');
		const view = render(Layout as never);

		expect(view.getByText('Admin')).toBeInTheDocument();
		expect(view.getByText('Backups')).toBeInTheDocument();

		authStateStore.set({
			status: 'authenticated',
			user: { id: 1, username: 'admin', role: 'admin', must_change_password: false },
			offline: true
		});

		await waitFor(() => {
			expect(view.queryByText('Admin')).not.toBeInTheDocument();
			expect(view.queryByText('Backups')).not.toBeInTheDocument();
			expect(
				view.getByText('Offline mode: showing cached data. Some actions are disabled.')
			).toBeInTheDocument();
		});
	});
});
