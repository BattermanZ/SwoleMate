import type { PageLoad } from './$types';
import { getWorkouts } from '$lib/api';

export const load: PageLoad = async () => {
	try {
		const workouts = await getWorkouts();
		return {
			workouts
		};
	} catch {
		return {
			workouts: []
		};
	}
};
