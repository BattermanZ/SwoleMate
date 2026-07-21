import { beforeEach, describe, expect, it, vi } from 'vitest';

// F-LOW-2: an offline session must never be persisted while the active user id is
// unknown — it would land in the unscoped `anon:` bucket, be orphaned once the id
// resolves, and be readable by another account on a shared device.
describe('saveOfflineSession user-scope guard (F-LOW-2)', () => {
	beforeEach(() => {
		vi.resetModules();
		vi.clearAllMocks();
	});

	const setup = async (hasScope: boolean) => {
		const kvSet = vi.fn(async () => undefined);
		vi.doMock('$lib/offline/storage', () => ({
			kvSet,
			kvGet: async () => null,
			kvDelete: async () => undefined,
			kvListKeys: async () => []
		}));
		vi.doMock('$lib/auth/scope', () => ({
			hasActiveUserScope: () => hasScope,
			scopedKey: (base: string) => (hasScope ? `u7:${base}` : `anon:${base}`)
		}));
		const mod = await import('$lib/offline/todaySessions');
		return { mod, kvSet };
	};

	const record = () =>
		({
			key: 'offline.today.session.1',
			status: 'in_progress' as const,
			updated_at: '2026-01-01T00:00:00.000Z',
			session: { id: 1, startedAt: '2026-01-01T00:00:00.000Z', notes: '', exercises: [] }
		}) as Parameters<
			Awaited<ReturnType<typeof setup>>['mod']['saveOfflineSession']
		>[0];

	it('skips the write when no active user scope is known', async () => {
		const { mod, kvSet } = await setup(false);
		await mod.saveOfflineSession(record());
		expect(kvSet).not.toHaveBeenCalled();
	});

	it('persists normally once a user scope is known', async () => {
		const { mod, kvSet } = await setup(true);
		await mod.saveOfflineSession(record());
		expect(kvSet).toHaveBeenCalledTimes(1);
	});
});
