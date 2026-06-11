import { get } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createTodayState } from '$lib/today/controller/state';
import { createSessionActions } from '$lib/today/controller/actions/session';

const apiMocks = vi.hoisted(() => ({
	createWorkout: vi.fn(),
	cancelWorkout: vi.fn(),
	endExercise: vi.fn(),
	endWorkout: vi.fn(),
	getWorkoutTemplate: vi.fn()
}));

vi.mock('$lib/api', () => ({
	createWorkout: apiMocks.createWorkout,
	cancelWorkout: apiMocks.cancelWorkout,
	endExercise: apiMocks.endExercise,
	endWorkout: apiMocks.endWorkout,
	getWorkoutTemplate: apiMocks.getWorkoutTemplate
}));

const demoMocks = vi.hoisted(() => ({
	createDemoSession: vi.fn(),
	EXERCISE_LIBRARY: []
}));

vi.mock('$lib/mocks/today', () => ({
	createDemoSession: demoMocks.createDemoSession,
	EXERCISE_LIBRARY: demoMocks.EXERCISE_LIBRARY
}));

const offlineSessionMocks = vi.hoisted(() => ({
	loadOfflineSession: vi.fn(),
	saveOfflineSession: vi.fn(),
	deleteOfflineSession: vi.fn(),
	sessionKeyForId: vi.fn((id: number) => `session:${id}`)
}));

vi.mock('$lib/offline/todaySessions', () => ({
	loadOfflineSession: offlineSessionMocks.loadOfflineSession,
	saveOfflineSession: offlineSessionMocks.saveOfflineSession,
	deleteOfflineSession: offlineSessionMocks.deleteOfflineSession,
	sessionKeyForId: offlineSessionMocks.sessionKeyForId
}));

const offlineActionMocks = vi.hoisted(() => ({
	hydrateOfflineState: vi.fn(),
	persistInProgressSession: vi.fn(),
	refreshPendingSyncCount: vi.fn(),
	setOffline: vi.fn(
		(
			access: {
				offlineMode: { set: (value: boolean) => void };
				notice: { set: (value: string | null) => void };
			},
			message?: string
		) => {
			access.offlineMode.set(true);
			access.notice.set(
				message ?? 'Offline mode: changes are saved on this device and will sync later.'
			);
		}
	)
}));

vi.mock('$lib/today/controller/offline', () => ({
	hydrateOfflineState: offlineActionMocks.hydrateOfflineState,
	persistInProgressSession: offlineActionMocks.persistInProgressSession,
	refreshPendingSyncCount: offlineActionMocks.refreshPendingSyncCount,
	setOffline: offlineActionMocks.setOffline
}));

const sharedMocks = vi.hoisted(() => ({
	resetLocalSessionUi: vi.fn()
}));

vi.mock('$lib/today/controller/actions/shared', () => ({
	resetLocalSessionUi: sharedMocks.resetLocalSessionUi
}));

