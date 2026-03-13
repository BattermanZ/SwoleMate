import { fireEvent, render, waitFor } from '@testing-library/svelte';
import { readable, writable } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('$app/stores', () => ({
	page: readable({ url: new URL('http://localhost/') })
}));

vi.mock('$app/navigation', () => ({
	goto: vi.fn(async () => undefined)
}));

vi.mock('$app/environment', () => ({
	browser: true
}));

type AuthState = {
	status: 'authenticated' | 'unauthenticated';
	user: {
		id: number;
		username: string;
		role: 'admin' | 'user';
		must_change_password: boolean;
	} | null;
	offline: boolean;
};

const authStateStore = writable<AuthState>({
	status: 'authenticated',
	user: { id: 1, username: 'admin', role: 'admin', must_change_password: false },
	offline: false
});

const authMocks = vi.hoisted(() => ({
	login: vi.fn(async () => undefined),
	logout: vi.fn(async () => undefined),
	changePassword: vi.fn(async () => undefined),
	refresh: vi.fn(async () => undefined)
}));

vi.mock('$lib/auth', () => ({
	auth: {
		state: authStateStore,
		login: authMocks.login,
		logout: authMocks.logout,
		changePassword: authMocks.changePassword,
		refresh: authMocks.refresh
	}
}));

const apiMocks = vi.hoisted(() => ({
	getWorkouts: vi.fn(async () => []),
	getWorkout: vi.fn(async (id: number) => ({
		workout: {
			id,
			date: '2026-01-01T10:00:00.000Z',
			start_time: '2026-01-01T10:00:00.000Z',
			end_time: '2026-01-01T11:00:00.000Z',
			notes: '',
			feedback: null
		},
		exercises: []
	})),
	cancelWorkout: vi.fn(async () => undefined),
	updateWorkoutTimes: vi.fn(async () => undefined),
	getBackups: vi.fn(async () => []),
	createBackup: vi.fn(async () => ({
		filename: 'a',
		created_at: new Date().toISOString(),
		backup_type: 'Manual'
	})),
	getMcpTokens: vi.fn(async () => []),
	createMcpToken: vi.fn(async () => ({
		id: 1,
		token: 'smcp_created',
		name: 'Claude Desktop',
		scopes: ['workouts.read', 'progress.read'],
		expires_at: '2026-02-01T00:00:00.000Z'
	})),
	revokeMcpToken: vi.fn(async () => undefined),
	rotateMcpToken: vi.fn(async () => ({
		id: 8,
		token: 'smcp_rotated',
		name: 'Existing token',
		scopes: ['workouts.read', 'progress.read'],
		expires_at: '2026-02-10T00:00:00.000Z'
	})),
	restoreBackup: vi.fn(async () => undefined),
	deleteBackup: vi.fn(async () => undefined),
	adminListUsers: vi.fn(async () => []),
	adminCreateUser: vi.fn(async () => ({ id: 2 })),
	adminDisableUser: vi.fn(async () => undefined),
	adminResetUserPassword: vi.fn(async () => undefined),
	adminDeleteUser: vi.fn(async () => undefined)
}));

vi.mock('$lib/api', () => ({
	...apiMocks
}));

vi.mock('$lib/logger', () => ({
	logger: {
		error: vi.fn(),
		info: vi.fn(),
		debug: vi.fn(),
		warn: vi.fn(),
		setRemoteEnabled: vi.fn()
	}
}));

beforeEach(() => {
	vi.clearAllMocks();
	localStorage.clear();
	authStateStore.set({
		status: 'authenticated',
		user: { id: 1, username: 'admin', role: 'admin', must_change_password: false },
		offline: false
	});
	vi.stubGlobal(
		'confirm',
		vi.fn(() => true)
	);
	vi.stubGlobal(
		'prompt',
		vi.fn(() => 'user-a')
	);
});

