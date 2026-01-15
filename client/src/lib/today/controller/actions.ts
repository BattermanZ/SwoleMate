import type { TodayState } from './state';
import { refreshFromBackend } from './actions/backend';
import { hydrateExerciseLibrary } from './actions/library';
import { createPersistScheduler, attachSessionNotesPersistence } from './actions/shared';
import { createSyncActions } from './actions/sync';
import { createSessionActions } from './actions/session';
import { createExerciseCoreActions } from './actions/exercise/core';
import { createExerciseSetActions } from './actions/exercise/sets';
import { createExerciseSettingsActions } from './actions/exercise/settings';
import { createExerciseWeightModeActions } from './actions/exercise/weightModes';

export function createTodayActions(state: TodayState) {
	const { schedulePersist } = createPersistScheduler(state);
	attachSessionNotesPersistence(state, schedulePersist);

	const refresh = () => refreshFromBackend(state);
	const hydrateLibrary = () => hydrateExerciseLibrary(state);

	const setActions = createExerciseSetActions({ state });
	const coreActions = createExerciseCoreActions({
		state,
		addSet: setActions.addSet,
		refreshFromBackend: refresh
	});
	const settingsActions = createExerciseSettingsActions({ state, schedulePersist });
	const weightModeActions = createExerciseWeightModeActions({ state, refreshFromBackend: refresh });

	const sessionActions = createSessionActions({
		state,
		addExercise: coreActions.addExercise,
		refreshFromBackend: refresh
	});

	const syncActions = createSyncActions({
		state,
		refreshFromBackend: refresh,
		hydrateExerciseLibrary: hydrateLibrary
	});

	return {
		// actions
		...coreActions,
		...setActions,
		...settingsActions,
		...weightModeActions,
		...sessionActions,
		...syncActions,
		refreshFromBackend: refresh
	};
}
