import { get } from 'svelte/store';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createTodayState } from '$lib/today/controller/state';
import { createSyncActions } from '$lib/today/controller/actions/sync';

const apiMocks = vi.hoisted(() => ({
	cancelExercise: vi.fn(),
	cancelWorkout: vi.fn(),
	createExercise: vi.fn(),
	createWorkout: vi.fn(),
	endExercise: vi.fn(),
	endWorkout: vi.fn(),
	replaceSets: vi.fn()
}));

vi.mock('$lib/api', () => ({
	cancelExercise: apiMocks.cancelExercise,
	cancelWorkout: apiMocks.cancelWorkout,
	createExercise: apiMocks.createExercise,
	createWorkout: apiMocks.createWorkout,
	endExercise: apiMocks.endExercise,
	endWorkout: apiMocks.endWorkout,
	replaceSets: apiMocks.replaceSets
}));

const sessionMocks = vi.hoisted(() => ({
	listOfflineSessions: vi.fn()
}));

vi.mock('$lib/offline/todaySessions', () => ({
	listOfflineSessions: sessionMocks.listOfflineSessions
}));

const offlineMocks = vi.hoisted(() => ({
	hydrateOfflineState: vi.fn(),
	persistInProgressSession: vi.fn(),
	refreshPendingSyncCount: vi.fn(),
	syncOne: vi.fn(),
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
	hydrateOfflineState: offlineMocks.hydrateOfflineState,
	persistInProgressSession: offlineMocks.persistInProgressSession,
	refreshPendingSyncCount: offlineMocks.refreshPendingSyncCount,
	syncOne: offlineMocks.syncOne,
	setOffline: offlineMocks.setOffline
}));

describe('today controller sync actions', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	it('syncPendingSessions clears notice when there are no offline records', async () => {
		sessionMocks.listOfflineSessions.mockResolvedValueOnce([]);
		const state = createTodayState();

		const refreshFromBackend = vi.fn(async () => undefined);
		const hydrateExerciseLibrary = vi.fn(async () => undefined);
		const actions = createSyncActions({ state, refreshFromBackend, hydrateExerciseLibrary });

		await actions.syncPendingSessions();

		expect(get(state.notice)).toBeNull();
		expect(offlineMocks.syncOne).not.toHaveBeenCalled();
		expect(refreshFromBackend).not.toHaveBeenCalled();
	});

	it('syncPendingSessions runs syncOne for records and refreshes backend', async () => {
		sessionMocks.listOfflineSessions.mockResolvedValueOnce([
			{ key: 'a', status: 'pending_sync' },
			{ key: 'b', status: 'in_progress' }
		]);
		const state = createTodayState();
		offlineMocks.refreshPendingSyncCount.mockImplementationOnce(
			async (access: { pendingSyncCount: { set: (value: number) => void } }) => {
				access.pendingSyncCount.set(0);
				return 0;
			}
		);

		const refreshFromBackend = vi.fn(async () => undefined);
		const hydrateExerciseLibrary = vi.fn(async () => undefined);
		const actions = createSyncActions({ state, refreshFromBackend, hydrateExerciseLibrary });

		await actions.syncPendingSessions();

		expect(offlineMocks.syncOne).toHaveBeenCalledTimes(2);
		expect(offlineMocks.refreshPendingSyncCount).toHaveBeenCalledTimes(1);
		expect(get(state.offlineMode)).toBe(false);
		expect(refreshFromBackend).toHaveBeenCalledTimes(1);
	});

	it('syncPendingSessions sets offline message on network failure', async () => {
		sessionMocks.listOfflineSessions.mockRejectedValueOnce(new TypeError('Failed to fetch'));
		const state = createTodayState();

		const refreshFromBackend = vi.fn(async () => undefined);
		const hydrateExerciseLibrary = vi.fn(async () => undefined);
		const actions = createSyncActions({ state, refreshFromBackend, hydrateExerciseLibrary });

		await actions.syncPendingSessions();

		expect(offlineMocks.setOffline).toHaveBeenCalledWith(
			expect.anything(),
			'Still offline. Your changes are safe and will sync later.'
		);
	});

	it('syncPendingSessions sets error for non-network failures', async () => {
		sessionMocks.listOfflineSessions.mockRejectedValueOnce(new Error('server exploded'));
		const state = createTodayState();
		const actions = createSyncActions({
			state,
			refreshFromBackend: vi.fn(async () => undefined),
			hydrateExerciseLibrary: vi.fn(async () => undefined)
		});

		await actions.syncPendingSessions();

		expect(offlineMocks.setOffline).not.toHaveBeenCalled();
		expect(get(state.error)).toBe('server exploded');
	});

	it('start wires online/offline listeners and returns cleanup', async () => {
		const listeners = new Map<string, EventListener>();
		const addSpy = vi.spyOn(window, 'addEventListener').mockImplementation((type, listener) => {
			listeners.set(type, listener as EventListener);
		});
		const removeSpy = vi.spyOn(window, 'removeEventListener').mockImplementation(() => undefined);
		const setIntervalSpy = vi
			.spyOn(window, 'setInterval')
			.mockReturnValue(123 as unknown as ReturnType<typeof window.setInterval>);
		const clearIntervalSpy = vi.spyOn(window, 'clearInterval').mockImplementation(() => undefined);

		const state = createTodayState();
		state.pendingSyncCount.set(1);
		sessionMocks.listOfflineSessions.mockResolvedValueOnce([]);

		const refreshFromBackend = vi.fn(async () => undefined);
		const hydrateExerciseLibrary = vi.fn(async () => undefined);
		const actions = createSyncActions({ state, refreshFromBackend, hydrateExerciseLibrary });

		const dispose = actions.start();

		expect(offlineMocks.hydrateOfflineState).toHaveBeenCalledTimes(1);
		expect(refreshFromBackend).toHaveBeenCalledTimes(1);
		expect(hydrateExerciseLibrary).toHaveBeenCalledTimes(1);
		expect(addSpy).toHaveBeenCalledWith('online', expect.any(Function));
		expect(addSpy).toHaveBeenCalledWith('offline', expect.any(Function));
		expect(setIntervalSpy).toHaveBeenCalledTimes(1);

		listeners.get('online')?.(new Event('online'));
		await vi.waitFor(() => expect(sessionMocks.listOfflineSessions).toHaveBeenCalled());

		listeners.get('offline')?.(new Event('offline'));
		expect(offlineMocks.setOffline).toHaveBeenCalled();
		expect(offlineMocks.persistInProgressSession).toHaveBeenCalled();

		dispose();
		expect(clearIntervalSpy).toHaveBeenCalledWith(123);
		expect(removeSpy).toHaveBeenCalledWith('online', expect.any(Function));
		expect(removeSpy).toHaveBeenCalledWith('offline', expect.any(Function));

		addSpy.mockRestore();
		removeSpy.mockRestore();
		setIntervalSpy.mockRestore();
		clearIntervalSpy.mockRestore();
	});
});
