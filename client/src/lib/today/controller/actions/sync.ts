import {
	cancelExercise,
	cancelWorkout,
	createExercise,
	createWorkout,
	endExercise,
	endWorkout,
	replaceSets
} from '$lib/api';
import { listOfflineSessions } from '$lib/offline/todaySessions';
import { get } from 'svelte/store';
import {
	hydrateOfflineState,
	persistInProgressSession,
	refreshPendingSyncCount,
	setOffline,
	syncOne
} from '../offline';
import type { TodayState } from '../state';
import { getErrorMessage, isNetworkFailure } from '../utils';

export type SyncActions = {
	syncPendingSessions: () => Promise<void>;
	start: () => () => void;
};

// Module-level guard: flaky connectivity fires online/offline/online in quick
// succession, and each `online` invokes syncPendingSessions. Two concurrent runs
// would both list and re-create the same offline records, duplicating workouts on
// the server. A single in-flight run short-circuits the rest.
let isSyncing = false;

export function createSyncActions(args: {
	state: TodayState;
	refreshFromBackend: () => Promise<void>;
	hydrateExerciseLibrary: () => Promise<void>;
}) {
	const { state, refreshFromBackend, hydrateExerciseLibrary } = args;

	async function syncPendingSessions() {
		if (isSyncing) return;
		isSyncing = true;
		state.error.set(null);
		state.notice.set('Syncing offline changes…');

		try {
			const records = await listOfflineSessions();
			if (!records.length) {
				state.notice.set(null);
				return;
			}

			state.loading.set(true);
			for (const record of records) {
				await syncOne(record, {
					cancelExercise,
					cancelWorkout,
					createExercise,
					createWorkout,
					endExercise,
					endWorkout,
					replaceSets
				});
			}
			await refreshPendingSyncCount(state);
			state.offlineMode.set(false);
			state.notice.set(get(state.pendingSyncCount) ? 'Some changes are still pending sync.' : null);
			await refreshFromBackend();
		} catch (e) {
			if (isNetworkFailure(e)) {
				setOffline(state, 'Still offline. Your changes are safe and will sync later.');
			} else {
				state.error.set(getErrorMessage(e));
			}
		} finally {
			state.loading.set(false);
			isSyncing = false;
		}
	}

	function start() {
		void hydrateOfflineState(state);
		void refreshFromBackend();
		void hydrateExerciseLibrary();

		const onOnline = () => {
			if (get(state.pendingSyncCount)) void syncPendingSessions();
		};
		const onOffline = () => {
			setOffline(state);
			void persistInProgressSession(state);
		};
		window.addEventListener('online', onOnline);
		window.addEventListener('offline', onOffline);

		const timer = window.setInterval(() => state.nowMs.set(Date.now()), 10_000);
		return () => {
			window.clearInterval(timer);
			window.removeEventListener('online', onOnline);
			window.removeEventListener('offline', onOffline);
		};
	}

	return { syncPendingSessions, start } satisfies SyncActions;
}
