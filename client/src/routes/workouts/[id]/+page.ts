import type { PageLoad } from './$types';
import { getWorkout } from '$lib/api';
import { logger } from '$lib/logger';
import type { WorkoutWithExercises } from '$lib/types';

export const load: PageLoad = async ({ fetch, params }) => {
	const workoutId = Number(params.id);
	if (!Number.isFinite(workoutId)) {
		return { workout: null as WorkoutWithExercises | null, error: 'Invalid workout ID' };
	}

	try {
		const data = await getWorkout(workoutId, fetch);
		const workout: WorkoutWithExercises = { ...data.workout, exercises: data.exercises };
		return { workout, error: null as string | null };
	} catch (error) {
		logger.error('workout', 'Failed to load workout', { error, workoutId });
		return {
			workout: null as WorkoutWithExercises | null,
			error: error instanceof Error ? error.message : 'Failed to load workout'
		};
	}
};
