import { fireEvent, render, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const gotoMock = vi.fn(async () => undefined);
const authStateStore = { subscribe: (run: (value: unknown) => void) => (run({
	status: 'authenticated',
	user: { id: 1, username: 'admin', role: 'admin', must_change_password: false },
	offline: false
}), () => undefined) };

const apiMocks = vi.hoisted(() => ({
	getWorkout: vi.fn(async () => ({
		workout: {
			id: 7,
			date: '2026-01-01T10:00:00.000Z',
			start_time: '2026-01-01T10:15:00.000Z',
			end_time: '2026-01-01T11:15:00.000Z',
			notes: 'Updated',
			feedback: '😐',
			auto_closed_at: null
		},
		exercises: []
	})),
	updateWorkoutTimes: vi.fn(async () => undefined),
	cancelWorkout: vi.fn(async () => undefined)
}));

vi.mock('$app/navigation', () => ({
	goto: gotoMock
}));

vi.mock('$lib/auth', () => ({
	auth: {
		state: authStateStore
	}
}));

vi.mock('$lib/api', () => ({
	cancelWorkout: apiMocks.cancelWorkout,
	getWorkout: apiMocks.getWorkout,
	updateWorkoutTimes: apiMocks.updateWorkoutTimes
}));

beforeEach(() => {
	vi.clearAllMocks();
	vi.stubGlobal('confirm', vi.fn(() => true));
});

describe('workout detail route page', () => {
	it('renders error state from loader data', async () => {
		const { default: WorkoutDetailPage } = await import('../routes/workouts/[id]/+page.svelte');
		const { getByRole, getByText } = render(
			WorkoutDetailPage as never,
			{
				props: { data: { workout: null, error: 'Not found' } }
			} as never
		);

		expect(getByText('Workout')).toBeInTheDocument();
		expect(getByText('Not found')).toBeInTheDocument();
		expect(getByRole('link', { name: /back to history/i })).toBeInTheDocument();
	});

	it('renders workout content when workout data exists', async () => {
		const { default: WorkoutDetailPage } = await import('../routes/workouts/[id]/+page.svelte');
		const { getByText } = render(
			WorkoutDetailPage as never,
			{
				props: {
					data: {
						workout: {
							id: 7,
							date: '2026-01-01T10:00:00.000Z',
							start_time: '2026-01-01T10:00:00.000Z',
							end_time: '2026-01-01T11:00:00.000Z',
							notes: 'Strong day',
							feedback: '😊',
							auto_closed_at: null,
							exercises: [
								{
									exercise: {
										id: 20,
										workout_id: 7,
										exercise_type: 'Bench Press',
										start_time: '2026-01-01T10:00:00.000Z',
										end_time: '2026-01-01T10:20:00.000Z',
										notes: 'Paused',
										per_side_weight: false,
										split_weight: false,
										settings: []
									},
									sets: [{ id: 1, exercise_id: 20, reps: 10, weight: 60 }]
								}
							]
						},
						error: null
					}
				}
			} as never
		);

		expect(getByText('Workout details')).toBeInTheDocument();
		expect(getByText('Bench Press')).toBeInTheDocument();
		expect(getByText('600 kg')).toBeInTheDocument();
	});

	it('supports editing times and deleting from the workout detail page', async () => {
		const { default: WorkoutDetailPage } = await import('../routes/workouts/[id]/+page.svelte');
		const view = render(
			WorkoutDetailPage as never,
			{
				props: {
					data: {
						workout: {
							id: 7,
							date: '2026-01-01T10:00:00.000Z',
							start_time: '2026-01-01T10:00:00.000Z',
							end_time: '2026-01-01T11:00:00.000Z',
							notes: 'Strong day',
							feedback: '😊',
							auto_closed_at: null,
							exercises: []
						},
						error: null
					}
				}
			} as never
		);

		await fireEvent.click(view.getByText('Edit times'));
		await fireEvent.click(view.getByText('Save'));
		await waitFor(() => expect(apiMocks.updateWorkoutTimes).toHaveBeenCalledWith(7, expect.any(Object)));
		await waitFor(() => expect(apiMocks.getWorkout).toHaveBeenCalledWith(7));

		await fireEvent.click(view.getByText('Delete'));
		await waitFor(() => expect(apiMocks.cancelWorkout).toHaveBeenCalledWith(7));
		await waitFor(() => expect(gotoMock).toHaveBeenCalledWith('/workouts'));
	});
});
