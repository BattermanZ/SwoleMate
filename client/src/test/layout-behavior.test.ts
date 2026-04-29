import { fireEvent, render, waitFor, within } from '@testing-library/svelte';
import { writable } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';

type AuthState = {
	status: 'unknown' | 'authenticated' | 'unauthenticated';
	user: {
		id: number;
		username: string;
		role: 'admin' | 'user';
		must_change_password: boolean;
	} | null;
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
	}, 30_000);

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

		const mobileNav = view.getByRole('navigation', { name: 'Primary mobile navigation' });
		await fireEvent.click(within(mobileNav).getByRole('button', { name: /open more navigation/i }));
		const moreMenu = view.getByRole('dialog');
		expect(within(moreMenu).queryByRole('link', { name: /Admin/i })).not.toBeInTheDocument();
		expect(within(moreMenu).queryByRole('link', { name: /Backups/i })).not.toBeInTheDocument();
	});

	it('renders mobile bottom navigation and opens secondary routes from More', async () => {
		const { default: Layout } = await import('../routes/+layout.svelte');
		const view = render(Layout as never);

		const mobileNav = view.getByRole('navigation', { name: 'Primary mobile navigation' });
		expect(within(mobileNav).getByRole('link', { name: /Today/i })).toHaveAttribute('href', '/');
		expect(within(mobileNav).getByRole('link', { name: /Templates/i })).toHaveAttribute(
			'href',
			'/templates'
		);
		expect(within(mobileNav).getByRole('link', { name: /History/i })).toHaveAttribute(
			'href',
			'/workouts'
		);
		expect(within(mobileNav).getByRole('link', { name: /Progress/i })).toHaveAttribute(
			'href',
			'/progress'
		);

		await fireEvent.click(within(mobileNav).getByRole('button', { name: /open more navigation/i }));

		const moreMenu = view.getByRole('dialog');
		expect(within(moreMenu).getByRole('link', { name: /Settings/i })).toHaveAttribute(
			'href',
			'/settings'
		);
		expect(within(moreMenu).getByRole('link', { name: /Help/i })).toHaveAttribute('href', '/help');
		expect(within(moreMenu).getByRole('link', { name: /Admin/i })).toHaveAttribute(
			'href',
			'/admin'
		);
		expect(within(moreMenu).getByRole('link', { name: /Backups/i })).toHaveAttribute(
			'href',
			'/backups'
		);
	});

	it('moves the active mobile tab when the route changes', async () => {
		const { default: Layout } = await import('../routes/+layout.svelte');
		const view = render(Layout as never);
		const mobileNav = view.getByRole('navigation', { name: 'Primary mobile navigation' });

		expect(within(mobileNav).getByRole('link', { name: /Today/i })).toHaveAttribute(
			'aria-current',
			'page'
		);

		pageStore.set({ url: new URL('http://localhost/progress') });

		await waitFor(() => {
			expect(within(mobileNav).getByRole('link', { name: /Today/i })).not.toHaveAttribute(
				'aria-current'
			);
			expect(within(mobileNav).getByRole('link', { name: /Progress/i })).toHaveAttribute(
				'aria-current',
				'page'
			);
		});
	});

	it('toggles the .dark class and persists to localStorage', async () => {
		localStorage.removeItem('theme');
		document.documentElement.classList.remove('dark');

		const { default: Layout } = await import('../routes/+layout.svelte');
		const { getByRole } = render(Layout as never);

		const toggle = getByRole('button', { name: /toggle dark mode/i });
		expect(document.documentElement.classList.contains('dark')).toBe(false);

		await fireEvent.click(toggle);
		expect(document.documentElement.classList.contains('dark')).toBe(true);
		expect(localStorage.getItem('theme')).toBe('dark');

		await fireEvent.click(toggle);
		expect(document.documentElement.classList.contains('dark')).toBe(false);
		expect(localStorage.getItem('theme')).toBe('light');
	});
});
