import type { PageLoad } from './$types';
import { getWorkout } from '$lib/api';
import type { WorkoutWithExercises } from '$lib/types';

export const load: PageLoad = async () => {
    try {
        // For now, we'll just fetch the most recent workout (ID: 1)
        // In the future, we can add an endpoint to list all workouts
        const data = await getWorkout(1);
        return {
            workout: {
                ...data.workout,
                exercises: data.exercises
            } as WorkoutWithExercises
        };
    } catch (error) {
        return {
            workout: null
        };
    }
}; 