import { afterEach, describe, expect, it, vi } from 'vitest';

vi.mock('$lib/logger', () => ({
	logger: { debug: vi.fn(), error: vi.fn() }
}));

import { requestPersistentStorage } from '$lib/pwa/persistentStorage';

describe('requestPersistentStorage', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('returns false when the Storage API is unavailable', async () => {
		vi.stubGlobal('navigator', {});
		expect(await requestPersistentStorage()).toBe(false);
	});

	it('does not re-request when storage is already persisted', async () => {
		const persist = vi.fn(async () => true);
		vi.stubGlobal('navigator', {
			storage: { persisted: vi.fn(async () => true), persist }
		});

		expect(await requestPersistentStorage()).toBe(true);
		expect(persist).not.toHaveBeenCalled();
	});

	it('requests persistence and returns the grant result', async () => {
		const persist = vi.fn(async () => true);
		vi.stubGlobal('navigator', {
			storage: { persisted: vi.fn(async () => false), persist }
		});

		expect(await requestPersistentStorage()).toBe(true);
		expect(persist).toHaveBeenCalledTimes(1);
	});

	it('returns false when the grant is denied', async () => {
		vi.stubGlobal('navigator', {
			storage: { persisted: vi.fn(async () => false), persist: vi.fn(async () => false) }
		});

		expect(await requestPersistentStorage()).toBe(false);
	});
});
