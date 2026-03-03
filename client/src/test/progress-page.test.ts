import { cleanup, fireEvent, render, waitFor } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const apiMocks = vi.hoisted(() => ({
	getWorkoutStats: vi.fn(),
	getExerciseTypes: vi.fn(),
	getVolumeStats: vi.fn(),
	getExerciseProgress: vi.fn()
}));

vi.mock('$lib/api', () => ({
	getWorkoutStats: apiMocks.getWorkoutStats,
	getExerciseTypes: apiMocks.getExerciseTypes,
	getVolumeStats: apiMocks.getVolumeStats,
	getExerciseProgress: apiMocks.getExerciseProgress
}));

vi.mock('$lib/progress/charting', () => ({
	upsertChart: vi.fn(() => null),
	baseOptions: vi.fn(() => ({})),
	readTheme: vi.fn(() => ({
		isDark: false,
		text: '#111',
		mutedText: '#222',
		grid: '#333',
		primary: '#444',
		secondary: '#555',
		tertiary: '#666',
		success: '#777',
		warning: '#888',
		error: '#999'
	})),
	observeTheme: vi.fn(() => ({ disconnect: vi.fn() })),
	rgba: vi.fn((color: string) => color)
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

describe('progress route page', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		localStorage.clear();
		apiMocks.getWorkoutStats.mockResolvedValue({
			total_workouts: 0,
			workout_frequency: { average_per_week: 0, trend: 0 },
			average_duration_minutes: 0,
			duration_trend: 0,
			feedback_distribution: { good: 0, neutral: 0, bad: 0 },
			session_start_times: [],
			duration_distribution: [],
			sessions_per_month: [],
			avg_exercise_duration_series: []
		});
		apiMocks.getExerciseTypes.mockResolvedValue(['Bench Press', 'Squat']);
		apiMocks.getVolumeStats.mockResolvedValue(null);
		apiMocks.getExerciseProgress.mockResolvedValue([]);
	});

	afterEach(() => {
		cleanup();
	});

	it('loads overall and exercise data on mount', async () => {
		const { default: ProgressPage } = await import('../routes/progress/+page.svelte');
		render(ProgressPage as never);

		await waitFor(() => {
			expect(apiMocks.getWorkoutStats).toHaveBeenCalledTimes(1);
			expect(apiMocks.getExerciseTypes).toHaveBeenCalledTimes(1);
			expect(apiMocks.getVolumeStats).toHaveBeenCalledWith('Bench Press');
			expect(apiMocks.getExerciseProgress).toHaveBeenCalledWith('Bench Press');
		});
	}, 15_000);

	it('reloads exercise-specific data when selection changes', async () => {
		const { default: ProgressPage } = await import('../routes/progress/+page.svelte');
		const { getByRole } = render(ProgressPage as never);

		await waitFor(() => expect(apiMocks.getVolumeStats).toHaveBeenCalledWith('Bench Press'));

		const targetSelect = getByRole('combobox', { name: 'Exercise' });
		await fireEvent.change(targetSelect, { target: { value: 'Squat' } });

		await waitFor(() => {
			expect(apiMocks.getVolumeStats).toHaveBeenCalledWith('Squat');
			expect(apiMocks.getExerciseProgress).toHaveBeenCalledWith('Squat');
		});
		expect(localStorage.getItem('progress.selectedExercise')).toBe('Squat');
	});

	it('shows error when overall load fails', async () => {
		apiMocks.getWorkoutStats.mockRejectedValueOnce(new Error('stats down'));
		const { default: ProgressPage } = await import('../routes/progress/+page.svelte');
		const { findByText } = render(ProgressPage as never);

		expect(await findByText('stats down')).toBeInTheDocument();
	});

	it('shows exercise error when progress load fails', async () => {
		apiMocks.getVolumeStats.mockRejectedValue(new Error('progress down'));
		const { default: ProgressPage } = await import('../routes/progress/+page.svelte');
		const { findAllByText } = render(ProgressPage as never);

		expect((await findAllByText('progress down')).length).toBeGreaterThan(0);
	});
});