describe('today controller session actions', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		demoMocks.createDemoSession.mockReturnValue({
			startedAt: '2026-01-01T10:00:00.000Z',
			notes: 'demo',
			exercises: []
		});
	});

	it('starts an online empty session', async () => {
		apiMocks.createWorkout.mockResolvedValueOnce({ id: 42 });

		const state = createTodayState();
		const refreshFromBackend = vi.fn(async () => undefined);
		const addExercise = vi.fn(async () => undefined);
		const actions = createSessionActions({ state, addExercise, refreshFromBackend });

		await actions.startSession('empty');

		expect(apiMocks.createWorkout).toHaveBeenCalledTimes(1);
		expect(get(state.currentSession)?.id).toBe(42);
		expect(refreshFromBackend).toHaveBeenCalledTimes(1);
		expect(addExercise).not.toHaveBeenCalled();
	});

	it('falls back to offline local session when start fails from network', async () => {
		apiMocks.createWorkout.mockRejectedValueOnce(new TypeError('Failed to fetch'));

		const state = createTodayState();
		const refreshFromBackend = vi.fn(async () => undefined);
		const addExercise = vi.fn(async () => undefined);
		const actions = createSessionActions({ state, addExercise, refreshFromBackend });

		await actions.startSession('empty');

		const session = get(state.currentSession);
		expect(session).toBeTruthy();
		expect(session!.id).toBeLessThan(0);
		expect(offlineActionMocks.setOffline).toHaveBeenCalledTimes(1);
		expect(offlineActionMocks.persistInProgressSession).toHaveBeenCalledTimes(1);
		expect(refreshFromBackend).toHaveBeenCalledTimes(1);
	});

	it('cancels local session with linked server id by saving pending cancel record', async () => {
		const state = createTodayState();
		state.offlineMode.set(true);
		state.currentSession.set({
			id: -7,
			startedAt: '2026-01-01T10:00:00.000Z',
			notes: 'n',
			exercises: []
		});
		offlineSessionMocks.loadOfflineSession.mockResolvedValueOnce({
			key: 'session:-7',
			status: 'in_progress',
			updated_at: '2026-01-01T10:01:00.000Z',
			session: get(state.currentSession),
			server_workout_id: 900,
			server_exercise_ids_by_local: {},
			deleted_server_exercise_ids: []
		});

		const refreshFromBackend = vi.fn(async () => undefined);
		const addExercise = vi.fn(async () => undefined);
		const actions = createSessionActions({ state, addExercise, refreshFromBackend });

		await actions.cancelSession();

		expect(offlineSessionMocks.saveOfflineSession).toHaveBeenCalledWith(
			expect.objectContaining({
				status: 'pending_sync',
				cancel_workout: true,
				server_workout_id: 900
			})
		);
		expect(get(state.currentSession)).toBeNull();
		expect(get(state.notice)).toBe('Session canceled locally.');
		expect(offlineActionMocks.refreshPendingSyncCount).toHaveBeenCalledTimes(1);
	});

	it('ends a local session by persisting pending_sync with mood and notes', async () => {
		const state = createTodayState();
		state.offlineMode.set(true);
		state.endModalOpen.set(true);
		state.endMood.set('😊');
		state.endNotes.set('  great session  ');
		state.currentSession.set({
			id: -9,
			startedAt: '2026-01-01T10:00:00.000Z',
			notes: 'session note',
			exercises: [
				{
					id: -3,
					name: 'Bench Press',
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
		offlineSessionMocks.loadOfflineSession.mockResolvedValueOnce(null);

		const refreshFromBackend = vi.fn(async () => undefined);
		const addExercise = vi.fn(async () => undefined);
		const actions = createSessionActions({ state, addExercise, refreshFromBackend });

		await actions.submitEndSession();

		expect(offlineSessionMocks.saveOfflineSession).toHaveBeenCalledWith(
			expect.objectContaining({
				key: 'session:-9',
				status: 'pending_sync',
				end_mood: '😊',
				end_notes: 'great session'
			})
		);
		expect(get(state.currentSession)).toBeNull();
		expect(get(state.endModalOpen)).toBe(false);
		expect(offlineActionMocks.setOffline).toHaveBeenCalledWith(
			expect.anything(),
			'Saved locally. Sync when you’re back online.'
		);
	});

	it('surfaces start-session non-network errors', async () => {
		apiMocks.createWorkout.mockRejectedValueOnce(new Error('server rejected payload'));

		const state = createTodayState();
		const refreshFromBackend = vi.fn(async () => undefined);
		const actions = createSessionActions({
			state,
			addExercise: vi.fn(async () => undefined),
			refreshFromBackend
		});

		await actions.startSession('empty');

		expect(offlineActionMocks.setOffline).not.toHaveBeenCalled();
		expect(get(state.error)).toBe('server rejected payload');
		expect(refreshFromBackend).toHaveBeenCalledTimes(1);
	});

	it('starts an empty session from a template and queues planned exercises without legacy notes', async () => {
		apiMocks.getWorkoutTemplate.mockResolvedValueOnce({
			template: {
				id: 12,
				name: 'Push A',
				exercise_count: 2,
				created_at: '2026-01-01T00:00:00.000Z',
				updated_at: '2026-01-01T00:00:00.000Z'
			},
			exercises: [
				{
					id: 31,
					template_id: 12,
					position: 2,
					exercise_type: 'Overhead Press',
					notes: null,
					per_side_weight: false,
					split_weight: false,
					settings: []
				},
				{
					id: 30,
					template_id: 12,
					position: 1,
					exercise_type: 'Bench Press',
					notes: 'pause reps',
					per_side_weight: true,
					split_weight: true,
					settings: [
						{ id: 1, template_exercise_id: 30, key: 'Bench angle', value: '30' },
						{ id: 2, template_exercise_id: 30, key: '_tracking_fields', value: 'reps,weight' }
					]
				}
			]
		});
		apiMocks.createWorkout.mockResolvedValueOnce({ id: 77 });

		const state = createTodayState();
		const refreshFromBackend = vi.fn(async () => undefined);
		const actions = createSessionActions({
			state,
			addExercise: vi.fn(async () => undefined),
			refreshFromBackend
		});

		await actions.startSessionFromTemplate(12);

		expect(apiMocks.getWorkoutTemplate).toHaveBeenCalledWith(12);
		expect(apiMocks.createWorkout).toHaveBeenCalledWith(
			expect.objectContaining({
				start_time: expect.any(String),
				date: expect.any(String)
			})
		);
		expect(get(state.currentSession)?.id).toBe(77);
		expect(get(state.currentSession)?.exercises).toEqual([]);
		expect(get(state.plannedTemplateExercises)).toEqual([
			expect.objectContaining({
				id: 30,
				name: 'Bench Press',
				perSideWeight: true,
				splitWeight: true,
				tracksReps: true,
				tracksTime: false,
				tracksWeight: true,
				settings: [{ key: 'Bench angle', value: '30' }]
			}),
			expect.objectContaining({ id: 31, name: 'Overhead Press' })
		]);
		expect(get(state.plannedTemplateExercises).some((exercise) => exercise.notes)).toBe(false);
		expect(refreshFromBackend).toHaveBeenCalledTimes(1);
		expect(offlineActionMocks.setOffline).not.toHaveBeenCalled();
	});
});
