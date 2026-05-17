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
import { persistPlannedTemplate } from './actions/plannedTemplate';
import { get } from 'svelte/store';

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

	async function startPlannedTemplateExercise(plannedExerciseId: number) {
		const planned = get(state.plannedTemplateExercises).find(
			(exercise) => exercise.id === plannedExerciseId
		);
		if (!planned) return;

		await coreActions.addExercise(planned.name, {
			notes: planned.notes,
			perSideWeight: planned.perSideWeight,
			splitWeight: planned.splitWeight,
			tracksReps: planned.tracksReps,
			tracksTime: planned.tracksTime,
			tracksWeight: planned.tracksWeight,
			settings: planned.settings
		});

		if (!get(state.error)) {
			state.plannedTemplateExercises.update((exercises) =>
				exercises.filter((exercise) => exercise.id !== plannedExerciseId)
			);
			const session = get(state.currentSession);
			if (session) {
				await persistPlannedTemplate(session.id, get(state.plannedTemplateExercises));
			}
		}
	}

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
		startPlannedTemplateExercise,
		...syncActions,
		refreshFromBackend: refresh
	};
}
