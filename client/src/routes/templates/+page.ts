import type { PageLoad } from './$types';
import { getWorkoutTemplates } from '$lib/api';
import { logger } from '$lib/logger';

export const load: PageLoad = async ({ fetch }) => {
	try {
		const templates = await getWorkoutTemplates(fetch);
		return { templates };
	} catch (error) {
		logger.error('template', 'Failed to load templates', { error });
		return { templates: [] };
	}
};
