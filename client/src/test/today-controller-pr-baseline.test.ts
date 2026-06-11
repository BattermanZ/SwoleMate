import { beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import type { UiSession } from '$lib/today/types';
import { createTodayController } from '$lib/today/controller';
import { getVolumeStats } from '$lib/api';

vi.mock('$lib/api', () => ({
	getVolumeStats: vi.fn()
}));

const volumeStats = {
	weekly_volume: [],
	monthly_volume: [],
	personal_records: { all_time_max_weight: 0, max_volume: 0, estimated_max_1rm: 25.41 }
};

describe('today PR baseline', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('excludes the active workout so current-session sets are not in the baseline', async () => {
		vi.mocked(getVolumeStats).mockResolvedValue(volumeStats);
		const controller = createTodayController();

		const session: UiSession = {
			id: 77,
			startedAt: '2026-01-02T10:00:00.000Z',
			notes: '',
			exercises: []
		};
		controller.currentSession.set(session);

		await controller.loadEstimated1RmBaseline('Bench Press');

		expect(getVolumeStats).toHaveBeenCalledWith('Bench Press', { excludeWorkoutId: 77 });
		expect(get(controller.estimated1RmBaselines)['Bench Press']).toBe(25.41);
	});

	it('omits the exclusion when there is no active server-backed session', async () => {
		vi.mocked(getVolumeStats).mockResolvedValue(volumeStats);
		const controller = createTodayController();

		await controller.loadEstimated1RmBaseline('Bench Press');

		expect(getVolumeStats).toHaveBeenCalledWith('Bench Press', { excludeWorkoutId: undefined });
	});
});
