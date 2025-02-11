import type { PageLoad } from './$types';
import { getBackups } from '$lib/api';

export const load: PageLoad = async () => {
    try {
        const backups = await getBackups();
        return {
            backups
        };
    } catch (error) {
        console.error('Failed to load backups:', error);
        return {
            backups: []
        };
    }
}; 