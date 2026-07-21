import { beforeEach, describe, expect, it, vi } from 'vitest';

// F-MED-3: overlapping persistInProgressSession calls must not clobber each
// other's merge-only fields (deleted ids). We back the offline store with an
// in-memory map whose load/save yield to the event loop, so an un-serialized
// implementation would interleave load-then-save and lose a deletion.
describe('persistInProgressSession serialization (F-MED-3)', () => {
	beforeEach(() => {
		vi.resetModules();
	});

	it('preserves a concurrent deleted-id merge against a plain persist', async () => {
		const store = new Map<string, unknown>();
		const tick = () => new Promise((r) => setTimeout(r, 0));

		vi.doMock('$lib/offline/todaySessions', () => ({
			sessionKeyForId: (id: number) => `k${id}`,
			loadOfflineSession: async (key: string) => {
				await tick();
				return store.get(key) ?? null;
			},
			saveOfflineSession: async (rec: { key: string }) => {
				await tick();
				store.set(rec.key, rec);
			},
			listOfflineSessions: async () => [...store.values()]
		}));
		vi.doMock('$lib/offline/storage', () => ({
			kvGet: async () => null
		}));

		const { persistInProgressSession } = await import('$lib/today/controller/offline');
		const { writable } = await import('svelte/store');

		const currentSession = writable({
			id: 5,
			startedAt: '2026-01-01T10:00:00.000Z',
			notes: '',
			exercises: []
		});
		const pendingSyncCount = writable(0);
		const access = { currentSession, pendingSyncCount } as never;

		// A plain persist and a deletion persist fire concurrently.
		await Promise.all([
			persistInProgressSession(access),
			persistInProgressSession(access, (existing: { deleted_server_exercise_ids?: number[] } | null) => ({
				deleted_server_exercise_ids: [...(existing?.deleted_server_exercise_ids ?? []), 99]
			}))
		]);

		const saved = store.get('k5') as { deleted_server_exercise_ids: number[] };
		expect(saved.deleted_server_exercise_ids).toContain(99);
	});
});

describe('kvSet surfaces storage write failures (F-MED-4)', () => {
	beforeEach(() => {
		vi.resetModules();
		vi.doUnmock('$lib/offline/storage');
		vi.doUnmock('$lib/offline/todaySessions');
		vi.unstubAllGlobals();
	});

	it('throws a StorageWriteError (quota) when the localStorage fallback is full', async () => {
		vi.stubGlobal('indexedDB', undefined);
		const { kvSet, isStorageWriteError } = await import('$lib/offline/storage');

		const spy = vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
			const err = new DOMException('The quota has been exceeded.', 'QuotaExceededError');
			throw err;
		});

		let caught: unknown;
		try {
			await kvSet('offline.today.session.1', { a: 1 });
		} catch (e) {
			caught = e;
		}

		expect(isStorageWriteError(caught)).toBe(true);
		expect((caught as { quotaExceeded: boolean }).quotaExceeded).toBe(true);
		spy.mockRestore();
	});
});
