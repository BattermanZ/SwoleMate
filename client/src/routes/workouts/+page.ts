import type { PageLoad } from './$types';
import { getWorkouts } from '$lib/api';
import type { Workout } from '$lib/types';

export const load: PageLoad = async () => {
    try {
        const workouts = await getWorkouts();
        return {
            workouts
        };
    } catch (error) {
        return {
            workouts: []
        };
    }
}; 