import { beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import type { UiSession } from '$lib/today/types';
import { createTodayController } from '$lib/today/controller';
import { createExercise, getLastExerciseData } from '$lib/api';

vi.mock('$lib/api', () => ({
	cancelExercise: vi.fn(),
	cancelWorkout: vi.fn(),
	createExercise: vi.fn(async () => ({ id: 123 })),
	createSet: vi.fn(),
	createWorkout: vi.fn(),
	endExercise: vi.fn(),
	endWorkout: vi.fn(),
	getExerciseTypes: vi.fn(),
	getLastExerciseData: vi.fn(),
	getWorkout: vi.fn(),
	getWorkouts: vi.fn(),
	replaceSets: vi.fn()
}));

const lastExercisePayload = {
	exercise: {
		id: 500,
		user_id: 1,
		workout_id: 50,
		exercise_type: 'Shoulder press',
		start_time: '2025-12-01T09:00:00.000Z',
		end_time: '2025-12-01T09:20:00.000Z',
		notes: 'Kept core tight, slow eccentric.',
		per_side_weight: true,
		split_weight: false,
		settings: [
			{ id: 1, exercise_id: 500, key: 'Seat height', value: '4' },
			{ id: 2, exercise_id: 500, key: '_tracking_fields', value: 'reps,weight' }
		]
	},
	sets: [{ id: 900, exercise_id: 500, reps: 10, weight: 25 }]
};

describe('today controller last-time fetching', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('fetches the last session for an exercise missing from the recent-sessions cache', async () => {
		vi.mocked(getLastExerciseData).mockResolvedValue(lastExercisePayload);
		const controller = createTodayController();

		// recent sessions does NOT contain Shoulder press
		controller.recentSessions.set([]);

		expect(controller.getLastTimeForExercise('Shoulder press')).toBeUndefined();

		await controller.loadLastTimeForExercise('Shoulder press');

		expect(getLastExerciseData).toHaveBeenCalledWith('Shoulder press', {
			excludeWorkoutId: undefined
		});
		const cached = get(controller.lastTimeByExercise)['Shoulder press'];
		expect(cached).toMatchObject({
			notes: 'Kept core tight, slow eccentric.',
			perSideWeight: true,
			splitWeight: false,
			tracksReps: true,
			tracksWeight: true
		});
		expect(cached?.sets).toEqual([{ id: 900, reps: 10, weight: 25 }]);
		expect(cached?.settings.map((s) => `${s.key}:${s.value}`)).toEqual(['Seat height:4']);
	});

	it('caches a null result and does not refetch when an exercise has no history', async () => {
		vi.mocked(getLastExerciseData).mockResolvedValue(null);
		const controller = createTodayController();
		controller.recentSessions.set([]);

		await controller.loadLastTimeForExercise('Brand New Move');
		await controller.loadLastTimeForExercise('Brand New Move');

		expect(getLastExerciseData).toHaveBeenCalledTimes(1);
		expect(get(controller.lastTimeByExercise)['Brand New Move']).toBeNull();
	});

	it('does not fetch when the exercise is already in the recent-sessions cache', async () => {
		const controller = createTodayController();
		controller.recentSessions.set([
			{
				id: 1,
				startedAt: '2025-12-20T10:00:00.000Z',
				endedAt: '2025-12-20T11:00:00.000Z',
				notes: '',
				exercises: [
					{
						id: 10,
						name: 'Bench Press',
						notes: '',
						startedAt: '2025-12-20T10:00:00.000Z',
						endedAt: '2025-12-20T10:30:00.000Z',
						status: 'done',
						perSideWeight: false,
						splitWeight: false,
						sets: [{ id: 20, reps: 8, weight: 30 }],
						settings: []
					}
				]
			}
		]);

		await controller.loadLastTimeForExercise('Bench Press');

		expect(getLastExerciseData).not.toHaveBeenCalled();
	});

	it('carries over fetched settings and weight modes when adding a searched exercise', async () => {
		vi.mocked(getLastExerciseData).mockResolvedValue(lastExercisePayload);
		const controller = createTodayController();

		const session: UiSession = {
			id: 99,
			startedAt: '2026-01-02T10:00:00.000Z',
			notes: '',
			exercises: []
		};
		controller.currentSession.set(session);
		controller.recentSessions.set([]);

		await controller.addExercise('Shoulder press');

		expect(getLastExerciseData).toHaveBeenCalledWith('Shoulder press', { excludeWorkoutId: 99 });
		expect(createExercise).toHaveBeenCalledWith(
			99,
			expect.objectContaining({
				exercise_type: 'Shoulder press',
				per_side_weight: true,
				split_weight: false,
				settings: expect.arrayContaining([
					{ key: 'Seat height', value: '4' },
					{ key: '_tracking_fields', value: 'reps,weight' }
				])
			})
		);
	});
});
