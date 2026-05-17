import type { PageLoad } from './$types';
import { getWorkouts } from '$lib/api';
import { logger } from '$lib/logger';

export const load: PageLoad = async ({ fetch }) => {
	try {
		const workouts = await getWorkouts(fetch);
		return { workouts };
	} catch (error) {
		logger.error('workout', 'Failed to load workouts', { error });
		return { workouts: [] };
	}
};
