import { render } from '@testing-library/svelte';
import { readable, writable } from 'svelte/store';
import { describe, expect, it, vi } from 'vitest';

vi.mock('$app/stores', () => ({
	page: readable({
		url: new URL('http://localhost/')
	})
}));

vi.mock('$app/navigation', () => ({
	goto: vi.fn(async () => {})
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
		getWorkout: vi.fn(async () => ({ workout: null, exercises: [] })),
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
		adminListUsers: vi.fn(async () => []),
		adminCreateUser: vi.fn(async () => ({ id: 2 })),
		adminDisableUser: vi.fn(async () => undefined),
		adminResetUserPassword: vi.fn(async () => undefined),
		adminDeleteUser: vi.fn(async () => undefined)
	};
});

vi.mock('$lib/today/controller', () => {
	const currentSession = writable(null);
	return {
		createTodayController: () => ({
			addExercise: vi.fn(async () => undefined),
			addExerciseSetting: vi.fn(),
			addSet: vi.fn(async () => undefined),
			cancelSession: vi.fn(async () => undefined),
			currentSession,
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
			openExerciseId: writable<number | null>(null),
			pendingSyncCount: writable(0),
			quickPicks: writable<string[]>([]),
			recentSessions: writable([]),
			removeExercise: vi.fn(async () => undefined),
			removeExerciseSetting: vi.fn(),
			sessionNotes: writable(''),
			start: vi.fn(() => () => undefined),
			startSession: vi.fn(async () => undefined),
			submitEndSession: vi.fn(async () => undefined),
			syncPendingSessions: vi.fn(async () => undefined),
			suggestions: writable<string[]>([]),
			toggleExercise: vi.fn(),
			toggleExercisePerSideWeight: vi.fn(async () => undefined),
			toggleExerciseSplitWeight: vi.fn(async () => undefined),
			totalSets: writable(0),
			totalVolumeKg: writable(0),
			updateExerciseNotes: vi.fn(),
			updateExerciseSetting: vi.fn()
		})
	};
});

async function importComponent(path: string) {
	const mod = (await import(path)) as { default: unknown };
	return mod.default;
}

describe('route smoke', () => {
	it('renders +layout without throwing', async () => {
		const Layout = await importComponent('../routes/+layout.svelte');
		const { getByText } = render(Layout as never);
		expect(getByText('SwoleMate')).toBeInTheDocument();
	}, 15_000);

	it('renders today page', async () => {
		const TodayPage = await importComponent('../routes/+page.svelte');
		const { getByText } = render(TodayPage as never);
		expect(getByText('Today')).toBeInTheDocument();
	});

	it('renders workouts page with basic data', async () => {
		const WorkoutsPage = await importComponent('../routes/workouts/+page.svelte');
		const { getByText } = render(
			WorkoutsPage as never,
			{ props: { data: { workouts: [] } } } as never
		);
		expect(getByText('History')).toBeInTheDocument();
	});

	it('renders backups page with basic data', async () => {
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
	});

	it('renders admin page', async () => {
		const AdminPage = await importComponent('../routes/admin/+page.svelte');
		const { getByRole } = render(AdminPage as never);
		expect(getByRole('heading', { name: 'Admin' })).toBeInTheDocument();
	});

	it('renders settings page', async () => {
		const SettingsPage = await importComponent('../routes/settings/+page.svelte');
		const { getByText } = render(SettingsPage as never);
		expect(getByText('Help')).toBeInTheDocument();
	});

	it('renders login page', async () => {
		const LoginPage = await importComponent('../routes/login/+page.svelte');
		const { getByRole } = render(LoginPage as never);
		expect(getByRole('heading', { name: 'Sign in' })).toBeInTheDocument();
	});
});