describe('route behaviors', () => {
	it('workouts page shows refresh error and honors delete confirm', async () => {
		apiMocks.getWorkouts.mockRejectedValueOnce(new Error('refresh failed'));
		const { default: WorkoutsPage } = await import('../routes/workouts/+page.svelte');
		const { getByText, getByRole, findByText } = render(
			WorkoutsPage as never,
			{
				props: {
					data: {
						workouts: [
							{
								id: 10,
								date: '2026-01-01T10:00:00.000Z',
								start_time: '2026-01-01T10:00:00.000Z',
								end_time: '2026-01-01T11:00:00.000Z',
								notes: '',
								feedback: null
							}
						]
					}
				}
			} as never
		);

		await fireEvent.click(getByText('Refresh'));
		expect(await findByText('refresh failed')).toBeInTheDocument();

		const confirmMock = globalThis.confirm as ReturnType<typeof vi.fn>;
		confirmMock.mockReturnValueOnce(false);
		await fireEvent.click(getByRole('button', { name: 'Delete' }));
		expect(apiMocks.cancelWorkout).not.toHaveBeenCalled();

		await fireEvent.click(getByRole('button', { name: 'Delete' }));
		expect(apiMocks.cancelWorkout).toHaveBeenCalledWith(10);
	}, 10_000);

	it('backups page blocks admin actions for non-admin and surfaces export error', async () => {
		authStateStore.set({
			status: 'authenticated',
			user: { id: 2, username: 'user', role: 'user', must_change_password: false },
			offline: false
		});
		const { default: BackupsPage } = await import('../routes/backups/+page.svelte');
		let view = render(BackupsPage as never, { props: { data: { backups: [] } } } as never);
		expect(view.getByText('Admin only.')).toBeInTheDocument();
		view.unmount();

		authStateStore.set({
			status: 'authenticated',
			user: { id: 1, username: 'admin', role: 'admin', must_change_password: false },
			offline: false
		});
		apiMocks.getWorkouts.mockRejectedValueOnce(new Error('export failed'));
		view = render(
			BackupsPage as never,
			{
				props: {
					data: {
						backups: [
							{
								filename: 'snap.tar.gz',
								created_at: '2026-01-01T00:00:00.000Z',
								backup_type: 'Manual'
							}
						]
					}
				}
			} as never
		);
		await fireEvent.click(view.getByText('Export JSON'));
		expect(await view.findByText('export failed')).toBeInTheDocument();
	});

	it('admin page validates create/reset and handles blocked state', async () => {
		apiMocks.adminListUsers.mockResolvedValueOnce([
			{ id: 3, username: 'user-a', role: 'user', disabled_at: null }
		] as never);
		const { default: AdminPage } = await import('../routes/admin/+page.svelte');
		let view = render(AdminPage as never);
		await view.findByText('user-a');

		await fireEvent.click(view.getByRole('button', { name: 'Create user' }));
		expect(await view.findByText('Username and password are required.')).toBeInTheDocument();

		await fireEvent.click(view.getByText('Reset password'));
		await fireEvent.click(view.getByText('Reset'));
		expect(await view.findByText('New password is required.')).toBeInTheDocument();
		view.unmount();

		authStateStore.set({
			status: 'authenticated',
			user: { id: 1, username: 'admin', role: 'admin', must_change_password: false },
			offline: true
		});
		view = render(AdminPage as never);
		expect(view.getByText('Admin access required.')).toBeInTheDocument();
	});

	it('settings validates password mismatch, rotates MCP tokens, and login shows auth errors', async () => {
		apiMocks.getMcpTokens
			.mockResolvedValueOnce([
				{
					id: 7,
					name: 'Existing token',
					scopes: ['workouts.read', 'progress.read'],
					expires_at: '2026-02-01T00:00:00.000Z',
					revoked_at: null,
					last_used_at: null,
					created_at: '2026-01-01T00:00:00.000Z'
				}
			] as never)
			.mockResolvedValueOnce([
				{
					id: 8,
					name: 'Existing token',
					scopes: ['workouts.read', 'progress.read'],
					expires_at: '2026-02-10T00:00:00.000Z',
					revoked_at: null,
					last_used_at: '2026-01-05T10:15:00.000Z',
					created_at: '2026-01-01T00:00:00.000Z'
				}
			] as never)
			.mockResolvedValueOnce([
				{
					id: 8,
					name: 'Existing token',
					scopes: ['workouts.read', 'progress.read'],
					expires_at: '2026-02-10T00:00:00.000Z',
					revoked_at: null,
					last_used_at: '2026-01-05T10:15:00.000Z',
					created_at: '2026-01-01T00:00:00.000Z'
				}
			] as never)
			.mockResolvedValueOnce([
				{
					id: 8,
					name: 'Existing token',
					scopes: ['workouts.read', 'progress.read'],
					expires_at: '2026-02-10T00:00:00.000Z',
					revoked_at: '2026-01-02T00:00:00.000Z',
					last_used_at: '2026-01-05T10:15:00.000Z',
					created_at: '2026-01-01T00:00:00.000Z'
				}
			] as never);
		const { default: SettingsPage } = await import('../routes/settings/+page.svelte');
		const view = render(SettingsPage as never);
		await waitFor(() => expect(apiMocks.getMcpTokens).toHaveBeenCalledTimes(1));

		await fireEvent.click(view.getByText('Create MCP token'));
		expect(await view.findByText('Token name is required.')).toBeInTheDocument();

		await fireEvent.input(view.getByLabelText('Token name'), {
			target: { value: 'Claude Desktop' }
		});
		await fireEvent.change(view.getByLabelText('Access level'), { target: { value: 'write' } });
		await fireEvent.click(view.getByText('Create MCP token'));
		await waitFor(() =>
			expect(apiMocks.createMcpToken).toHaveBeenCalledWith({
				name: 'Claude Desktop',
				scopes: ['workouts.read', 'progress.read', 'workouts.write'],
				expires_in_days: 7
			})
		);
		expect(await view.findByText('Copy this token now')).toBeInTheDocument();
		expect(view.getByText('smcp_created')).toBeInTheDocument();

		await fireEvent.click(await view.findByText('Rotate'));
		await waitFor(() => expect(apiMocks.rotateMcpToken).toHaveBeenCalledWith(8));
		expect(await view.findByText('smcp_rotated')).toBeInTheDocument();
		expect(view.getByText('Last used')).toBeInTheDocument();

		await fireEvent.click(await view.findByText('Revoke'));
		await waitFor(() => expect(apiMocks.revokeMcpToken).toHaveBeenCalledWith(8));
		await waitFor(() => expect(view.queryByText('Existing token')).not.toBeInTheDocument());
		expect(view.getByText('No MCP tokens yet.')).toBeInTheDocument();

		const current = view.getByLabelText('Current password');
		const next = view.getByLabelText('New password');
		const confirmInput = view.getByLabelText('Confirm new password');
		await fireEvent.input(current, { target: { value: 'old' } });
		await fireEvent.input(next, { target: { value: 'new-a' } });
		await fireEvent.input(confirmInput, { target: { value: 'new-b' } });
		await fireEvent.click(view.getByText('Change password'));
		expect(await view.findByText('New passwords do not match.')).toBeInTheDocument();
		view.unmount();

		authMocks.login.mockRejectedValueOnce(new Error('bad credentials'));
		const { default: LoginPage } = await import('../routes/login/+page.svelte');
		const loginView = render(LoginPage as never);
		await fireEvent.input(loginView.getByLabelText('Username'), { target: { value: 'alice' } });
		await fireEvent.input(loginView.getByLabelText('Password'), { target: { value: 'wrong' } });
		await fireEvent.click(loginView.getByRole('button', { name: 'Sign in' }));
		await waitFor(() => expect(loginView.getByText('bad credentials')).toBeInTheDocument());
	});
});
