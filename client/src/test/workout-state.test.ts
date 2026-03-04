import { beforeEach, describe, expect, it } from 'vitest';
import {
	clearWorkoutState,
	CURRENT_WORKOUT_ID_KEY,
	loadWorkoutState,
	replaceExerciseId,
	replaceSetId,
	replaceWorkoutId,
	saveWorkoutState
} from '$lib/workoutState';

describe('workout state remapping', () => {
	beforeEach(() => {
		localStorage.clear();
	});

	it('replaceWorkoutId updates workout and exercise workout_id references', () => {
		saveWorkoutState({
			workout: {
				id: 11,
				date: '2026-01-01T00:00:00.000Z',
				start_time: '2026-01-01T10:00:00.000Z',
				end_time: '2026-01-01T11:00:00.000Z'
			},
			exercises: [
				{
					exercise: {
						id: 21,
						workout_id: 11,
						exercise_type: 'Bench Press',
						start_time: '2026-01-01T10:00:00.000Z',
						end_time: '2026-01-01T10:20:00.000Z'
					},
					sets: []
				}
			],
			activeExerciseId: 21,
			sessionNotes: '',
			sessionFeedback: null
		});
		localStorage.setItem(CURRENT_WORKOUT_ID_KEY, '11');

		replaceWorkoutId(11, 99);
		const state = loadWorkoutState();

		expect(state?.workout?.id).toBe(99);
		expect(state?.exercises[0]?.exercise.workout_id).toBe(99);
		expect(localStorage.getItem(CURRENT_WORKOUT_ID_KEY)).toBe('99');
	});

	it('replaceExerciseId updates activeExerciseId and set.exercise_id', () => {
		saveWorkoutState({
			workout: null,
			exercises: [
				{
					exercise: {
						id: 33,
						workout_id: 1,
						exercise_type: 'Squat',
						start_time: '2026-01-01T10:00:00.000Z',
						end_time: '2026-01-01T10:20:00.000Z'
					},
					sets: [{ id: 501, exercise_id: 33, reps: 5, weight: 100 }]
				}
			],
			activeExerciseId: 33,
			sessionNotes: '',
			sessionFeedback: null
		});

		replaceExerciseId(33, 44);
		const state = loadWorkoutState();

		expect(state?.activeExerciseId).toBe(44);
		expect(state?.exercises[0]?.exercise.id).toBe(44);
		expect(state?.exercises[0]?.sets[0]?.exercise_id).toBe(44);
	});

	it('replaceSetId only updates the targeted set id', () => {
		saveWorkoutState({
			workout: null,
			exercises: [
				{
					exercise: {
						id: 7,
						workout_id: 1,
						exercise_type: 'Row',
						start_time: '2026-01-01T10:00:00.000Z',
						end_time: '2026-01-01T10:20:00.000Z'
					},
					sets: [
						{ id: 1, exercise_id: 7, reps: 8, weight: 50 },
						{ id: 2, exercise_id: 7, reps: 6, weight: 60 }
					]
				}
			],
			activeExerciseId: 7,
			sessionNotes: '',
			sessionFeedback: null
		});

		replaceSetId(7, 2, 99);
		const state = loadWorkoutState();

		expect(state?.exercises[0]?.sets.map((s) => s.id)).toEqual([1, 99]);
	});

	it('no-op remaps keep stored state untouched', () => {
		saveWorkoutState({
			workout: null,
			exercises: [],
			activeExerciseId: null,
			sessionNotes: 'keep',
			sessionFeedback: null
		});

		replaceWorkoutId(1, 2);
		replaceExerciseId(1, 2);
		replaceSetId(1, 1, 2);

		expect(loadWorkoutState()?.sessionNotes).toBe('keep');
		clearWorkoutState();
		expect(loadWorkoutState()).toBeNull();
	});
});
