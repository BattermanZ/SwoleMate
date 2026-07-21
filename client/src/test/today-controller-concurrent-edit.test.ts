import { get } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createTodayState } from '$lib/today/controller/state';
import { createExerciseSetActions } from '$lib/today/controller/actions/exercise/sets';

const apiMocks = vi.hoisted(() => ({
	createSet: vi.fn(),
	endExercise: vi.fn(),
	replaceSets: vi.fn()
}));

vi.mock('$lib/api', () => ({
	createSet: apiMocks.createSet,
	endExercise: apiMocks.endExercise,
	replaceSets: apiMocks.replaceSets
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

function exercise(id: number, name: string) {
	return {
		id,
		name,
		notes: '',
		startedAt: '2026-01-01T10:00:00.000Z',
		endedAt: '2026-01-01T10:05:00.000Z',
		status: 'active' as const,
		perSideWeight: false,
		splitWeight: false,
		settings: [],
		sets: []
	};
}

// F-HIGH-1: an online reducer must not overwrite the store with a pre-await
// snapshot — a concurrent edit landing during the in-flight network call must
// survive.
describe('today controller concurrent-edit safety (F-HIGH-1)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('preserves a concurrent edit made during an in-flight addSet', async () => {
		let resolveCreate: (value: { id: number }) => void = () => {};
		apiMocks.createSet.mockReturnValueOnce(
			new Promise<{ id: number }>((resolve) => {
				resolveCreate = resolve;
			})
		);

		const state = createTodayState();
		state.currentSession.set({
			id: 5,
			startedAt: '2026-01-01T10:00:00.000Z',
			notes: '',
			exercises: [exercise(9, 'Squat'), exercise(10, 'Bench')]
		});

		const actions = createExerciseSetActions({ state });

		// Start an online addSet on exercise 9 — it blocks on createSet.
		const inFlight = actions.addSet(9, 8, 100);

		// While it is in flight, a concurrent edit changes exercise 10's notes.
		state.currentSession.update((current) => ({
			...current!,
			exercises: current!.exercises.map((e) =>
				e.id === 10 ? { ...e, notes: 'concurrent edit' } : e
			)
		}));

		// The network call now resolves and the reducer commits.
		resolveCreate({ id: 88 });
		await inFlight;

		const session = get(state.currentSession)!;
		const squat = session.exercises.find((e) => e.id === 9)!;
		const bench = session.exercises.find((e) => e.id === 10)!;

		// The new set landed...
		expect(squat.sets).toHaveLength(1);
		// ...and the concurrent edit was NOT clobbered by the stale snapshot.
		expect(bench.notes).toBe('concurrent edit');
	});
});
