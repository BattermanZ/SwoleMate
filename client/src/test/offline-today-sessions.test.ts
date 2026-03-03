import { beforeEach, describe, expect, it, vi } from 'vitest';

const storageMocks = vi.hoisted(() => ({
	kvSet: vi.fn(async () => undefined),
	kvGet: vi.fn<(key: string) => Promise<{ key: string; updated_at: string } | null>>(
		async (_key: string) => null
	),
	kvDelete: vi.fn(async () => undefined),
	kvListKeys: vi.fn<(prefix: string) => Promise<string[]>>(async (_prefix: string) => [])
}));

vi.mock('$lib/offline/storage', () => storageMocks);

describe('offline today sessions', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('builds stable prefixed keys', async () => {
		const { sessionKeyForId } = await import('$lib/offline/todaySessions');
		expect(sessionKeyForId(42)).toBe('offline.today.session.42');
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
