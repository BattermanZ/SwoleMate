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
			last_7_days: {
				label: 'Last 7 days',
				start_date: '2026-04-28T00:00:00Z',
				end_date: '2026-05-05T00:00:00Z',
				workouts: 2,
				total_training_minutes: 95,
				exercises: 7,
				sets: 24,
				reps: 126,
				total_volume: 5200,
				timed_sets: 2,
				total_timed_duration_seconds: 135,
				pr_count: 1,
				recent_best_count: 2,
				comparison: {
					workouts_delta: 1,
					total_training_minutes_delta: 35,
					exercises_delta: 2,
					sets_delta: 6,
					reps_delta: 24,
					total_volume_delta: 900,
					timed_sets_delta: 1,
					total_timed_duration_seconds_delta: 45,
					pr_count_delta: 1,
					recent_best_count_delta: 2
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
				recent_best_count: 6,
				comparison: {
					workouts_delta: 2,
					total_training_minutes_delta: 60,
					exercises_delta: 5,
					sets_delta: 18,
					reps_delta: 80,
					total_volume_delta: 4200,
					timed_sets_delta: 2,
					total_timed_duration_seconds_delta: 120,
					pr_count_delta: 2,
					recent_best_count_delta: 4
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
			],
			recent_bests: [
				{
					exercise_type: 'Plank',
					pr_type: 'timed_duration',
					new_value: 75,
					previous_value: 45,
					date: '2026-05-04T10:00:00Z',
					set_id: 10,
					set_details: { reps: 0, weight: 0, duration_seconds: 75 }
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

	it('loads overview data on mount without loading exercise details', async () => {
		const { default: ProgressPage } = await import('../routes/progress/+page.svelte');
		render(ProgressPage as never);

		await waitFor(() => {
			expect(apiMocks.getWorkoutStats).toHaveBeenCalledTimes(1);
			expect(apiMocks.getProgressOverview).toHaveBeenCalledTimes(1);
			expect(apiMocks.getExerciseTypes).toHaveBeenCalledTimes(1);
		});
		expect(apiMocks.getVolumeStats).not.toHaveBeenCalled();
		expect(apiMocks.getExerciseProgress).not.toHaveBeenCalled();
	}, 15_000);

	it('renders overview and recent PRs by default', async () => {
		const { default: ProgressPage } = await import('../routes/progress/+page.svelte');
		const { findByText, queryByText } = render(ProgressPage as never);

		expect(await findByText('Current progress')).toBeInTheDocument();
		expect(await findByText('Records')).toBeInTheDocument();
		expect(await findByText('Last 7 days')).toBeInTheDocument();
		expect(await findByText(/Estimated 1RM/)).toBeInTheDocument();
		expect(queryByText('Longest timed set')).not.toBeInTheDocument();
	});

	it('switches records between all-time PRs and recent bests', async () => {
		const { default: ProgressPage } = await import('../routes/progress/+page.svelte');
		const { findAllByText, findByRole, findByText } = render(ProgressPage as never);

		expect((await findAllByText('Bench Press')).length).toBeGreaterThan(0);
		await fireEvent.click(await findByRole('tab', { name: 'Recent bests' }));

		expect(await findByText('Plank')).toBeInTheDocument();
		expect(await findByText('Timed set')).toBeInTheDocument();
		expect(await findByText(/Timed duration/)).toBeInTheDocument();
	});

	it('loads and renders exercise details when the Exercise tab opens', async () => {
		const { default: ProgressPage } = await import('../routes/progress/+page.svelte');
		const { findByRole, findByText } = render(ProgressPage as never);

		await fireEvent.click(await findByRole('tab', { name: 'Exercise' }));

		await waitFor(() => {
			expect(apiMocks.getVolumeStats).toHaveBeenCalledWith('Bench Press');
			expect(apiMocks.getExerciseProgress).toHaveBeenCalledWith('Bench Press');
		});
		expect(await findByText('Longest timed set')).toBeInTheDocument();
		expect(await findByText('1m 15s')).toBeInTheDocument();
	});

	it('reloads exercise-specific data when selection changes', async () => {
		const { default: ProgressPage } = await import('../routes/progress/+page.svelte');
		const { findByRole, getByRole } = render(ProgressPage as never);

		await fireEvent.click(await findByRole('tab', { name: 'Exercise' }));
		await waitFor(() => expect(apiMocks.getVolumeStats).toHaveBeenCalledWith('Bench Press'));

		const targetSelect = getByRole('combobox', { name: 'Exercise' });
		await fireEvent.change(targetSelect, { target: { value: 'Squat' } });

		await waitFor(() => {
			expect(apiMocks.getVolumeStats).toHaveBeenCalledWith('Squat');
			expect(apiMocks.getExerciseProgress).toHaveBeenCalledWith('Squat');
		});
		expect(localStorage.getItem('progress.selectedExercise')).toBe('Squat');
	});

	it('persists selected tab and shows trends tab content', async () => {
		const { default: ProgressPage } = await import('../routes/progress/+page.svelte');
		const { findByRole, findByText, queryByText } = render(ProgressPage as never);

		await fireEvent.click(await findByRole('tab', { name: 'Trends' }));

		expect(localStorage.getItem('progress.selectedTab')).toBe('trends');
		expect(await findByText('Sessions per month')).toBeInTheDocument();
		expect(queryByText('Records')).not.toBeInTheDocument();
	});

	it('restores the Exercise tab and loads exercise details', async () => {
		localStorage.setItem('progress.selectedTab', 'exercise');
		const { default: ProgressPage } = await import('../routes/progress/+page.svelte');
		const { findByRole } = render(ProgressPage as never);

		expect(await findByRole('tab', { name: 'Exercise', selected: true })).toBeInTheDocument();
		await waitFor(() => {
			expect(apiMocks.getVolumeStats).toHaveBeenCalledWith('Bench Press');
			expect(apiMocks.getExerciseProgress).toHaveBeenCalledWith('Bench Press');
		});
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
		const { findAllByText, findByRole } = render(ProgressPage as never);

		await fireEvent.click(await findByRole('tab', { name: 'Exercise' }));

		expect((await findAllByText('progress down')).length).toBeGreaterThan(0);
	});
});
