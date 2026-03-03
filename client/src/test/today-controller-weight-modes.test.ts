import { get } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createTodayState } from '$lib/today/controller/state';
import { createExerciseWeightModeActions } from '$lib/today/controller/actions/exercise/weightModes';

const apiMocks = vi.hoisted(() => ({
	endExercise: vi.fn(),
	replaceSets: vi.fn()
}));

vi.mock('$lib/api', () => ({
	endExercise: apiMocks.endExercise,
	replaceSets: apiMocks.replaceSets
}));

const offlineMocks = vi.hoisted(() => ({
	persistInProgressSession: vi.fn(),
	setOffline: vi.fn((access: { offlineMode: { set: (value: boolean) => void } }) => {
		access.offlineMode.set(true);
	})
}));

vi.mock('$lib/today/controller/offline', () => ({
	persistInProgressSession: offlineMocks.persistInProgressSession,
	setOffline: offlineMocks.setOffline
}));

describe('today controller weight mode actions', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('converts split per-side sets to total weight when disabling per-side mode', async () => {
		apiMocks.replaceSets.mockResolvedValueOnce([
			{ id: 70, reps: 6, weight: 37, weight_left: null, weight_right: null }
		]);

		const state = createTodayState();
		state.currentSession.set({
			id: 11,
			startedAt: '2026-01-01T10:00:00.000Z',
			notes: '',
			exercises: [
				{
					id: 21,
					name: 'Incline DB Press',
					notes: '',
					startedAt: '2026-01-01T10:00:00.000Z',
					endedAt: '2026-01-01T10:05:00.000Z',
					status: 'active',
					perSideWeight: true,
					splitWeight: true,
					settings: [],
					sets: [{ id: 1, reps: 6, weight: 20, weightLeft: 17, weightRight: 20 }]
				}
			]
		});

		const refreshFromBackend = vi.fn(async () => undefined);
		const actions = createExerciseWeightModeActions({ state, refreshFromBackend });
		await actions.toggleExercisePerSideWeight(21, false);

		const exercise = get(state.currentSession)!.exercises[0]!;
		expect(exercise.perSideWeight).toBe(false);
		expect(exercise.splitWeight).toBe(false);
		expect(exercise.sets[0]).toMatchObject({ id: 70, weight: 37, weightLeft: undefined, weightRight: undefined });
		expect(apiMocks.endExercise).toHaveBeenCalledWith(
			21,
			expect.objectContaining({ per_side_weight: false, split_weight: false })
		);
		expect(apiMocks.replaceSets).toHaveBeenCalledTimes(1);
	});

	it('persists only locally when toggling split mode offline', async () => {
		const state = createTodayState();
		state.offlineMode.set(true);
		state.currentSession.set({
			id: -1,
			startedAt: '2026-01-01T10:00:00.000Z',
			notes: '',
			exercises: [
				{
					id: -5,
					name: 'Cable Row',
					notes: '',
					startedAt: '2026-01-01T10:00:00.000Z',
					endedAt: '2026-01-01T10:05:00.000Z',
					status: 'active',
					perSideWeight: true,
					splitWeight: false,
					settings: [],
					sets: [{ id: -9, reps: 10, weight: 25 }]
				}
			]
		});

		const refreshFromBackend = vi.fn(async () => undefined);
		const actions = createExerciseWeightModeActions({ state, refreshFromBackend });
		await actions.toggleExerciseSplitWeight(-5, true);

		const exercise = get(state.currentSession)!.exercises[0]!;
		expect(exercise.splitWeight).toBe(true);
		expect(exercise.sets[0]).toMatchObject({ weightLeft: 25, weightRight: 25 });
		expect(offlineMocks.persistInProgressSession).toHaveBeenCalledTimes(1);
		expect(apiMocks.endExercise).not.toHaveBeenCalled();
		expect(apiMocks.replaceSets).not.toHaveBeenCalled();
	});

	it('switches to offline mode and persists when online update hits network failure', async () => {
		apiMocks.endExercise.mockRejectedValueOnce(new TypeError('Failed to fetch'));

		const state = createTodayState();
		state.currentSession.set({
			id: 15,
			startedAt: '2026-01-01T10:00:00.000Z',
			notes: '',
			exercises: [
				{
					id: 25,
					name: 'Lat Pulldown',
					notes: '',
					startedAt: '2026-01-01T10:00:00.000Z',
					endedAt: '2026-01-01T10:05:00.000Z',
					status: 'active',
					perSideWeight: false,
					splitWeight: false,
					settings: [],
					sets: [{ id: 1, reps: 8, weight: 50 }]
				}
			]
		});

		const refreshFromBackend = vi.fn(async () => undefined);
		const actions = createExerciseWeightModeActions({ state, refreshFromBackend });
		await actions.toggleExercisePerSideWeight(25, true);

		expect(offlineMocks.setOffline).toHaveBeenCalledTimes(1);
		expect(offlineMocks.persistInProgressSession).toHaveBeenCalledTimes(1);
		expect(refreshFromBackend).not.toHaveBeenCalled();
	});

	it('sets error and refreshes backend for non-network failures', async () => {
		apiMocks.endExercise.mockRejectedValueOnce(new Error('bad request'));

		const state = createTodayState();
		state.currentSession.set({
			id: 15,
			startedAt: '2026-01-01T10:00:00.000Z',
			notes: '',
			exercises: [
				{
					id: 25,
					name: 'Lat Pulldown',
					notes: '',
					startedAt: '2026-01-01T10:00:00.000Z',
					endedAt: '2026-01-01T10:05:00.000Z',
					status: 'active',
					perSideWeight: false,
					splitWeight: false,
					settings: [],
					sets: [{ id: 1, reps: 8, weight: 50 }]
				}
			]
		});

		const refreshFromBackend = vi.fn(async () => undefined);
		const actions = createExerciseWeightModeActions({ state, refreshFromBackend });
		await actions.toggleExercisePerSideWeight(25, true);

		expect(offlineMocks.setOffline).not.toHaveBeenCalled();
		expect(get(state.error)).toBe('bad request');
		expect(refreshFromBackend).toHaveBeenCalledTimes(1);
	});
});
