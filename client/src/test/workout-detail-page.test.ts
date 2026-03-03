import { describe, expect, it } from 'vitest';
import { render } from '@testing-library/svelte';

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
});
