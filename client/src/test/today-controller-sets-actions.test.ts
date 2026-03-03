import { get } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createTodayState } from '$lib/today/controller/state';
import { createExerciseSetActions } from '$lib/today/controller/actions/exercise/sets';

const apiMocks = vi.hoisted(() => ({
	createSet: vi.fn(),
	endExercise: vi.fn()
}));

vi.mock('$lib/api', () => ({
	createSet: apiMocks.createSet,
	endExercise: apiMocks.endExercise
}));

const offlineMocks = vi.hoisted(() => ({
	hydrateOfflineState: vi.fn(),
	persistInProgressSession: vi.fn(),
	setOffline: vi.fn((access: { offlineMode: { set: (value: boolean) => void } }) => {
		access.offlineMode.set(true);
	})
}));

vi.mock('$lib/today/controller/offline', () => ({
	hydrateOfflineState: offlineMocks.hydrateOfflineState,
	persistInProgressSession: offlineMocks.persistInProgressSession,
	setOffline: offlineMocks.setOffline
}));

describe('today controller set actions', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('adds set online with server id', async () => {
		apiMocks.createSet.mockResolvedValueOnce({ id: 88 });

		const state = createTodayState();
		state.currentSession.set({
			id: 5,
			startedAt: '2026-01-01T10:00:00.000Z',
			notes: '',
			exercises: [
				{
					id: 9,
					name: 'Squat',
					notes: '',
					startedAt: '2026-01-01T10:00:00.000Z',
					endedAt: '2026-01-01T10:05:00.000Z',
					status: 'active',
					perSideWeight: false,
					splitWeight: false,
					settings: [],
					sets: []
				}
			]
		});

		const actions = createExerciseSetActions({ state });
		await actions.addSet(9, 8, 100);

		expect(apiMocks.createSet).toHaveBeenCalledWith(
			9,
			expect.objectContaining({ reps: 8, weight: 100, notes: undefined })
		);
		expect(get(state.currentSession)!.exercises[0]!.sets[0]).toMatchObject({ id: 88, reps: 8, weight: 100 });
	});

	it('falls back to offline add-set flow on network failure', async () => {
		apiMocks.createSet.mockRejectedValueOnce(new TypeError('Failed to fetch'));

		const state = createTodayState();
		state.currentSession.set({
			id: 6,
			startedAt: '2026-01-01T10:00:00.000Z',
			notes: '',
			exercises: [
				{
					id: 10,
					name: 'Deadlift',
					notes: '',
					startedAt: '2026-01-01T10:00:00.000Z',
					endedAt: '2026-01-01T10:05:00.000Z',
					status: 'active',
					perSideWeight: false,
					splitWeight: false,
					settings: [],
					sets: []
				}
			]
		});

		const actions = createExerciseSetActions({ state });
		await actions.addSet(10, 5, 140);

		expect(offlineMocks.setOffline).toHaveBeenCalledTimes(1);
		expect(offlineMocks.hydrateOfflineState).toHaveBeenCalledTimes(1);
		expect(offlineMocks.persistInProgressSession).toHaveBeenCalled();
		const added = get(state.currentSession)!.exercises[0]!.sets[0]!;
		expect(added.id).toBeLessThan(0);
		expect(added).toMatchObject({ reps: 5, weight: 140 });
	});

	it('marks exercise done offline and collapses open card', async () => {
		const state = createTodayState();
		state.offlineMode.set(true);
		state.openExerciseId.set(-3);
		state.currentSession.set({
			id: -1,
			startedAt: '2026-01-01T10:00:00.000Z',
			notes: '',
			exercises: [
				{
					id: -3,
					name: 'Press',
					notes: '',
					startedAt: '2026-01-01T10:00:00.000Z',
					endedAt: '2026-01-01T10:05:00.000Z',
					status: 'active',
					perSideWeight: false,
					splitWeight: false,
					settings: [],
					sets: []
				}
			]
		});

		const actions = createExerciseSetActions({ state });
		await actions.markExerciseDone(-3);

		expect(apiMocks.endExercise).not.toHaveBeenCalled();
		expect(offlineMocks.persistInProgressSession).toHaveBeenCalledTimes(1);
		expect(get(state.currentSession)!.exercises[0]!.status).toBe('done');
		expect(get(state.openExerciseId)).toBeNull();
	});
});
