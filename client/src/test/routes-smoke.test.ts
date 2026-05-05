import { fireEvent, render } from '@testing-library/svelte';
import type { PlannedTemplateExercise, UiSession } from '$lib/today/types';
import { readable, writable } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('$app/stores', () => ({
	page: readable({
		url: new URL('http://localhost/')
	})
}));

vi.mock('$app/navigation', () => ({
	goto: vi.fn(async () => {})
}));

const authStateStore = writable({
	status: 'authenticated' as const,
	user: { id: 1, username: 'admin', role: 'admin' as const, must_change_password: false },
	offline: false
});

const todayControllerMocks = vi.hoisted(() => ({
	startSession: vi.fn(async () => undefined),
	startSessionFromTemplate: vi.fn(async () => undefined)
}));

const todayCurrentSessionStore = writable<UiSession | null>(null);
const todayOpenExerciseIdStore = writable<number | null>(null);
const todayPlannedTemplateExercisesStore = writable<PlannedTemplateExercise[]>([]);

vi.mock('$lib/auth', () => ({
	auth: {
		state: authStateStore,
		refresh: vi.fn(async () => undefined),
		login: vi.fn(async () => undefined),
		logout: vi.fn(async () => undefined),
		changePassword: vi.fn(async () => undefined)
	}
}));

vi.mock('$lib/logger', () => ({
	logger: {
		setRemoteEnabled: vi.fn(),
		debug: vi.fn(),
		info: vi.fn(),
		warn: vi.fn(),
		error: vi.fn()
	}
}));

vi.mock('$lib/api', () => {
	class ApiError extends Error {
		status: number;
		constructor(status: number, message: string) {
			super(message);
			this.name = 'ApiError';
			this.status = status;
		}
	}

	return {
		ApiError,
		isApiError: (e: unknown): e is ApiError => e instanceof ApiError,
		setUnauthorizedHandler: vi.fn(),
		authMe: vi.fn(async () => ({
			id: 1,
			username: 'admin',
			role: 'admin',
			must_change_password: false
		})),
		authLogin: vi.fn(async () => ({
			status: 'ok',
			user: {
				id: 1,
				username: 'admin',
				role: 'admin',
				must_change_password: false
			}
		})),
		authLogout: vi.fn(async () => undefined),
		authChangePassword: vi.fn(async () => undefined),
		getWorkouts: vi.fn(async () => []),
		getWorkoutTemplates: vi.fn(async () => [
			{
				id: 5,
				name: 'Push A',
				exercise_count: 3,
				created_at: '2026-01-01T00:00:00.000Z',
				updated_at: '2026-01-02T00:00:00.000Z'
			}
		]),
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
			filename: 'swolemate_2026-03-03_01-00_manual.tar.gz',
			created_at: new Date().toISOString(),
			backup_type: 'Manual'
		})),
		restoreBackup: vi.fn(async () => undefined),
		deleteBackup: vi.fn(async () => undefined),
		getWorkoutStats: vi.fn(async () => null),
		getExerciseTypes: vi.fn(async () => []),
		getVolumeStats: vi.fn(async () => null),
		getExerciseProgress: vi.fn(async () => []),
		getMcpTokens: vi.fn(async () => []),
		createMcpToken: vi.fn(async () => ({
			id: 1,
			token: 'smcp_test',
			name: 'Test token',
			scopes: ['workouts.read', 'progress.read'],
			expires_at: new Date().toISOString()
		})),
		revokeMcpToken: vi.fn(async () => undefined),
		rotateMcpToken: vi.fn(async () => ({
			id: 2,
			token: 'smcp_rotated',
			name: 'Rotated token',
			scopes: ['workouts.read', 'progress.read'],
			expires_at: new Date().toISOString()
		})),
		adminListUsers: vi.fn(async () => []),
		adminCreateUser: vi.fn(async () => ({ id: 2 })),
		adminDisableUser: vi.fn(async () => undefined),
		adminResetUserPassword: vi.fn(async () => undefined),
		adminDeleteUser: vi.fn(async () => undefined)
	};
});

