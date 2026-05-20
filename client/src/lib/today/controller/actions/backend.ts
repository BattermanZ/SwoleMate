import { getWorkout, getWorkouts } from '$lib/api';
import { kvSet } from '$lib/offline/storage';
import { deleteOfflineSession, sessionKeyForId } from '$lib/offline/todaySessions';
import { toUiSession, workoutIsActive } from '$lib/today/backend';
import { get } from 'svelte/store';
import {
	findInProgressOffline,
	getRecentSessionsStorageKey,
	hydrateOfflineState,
	refreshPendingSyncCount,
	setOffline
} from '../offline';
import type { TodayState } from '../state';
import { getErrorMessage, isNetworkFailure } from '../utils';
import { resetLocalSessionUi } from './shared';
import { clearPlannedTemplate, loadPlannedTemplate } from './plannedTemplate';

export async function refreshFromBackend(state: TodayState) {
	state.loading.set(true);
	state.error.set(null);

	try {
		const workouts = await getWorkouts();
		const active = workouts.find((w) => workoutIsActive(w) && w.id != null);

		if (active?.id != null) {
			const data = await getWorkout(active.id);
			const next = toUiSession(data.workout, data.exercises);
			state.currentSession.set(next);
			state.sessionNotes.set(next.notes);
			state.openExerciseIds.set(
				next.exercises
					.filter((exercise) => exercise.status !== 'done')
					.map((exercise) => exercise.id)
			);
			if (get(state.plannedTemplateExercises).length === 0) {
				const restored = await loadPlannedTemplate(next.id);
				if (restored && restored.length > 0) {
					state.plannedTemplateExercises.set(restored);
				}
			}
		} else {
			const offlineInProgress = await findInProgressOffline();
			if (offlineInProgress?.session) {
				const offlineSessionId =
					offlineInProgress.server_workout_id ?? offlineInProgress.session.id;
				const isAlreadyCompleted =
					offlineSessionId > 0 && workouts.some((w) => w.id === offlineSessionId);
				if (isAlreadyCompleted) {
					await deleteOfflineSession(offlineInProgress.key).catch(() => undefined);
					state.currentSession.set(null);
					state.sessionNotes.set('');
					state.openExerciseIds.set([]);
					state.plannedTemplateExercises.set([]);
					void clearPlannedTemplate();
				} else {
					state.currentSession.set(offlineInProgress.session);
					state.sessionNotes.set(offlineInProgress.session.notes);
					state.openExerciseIds.set(
						offlineInProgress.session.exercises
							.filter((exercise) => exercise.status !== 'done')
							.map((exercise) => exercise.id)
					);
					if (get(state.plannedTemplateExercises).length === 0) {
						const restored = await loadPlannedTemplate(offlineInProgress.session.id);
						if (restored && restored.length > 0) {
							state.plannedTemplateExercises.set(restored);
						}
					}
					state.notice.set('Local session in progress. You can keep logging and sync later.');
				}
			} else {
				state.currentSession.set(null);
				state.sessionNotes.set('');
				state.openExerciseIds.set([]);
				state.plannedTemplateExercises.set([]);
				void clearPlannedTemplate();
			}
		}

		const completed = workouts.filter((w) => w.id != null && !workoutIsActive(w)).slice(0, 2);
		const recent = await Promise.all(completed.map((w) => getWorkout(w.id!)));
		const nextRecent = recent.map((d) => toUiSession(d.workout, d.exercises));
		state.recentSessions.set(nextRecent);
		void kvSet(getRecentSessionsStorageKey(), nextRecent);

		resetLocalSessionUi(state);
		state.offlineMode.set(false);
		await refreshPendingSyncCount(state);
		if (get(state.pendingSyncCount) && !get(state.notice)) {
			state.notice.set('Offline changes pending sync.');
		}
		if (!get(state.pendingSyncCount) && !get(state.currentSession)) {
			state.notice.set(null);
		}
	} catch (e) {
		if (isNetworkFailure(e)) {
			if (!get(state.offlineMode)) setOffline(state);
			state.error.set(null);
			await hydrateOfflineState(state);
		} else {
			state.error.set(getErrorMessage(e));
		}
	} finally {
		state.loading.set(false);
	}
}
