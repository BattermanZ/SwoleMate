import { get } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const apiMocks = vi.hoisted(() => ({
	getWorkouts: vi.fn(),
	getWorkout: vi.fn()
}));

vi.mock('$lib/api', () => apiMocks);

const offlineMocks = vi.hoisted(() => ({
	findInProgressOffline: vi.fn(async () => null as unknown),
	getRecentSessionsStorageKey: vi.fn(() => 'recent'),
	hydrateOfflineState: vi.fn(async () => undefined),
	refreshPendingSyncCount: vi.fn(async () => 0),
	setOffline: vi.fn()
}));

vi.mock('$lib/today/controller/offline', () => offlineMocks);

vi.mock('$lib/offline/storage', () => ({ kvSet: vi.fn(async () => undefined) }));
vi.mock('$lib/offline/todaySessions', () => ({ deleteOfflineSession: vi.fn(async () => undefined) }));
vi.mock('$lib/today/controller/actions/plannedTemplate', () => ({
	clearPlannedTemplate: vi.fn(async () => undefined),
	loadPlannedTemplate: vi.fn(async () => [])
}));

import { refreshFromBackend } from '$lib/today/controller/actions/backend';
import { createTodayState } from '$lib/today/controller/state';

const ACTIVE = {
	id: 10,
	date: '2026-06-19T10:00:00.000Z',
	start_time: '2026-06-19T10:00:00.000Z',
	end_time: '2026-06-19T10:00:00.000Z'
};

const serverWorkout = {
	workout: { ...ACTIVE, notes: '', feedback: null },
	exercises: [
		{
			exercise: {
				id: 7,
				workout_id: 10,
				exercise_type: 'Bench',
				start_time: '2026-06-19T10:01:00.000Z',
				end_time: '2026-06-19T10:01:00.000Z',
				notes: null,
				per_side_weight: false,
				split_weight: false,
				settings: []
			},
			// Server only knows about one set; the offline edit added a second.
			sets: [{ id: 100, exercise_id: 7, reps: 5, weight: 100, notes: null }]
		}
	]
};

describe('refreshFromBackend with unsynced offline edits (audit #3 C)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		apiMocks.getWorkouts.mockResolvedValue([ACTIVE]);
		apiMocks.getWorkout.mockResolvedValue(serverWorkout);
	});

	it('prefers offline edits over the server copy for the active workout', async () => {
		offlineMocks.findInProgressOffline.mockResolvedValue({
			key: 'offline.today.session.10',
			status: 'in_progress',
			server_workout_id: 10,
			session: {
				id: 10,
				startedAt: ACTIVE.start_time,
				endedAt: ACTIVE.start_time,
				notes: '',
				timezoneOffsetMinutes: 0,
				exercises: [
					{
						id: 7,
						name: 'Bench',
						startedAt: '2026-06-19T10:01:00.000Z',
						endedAt: '2026-06-19T10:01:00.000Z',
						notes: '',
						perSideWeight: false,
						splitWeight: false,
						settings: [],
						tracksReps: true,
						tracksTime: false,
						tracksWeight: true,
						status: 'in_progress',
						sets: [
							{ id: 100, reps: 5, weight: 100 },
							{ id: -1, reps: 5, weight: 105 }
						]
					}
				]
			}
		});

		const state = createTodayState();
		await refreshFromBackend(state);

		const session = get(state.currentSession);
		expect(session?.id).toBe(10);
		// The offline-added second set must survive the refresh.
		expect(session?.exercises[0]?.sets).toHaveLength(2);
	});

	it('uses the server copy when there is no offline record', async () => {
		offlineMocks.findInProgressOffline.mockResolvedValue(null);

		const state = createTodayState();
		await refreshFromBackend(state);

		const session = get(state.currentSession);
		expect(session?.id).toBe(10);
		expect(session?.exercises[0]?.sets).toHaveLength(1);
	});
});