vi.mock('$lib/today/controller', () => {
	return {
		createTodayController: () => ({
			addExercise: vi.fn(async () => undefined),
			addExerciseSetting: vi.fn(),
			addSet: vi.fn(async () => undefined),
			cancelSession: vi.fn(async () => undefined),
			currentSession: todayCurrentSessionStore,
			elapsedLabel: writable('0m'),
			endModalOpen: writable(false),
			endMood: writable(null),
			endNotes: writable(''),
			error: writable(null),
			exerciseQuery: writable(''),
			getLastTimeForExercise: vi.fn(() => null),
			loading: writable(false),
			notice: writable(null),
			offlineMode: writable(false),
			markExerciseDone: vi.fn(async () => undefined),
			openEndModal: vi.fn(),
			openExerciseId: todayOpenExerciseIdStore,
			plannedTemplateExercises: todayPlannedTemplateExercisesStore,
			pendingSyncCount: writable(0),
			quickPicks: writable<string[]>([]),
			recentSessions: writable([]),
			removeExercise: vi.fn(async () => undefined),
			removeExerciseSetting: vi.fn(),
			sessionNotes: writable(''),
			start: vi.fn(() => () => undefined),
			startSession: todayControllerMocks.startSession,
			startSessionFromTemplate: todayControllerMocks.startSessionFromTemplate,
			startPlannedTemplateExercise: vi.fn(async () => undefined),
			submitEndSession: vi.fn(async () => undefined),
			syncPendingSessions: vi.fn(async () => undefined),
			suggestions: writable<string[]>([]),
			toggleExercise: vi.fn(),
			toggleExercisePerSideWeight: vi.fn(async () => undefined),
			toggleExerciseSplitWeight: vi.fn(async () => undefined),
			totalSets: writable(0),
			totalVolumeKg: writable(0),
			totalDurationSeconds: writable(0),
			updateExerciseNotes: vi.fn(),
			updateExerciseSetting: vi.fn()
		})
	};
});

async function importComponent(path: string) {
	const mod = (await import(path)) as { default: unknown };
	return mod.default;
}

beforeEach(() => {
	localStorage.clear();
	vi.clearAllMocks();
	todayCurrentSessionStore.set(null);
	todayOpenExerciseIdStore.set(null);
	todayPlannedTemplateExercisesStore.set([]);
});

