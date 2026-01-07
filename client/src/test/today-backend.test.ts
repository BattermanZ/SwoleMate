import { describe, expect, it } from 'vitest';
import { toUiSession, workoutIsActive } from '$lib/today/backend';

describe('today/backend mapping', () => {
	it('detects active workouts (end_time == start_time)', () => {
		expect(
			workoutIsActive({
				id: 1,
				date: '2026-01-07T10:00:00.000Z',
				start_time: '2026-01-07T10:00:00.000Z',
				end_time: '2026-01-07T10:00:00.000Z'
			})
		).toBe(true);
	});

	it('maps workout+exercises to UiSession with settings and split weights', () => {
		const session = toUiSession(
			{
				id: 10,
				date: '2026-01-07T10:00:00.000Z',
				start_time: '2026-01-07T10:00:00.000Z',
				end_time: '2026-01-07T11:00:00.000Z',
				notes: 'session note',
				feedback: '😊'
			},
			[
				{
					exercise: {
						id: 20,
						workout_id: 10,
						exercise_type: 'Bench Press',
						start_time: '2026-01-07T10:05:00.000Z',
						end_time: '2026-01-07T10:05:00.000Z',
						notes: 'pause',
						per_side_weight: true,
						split_weight: true,
						settings: [
							{ id: 1, exercise_id: 20, key: 'Bench', value: 'Flat' },
							{ id: 2, exercise_id: 20, key: 'Rack height', value: '6' }
						]
					},
					sets: [
						{
							id: 30,
							exercise_id: 20,
							reps: 10,
							weight: 26.25,
							weight_left: 25,
							weight_right: 27.5,
							notes: null
						}
					]
				}
			]
		);

		expect(session.id).toBe(10);
		expect(session.endedAt).toBe('2026-01-07T11:00:00.000Z');
		expect(session.mood).toBe('😊');
		expect(session.exercises[0]?.perSideWeight).toBe(true);
		expect(session.exercises[0]?.splitWeight).toBe(true);
		expect(session.exercises[0]?.settings).toHaveLength(2);
		expect(session.exercises[0]?.sets[0]?.weightLeft).toBe(25);
		expect(session.exercises[0]?.sets[0]?.weightRight).toBe(27.5);
	});
});
