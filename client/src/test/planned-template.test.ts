import { describe, expect, it, vi } from 'vitest';
import { loadPlannedTemplate } from '$lib/today/controller/actions/plannedTemplate';

const storageMocks = vi.hoisted(() => ({
	kvDelete: vi.fn(),
	kvGet: vi.fn(),
	kvSet: vi.fn()
}));

vi.mock('$lib/offline/storage', () => ({
	kvDelete: storageMocks.kvDelete,
	kvGet: storageMocks.kvGet,
	kvSet: storageMocks.kvSet
}));

describe('planned template storage', () => {
	it('preserves exercise notes when restoring a planned template', async () => {
		storageMocks.kvGet.mockResolvedValueOnce({
			sessionId: 42,
			exercises: [
				{
					id: 7,
					name: 'Bench Press',
					notes: 'controlled eccentric',
					perSideWeight: false,
					splitWeight: false,
					settings: []
				}
			]
		});

		await expect(loadPlannedTemplate(42)).resolves.toEqual([
			{
				id: 7,
				name: 'Bench Press',
				notes: 'controlled eccentric',
				perSideWeight: false,
				splitWeight: false,
				settings: []
			}
		]);
	});
});