describe('route smoke', () => {
	it('renders today page', async () => {
		const TodayPage = await importComponent('../routes/+page.svelte');
		const { getByRole, queryByRole } = render(TodayPage as never);
		expect(getByRole('heading', { name: 'Today' })).toBeInTheDocument();
		expect(queryByRole('button', { name: 'Load demo' })).not.toBeInTheDocument();
	});

	it('shows and starts demo mode when enabled in settings', async () => {
		localStorage.setItem('settings.showDemoMode', 'true');
		const TodayPage = await importComponent('../routes/+page.svelte');
		const { getByRole } = render(TodayPage as never);

		await fireEvent.click(getByRole('button', { name: 'Load demo' }));

		expect(todayControllerMocks.startSession).toHaveBeenCalledWith('demo');
	});

	it('opens template picker and starts from a template', async () => {
		const TodayPage = await importComponent('../routes/+page.svelte');
		const { getByRole, findByRole } = render(TodayPage as never);

		await fireEvent.click(getByRole('button', { name: 'Use template' }));
		await fireEvent.click(await findByRole('button', { name: /Push A/i }));

		expect(todayControllerMocks.startSessionFromTemplate).toHaveBeenCalledWith(5);
	});

	it('renders only the selected exercise open', async () => {
		todayOpenExerciseIdStore.set(101);
		todayCurrentSessionStore.set({
			id: 12,
			startedAt: '2026-01-01T10:00:00.000Z',
			notes: '',
			exercises: [
				{
					id: 101,
					name: 'Bench Press',
					notes: '',
					startedAt: '2026-01-01T10:00:00.000Z',
					endedAt: '2026-01-01T10:05:00.000Z',
					sets: [],
					settings: [],
					perSideWeight: false,
					splitWeight: false,
					status: 'active' as const
				},
				{
					id: 202,
					name: 'Cable Row',
					notes: '',
					startedAt: '2026-01-01T10:06:00.000Z',
					endedAt: '2026-01-01T10:15:00.000Z',
					sets: [],
					settings: [],
					perSideWeight: false,
					splitWeight: false,
					status: 'done' as const
				}
			]
		});

		const TodayPage = await importComponent('../routes/+page.svelte');
		const { getByText, queryAllByText } = render(TodayPage as never);

		expect(getByText('Bench Press')).toBeInTheDocument();
		expect(getByText('Cable Row')).toBeInTheDocument();
		expect(queryAllByText('Add your first set for this exercise.')).toHaveLength(1);
		expect(queryAllByText('Mark done')).toHaveLength(1);
		expect(queryAllByText('Collapse')).toHaveLength(1);
	});

	it('points empty template sessions to the template plan', async () => {
		todayCurrentSessionStore.set({
			id: 12,
			startedAt: '2026-01-01T10:00:00.000Z',
			notes: '',
			exercises: []
		});
		todayPlannedTemplateExercisesStore.set([{ id: 77, name: 'Bench Press' }]);

		const TodayPage = await importComponent('../routes/+page.svelte');
		const { getByText } = render(TodayPage as never);

		expect(getByText('Start your template plan')).toBeInTheDocument();
		expect(getByText('Start an exercise from your template plan below.')).toBeInTheDocument();
		expect(getByText('Template plan')).toBeInTheDocument();
		expect(getByText('1 left')).toBeInTheDocument();
	});

	it('renders workouts page with basic data', async () => {
		authStateStore.set({
			status: 'authenticated',
			user: { id: 1, username: 'admin', role: 'admin', must_change_password: false },
			offline: false
		});
		const WorkoutsPage = await importComponent('../routes/workouts/+page.svelte');
		const { getByText } = render(
			WorkoutsPage as never,
			{ props: { data: { workouts: [] } } } as never
		);
		expect(getByText('History')).toBeInTheDocument();
	});

	it('blocks workout delete when auth state is offline', async () => {
		const { cancelWorkout } = await import('$lib/api');
		authStateStore.set({
			status: 'authenticated',
			user: { id: 1, username: 'admin', role: 'admin', must_change_password: false },
			offline: true
		});

		const WorkoutsPage = await importComponent('../routes/workouts/+page.svelte');
		const { getByRole, findByText } = render(
			WorkoutsPage as never,
			{
				props: {
					data: {
						workouts: [
							{
								id: 7,
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

		await fireEvent.click(getByRole('button', { name: 'Delete' }));
		expect(
			await findByText('Offline mode: delete workouts when you are back online.')
		).toBeInTheDocument();
		expect(cancelWorkout).not.toHaveBeenCalled();
	});

	it('renders backups page with basic data', async () => {
		authStateStore.set({
			status: 'authenticated',
			user: { id: 1, username: 'admin', role: 'admin', must_change_password: false },
			offline: false
		});
		const BackupsPage = await importComponent('../routes/backups/+page.svelte');
		const { getByText } = render(
			BackupsPage as never,
			{ props: { data: { backups: [] } } } as never
		);
		expect(getByText('Backups')).toBeInTheDocument();
	});

	it('renders progress page', async () => {
		const ProgressPage = await importComponent('../routes/progress/+page.svelte');
		const { getByText } = render(ProgressPage as never);
		expect(getByText('Refresh')).toBeInTheDocument();
	}, 10_000);

	it('renders admin page', async () => {
		const AdminPage = await importComponent('../routes/admin/+page.svelte');
		const { getByRole } = render(AdminPage as never);
		expect(getByRole('heading', { name: 'Admin' })).toBeInTheDocument();
	});

	it('renders settings page', async () => {
		const SettingsPage = await importComponent('../routes/settings/+page.svelte');
		const { getByText } = render(SettingsPage as never);
		expect(getByText('Settings')).toBeInTheDocument();
	});

	it('renders help page', async () => {
		const HelpPage = await importComponent('../routes/help/+page.svelte');
		const { getByText } = render(HelpPage as never);
		expect(getByText('Help')).toBeInTheDocument();
	});

	it('renders login page', async () => {
		const LoginPage = await importComponent('../routes/login/+page.svelte');
		const { getByRole } = render(LoginPage as never);
		expect(getByRole('heading', { name: 'Sign in' })).toBeInTheDocument();
	});
});
