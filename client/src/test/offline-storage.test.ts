import { beforeEach, describe, expect, it, vi } from 'vitest';

describe('offline storage fallback', () => {
	beforeEach(() => {
		localStorage.clear();
		vi.unstubAllGlobals();
	});

	it('stores and retrieves values via localStorage when indexedDB is unavailable', async () => {
		vi.stubGlobal('indexedDB', undefined);
		const { kvGet, kvSet, kvDelete } = await import('$lib/offline/storage');

		await kvSet('offline.today.session.1', { status: 'in_progress' });
		expect(await kvGet<{ status: string }>('offline.today.session.1')).toEqual({
			status: 'in_progress'
		});

		await kvDelete('offline.today.session.1');
		expect(await kvGet('offline.today.session.1')).toBeNull();
	});

	it('lists only prefix keys and returns them sorted in fallback mode', async () => {
		vi.stubGlobal('indexedDB', undefined);
		const { kvListKeys, kvSet } = await import('$lib/offline/storage');

		await kvSet('offline.today.session.20', { id: 20 });
		await kvSet('offline.today.session.3', { id: 3 });
		await kvSet('something.else', { id: 99 });

		expect(await kvListKeys('offline.today.session.')).toEqual([
			'offline.today.session.20',
			'offline.today.session.3'
		]);
	});

	it('throws on invalid fallback JSON payloads', async () => {
		vi.stubGlobal('indexedDB', undefined);
		const { kvGet } = await import('$lib/offline/storage');
		localStorage.setItem('swolemate:broken', '{bad-json');

		await expect(kvGet('broken')).rejects.toBeInstanceOf(SyntaxError);
	});
});
