import { beforeEach, describe, expect, it, vi } from 'vitest';

const storageMocks = vi.hoisted(() => ({
	kvSet: vi.fn(async () => undefined),
	kvGet: vi.fn<(key: string) => Promise<{ key: string; updated_at: string } | null>>(
		async () => null
	),
	kvDelete: vi.fn(async () => undefined),
	kvListKeys: vi.fn<(prefix: string) => Promise<string[]>>(async () => [])
}));

vi.mock('$lib/offline/storage', () => storageMocks);

describe('offline today sessions', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		localStorage.clear();
	});

	it('builds user-scoped keys, and an explicit anon namespace when no user (F-LOW-2)', async () => {
		const { sessionKeyForId } = await import('$lib/offline/todaySessions');

		// No active user: keys are namespaced under `anon:` (never the bare base
		// key), so they can't collide with a real user's `u<id>:` scoped reads.
		expect(sessionKeyForId(42)).toBe('anon:offline.today.session.42');

		// With an active user, keys are scoped to that user.
		localStorage.setItem('auth.activeUserId', '7');
		expect(sessionKeyForId(42)).toBe('u7:offline.today.session.42');
	});

	it('filters null entries and sorts records by updated_at descending', async () => {
		const { listOfflineSessions } = await import('$lib/offline/todaySessions');

		storageMocks.kvListKeys.mockResolvedValueOnce([
			'offline.today.session.1',
			'offline.today.session.2',
			'offline.today.session.3'
		]);

		storageMocks.kvGet.mockImplementation(async (key: string) => {
			if (key.endsWith('.1')) return { key, updated_at: '2026-01-01T10:00:00.000Z' };
			if (key.endsWith('.2')) return null;
			if (key.endsWith('.3')) return { key, updated_at: '2026-01-01T11:00:00.000Z' };
			return null;
		});

		const records = await listOfflineSessions();
		expect(records).toHaveLength(2);
		expect(records.map((r) => r.key)).toEqual([
			'offline.today.session.3',
			'offline.today.session.1'
		]);
	});
});
