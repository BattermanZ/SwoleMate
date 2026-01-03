import type { PageLoad } from './$types';
import { getBackups } from '$lib/api';
import { logger } from '$lib/logger';

export const load: PageLoad = async ({ fetch }) => {
	try {
		const backups = await getBackups(fetch);
		return {
			backups
		};
	} catch (error) {
		logger.error('backups', 'Failed to load backups', { error });
		return {
			backups: []
		};
	}
};
