import { logger } from '$lib/logger';

/**
 * Ask the browser to move this origin's Cache Storage + IndexedDB out of the
 * best-effort tier into persistent storage, so a workout logged offline isn't
 * evicted under storage pressure (or after iOS Safari's 7-day ITP window for
 * non-installed sites). Safe to call repeatedly: it no-ops once granted.
 *
 * Returns whether storage is persistent after the call.
 */
export async function requestPersistentStorage(): Promise<boolean> {
	try {
		if (typeof navigator === 'undefined' || !navigator.storage?.persist) return false;
		if (await navigator.storage.persisted?.()) return true;
		const granted = await navigator.storage.persist();
		logger.debug('pwa', 'persistent storage', { granted });
		return granted;
	} catch (err) {
		logger.error('pwa', 'persistent storage request failed', { err });
		return false;
	}
}
