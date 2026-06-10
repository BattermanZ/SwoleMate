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
	it('drops legacy exercise notes when restoring a planned template', async () => {
		storageMocks.kvGet.mockResolvedValueOnce({
			sessionId: 42,
			exercises: [
				{
					id: 7,
					name: 'Bench Press',
					notes: 'legacy cue',
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
				perSideWeight: false,
				splitWeight: false,
				settings: []
			}
		]);
	});
});
