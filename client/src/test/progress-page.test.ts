import { fireEvent, render, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const apiMocks = vi.hoisted(() => ({
	getWorkoutStats: vi.fn(),
	getProgressOverview: vi.fn(),
	getExerciseTypes: vi.fn(),
	getVolumeStats: vi.fn(),
	getExerciseProgress: vi.fn()
}));

vi.mock('$lib/api', () => ({
	getWorkoutStats: apiMocks.getWorkoutStats,
	getProgressOverview: apiMocks.getProgressOverview,
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
		apiMocks.getProgressOverview.mockResolvedValue({
			current_week: {
				label: 'Current week',
				start_date: '2026-05-04T00:00:00Z',
				end_date: '2026-05-11T00:00:00Z',
				workouts: 2,
				total_training_minutes: 95,
				exercises: 7,
				sets: 24,
				reps: 126,
				total_volume: 5200,
				timed_sets: 2,
				total_timed_duration_seconds: 135,
				pr_count: 1,
				comparison: {
					workouts_delta: 1,
					total_training_minutes_delta: 35,
					exercises_delta: 2,
					sets_delta: 6,
					reps_delta: 24,
					total_volume_delta: 900,
					timed_sets_delta: 1,
					total_timed_duration_seconds_delta: 45,
					pr_count_delta: 1
				}
			},
			last_30_days: {
				label: 'Last 30 days',
				start_date: '2026-04-05T00:00:00Z',
				end_date: '2026-05-05T00:00:00Z',
				workouts: 8,
				total_training_minutes: 410,
				exercises: 30,
				sets: 96,
				reps: 502,
				total_volume: 22100,
				timed_sets: 5,
				total_timed_duration_seconds: 390,
				pr_count: 3,
				comparison: {
					workouts_delta: 2,
					total_training_minutes_delta: 60,
					exercises_delta: 5,
					sets_delta: 18,
					reps_delta: 80,
					total_volume_delta: 4200,
					timed_sets_delta: 2,
					total_timed_duration_seconds_delta: 120,
					pr_count_delta: 2
				}
			},
			recent_prs: [
				{
					exercise_type: 'Bench Press',
					pr_type: 'estimated_1rm',
					new_value: 100,
					previous_value: 95,
					date: '2026-05-05T10:00:00Z',
					set_id: 9,
					set_details: { reps: 5, weight: 87.5, duration_seconds: null }
				}
			]
		});
		apiMocks.getExerciseTypes.mockResolvedValue(['Bench Press', 'Squat']);
		apiMocks.getVolumeStats.mockResolvedValue({
			weekly_volume: [],
			monthly_volume: [],
			personal_records: { all_time_max_weight: 0, max_volume: 0, estimated_max_1rm: 0 },
			timed_records: {
				longest_set_seconds: 75,
				best_session_duration_seconds: 135,
				lifetime_duration_seconds: 135,
				average_set_duration_seconds: 68
			}
		});
		apiMocks.getExerciseProgress.mockResolvedValue([]);
	});

	it('loads overall and exercise data on mount', async () => {
		const { default: ProgressPage } = await import('../routes/progress/+page.svelte');
		render(ProgressPage as never);

		await waitFor(() => {
			expect(apiMocks.getWorkoutStats).toHaveBeenCalledTimes(1);
			expect(apiMocks.getProgressOverview).toHaveBeenCalledTimes(1);
			expect(apiMocks.getExerciseTypes).toHaveBeenCalledTimes(1);
			expect(apiMocks.getVolumeStats).toHaveBeenCalledWith('Bench Press');
			expect(apiMocks.getExerciseProgress).toHaveBeenCalledWith('Bench Press');
		});
	}, 15_000);

	it('renders overview, recent PRs, and timed records', async () => {
		const { default: ProgressPage } = await import('../routes/progress/+page.svelte');
		const { findByText } = render(ProgressPage as never);

		expect(await findByText('Current progress')).toBeInTheDocument();
		expect(await findByText('Recent PRs')).toBeInTheDocument();
		expect(await findByText(/Estimated 1RM/)).toBeInTheDocument();
		expect(await findByText('Longest timed set')).toBeInTheDocument();
		expect(await findByText('1m 15s')).toBeInTheDocument();
	});

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
		const { findAllByText } = render(ProgressPage as never);

		expect((await findAllByText('stats down')).length).toBeGreaterThan(0);
	});

	it('shows exercise error when progress load fails', async () => {
		apiMocks.getVolumeStats.mockRejectedValue(new Error('progress down'));
		const { default: ProgressPage } = await import('../routes/progress/+page.svelte');
		const { findAllByText } = render(ProgressPage as never);

		expect((await findAllByText('progress down')).length).toBeGreaterThan(0);
	});
});
