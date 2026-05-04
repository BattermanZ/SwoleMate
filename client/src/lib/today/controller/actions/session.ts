import {
	cancelWorkout,
	createWorkout,
	endExercise,
	endWorkout,
	startWorkoutFromTemplate
} from '$lib/api';
import { createDemoSession } from '$lib/mocks/today';
import {
	deleteOfflineSession,
	loadOfflineSession,
	saveOfflineSession,
	sessionKeyForId
} from '$lib/offline/todaySessions';
import type { UiSession } from '$lib/today/types';
import { get } from 'svelte/store';
import {
	hydrateOfflineState,
	persistInProgressSession,
	refreshPendingSyncCount,
	setOffline
} from '../offline';
import type { TodayState } from '../state';
import { getErrorMessage, isNetworkFailure, makeLocalNumericId } from '../utils';
import type { ExerciseSeedOptions, SeedSet } from './types';
import { resetLocalSessionUi } from './shared';
import { trackingFieldsSetting } from '$lib/today/tracking';

export type SessionActions = {
	startSession: (mode: 'empty' | 'demo') => Promise<void>;
	startSessionFromTemplate: (templateId: number) => Promise<void>;
	cancelSession: () => Promise<void>;
	openEndModal: () => void;
	submitEndSession: () => Promise<void>;
};

export function createSessionActions(args: {
	state: TodayState;
	addExercise: (name: string, options?: ExerciseSeedOptions, seedSets?: SeedSet[]) => Promise<void>;
	refreshFromBackend: () => Promise<void>;
}) {
	const { state, addExercise, refreshFromBackend } = args;

	function beginLocalSession(sessionId: number, startIso: string, notes: string) {
		const timezoneOffsetMinutes = new Date(startIso).getTimezoneOffset();
		state.currentSession.set({
			id: sessionId,
			startedAt: startIso,
			timezoneOffsetMinutes,
			notes,
			exercises: []
		});
		state.sessionNotes.set(notes);
		state.openExerciseId.set(null);
		resetLocalSessionUi(state);
	}

	async function startSession(mode: 'empty' | 'demo') {
		if (get(state.currentSession)) return;
		state.error.set(null);

		try {
			const demo = mode === 'demo' ? createDemoSession() : null;
			const startIso = demo?.startedAt ?? new Date().toISOString();
			const timezoneOffsetMinutes = new Date(startIso).getTimezoneOffset();

			state.loading.set(true);
			const payload = {
				date: startIso,
				start_time: startIso,
				notes: demo?.notes?.trim() || undefined,
				timezone_offset_minutes: timezoneOffsetMinutes
			};
			const created = await createWorkout(payload);
			beginLocalSession(created.id, startIso, demo?.notes ?? '');

			if (demo) {
				for (const ex of demo.exercises) {
					await addExercise(
						ex.name,
						{
							notes: ex.notes,
							perSideWeight: ex.perSideWeight,
							splitWeight: ex.splitWeight,
							tracksReps: ex.tracksReps,
							tracksTime: ex.tracksTime,
							tracksWeight: ex.tracksWeight,
							settings: ex.settings.map((s) => ({ key: s.key, value: s.value }))
						},
						ex.sets.map((s) => ({
							reps: s.reps,
							weight: s.weight,
							weightLeft: s.weightLeft,
							weightRight: s.weightRight,
							durationSeconds: s.durationSeconds
						}))
					);
				}
			}

			await refreshFromBackend();
		} catch (e) {
			if (isNetworkFailure(e)) {
				setOffline(state, 'Offline mode: started a local session (will sync when back online).');
				const demo = mode === 'demo' ? createDemoSession() : null;
				const startIso = demo?.startedAt ?? new Date().toISOString();
				const localId = makeLocalNumericId();
				beginLocalSession(localId, startIso, demo?.notes ?? '');

				if (demo) {
					for (const ex of demo.exercises) {
						await addExercise(
							ex.name,
							{
								notes: ex.notes,
								perSideWeight: ex.perSideWeight,
								splitWeight: ex.splitWeight,
								tracksReps: ex.tracksReps,
								tracksTime: ex.tracksTime,
								tracksWeight: ex.tracksWeight,
								settings: ex.settings.map((s) => ({ key: s.key, value: s.value }))
							},
							ex.sets.map((s) => ({
								reps: s.reps,
								weight: s.weight,
								weightLeft: s.weightLeft,
								weightRight: s.weightRight,
								durationSeconds: s.durationSeconds
							}))
						);
					}
				}

				await persistInProgressSession(state);
			} else {
				state.error.set(getErrorMessage(e));
			}
			await refreshFromBackend();
		} finally {
			state.loading.set(false);
		}
	}

	async function startSessionFromTemplate(templateId: number) {
		if (get(state.currentSession)) return;
		state.error.set(null);
		state.loading.set(true);

		try {
			const startIso = new Date().toISOString();
			const timezoneOffsetMinutes = new Date(startIso).getTimezoneOffset();
			const created = await startWorkoutFromTemplate(templateId, {
				date: startIso,
				start_time: startIso,
				timezone_offset_minutes: timezoneOffsetMinutes
			});
			beginLocalSession(created.id, startIso, '');
			await refreshFromBackend();
		} catch (e) {
			state.error.set(getErrorMessage(e));
		} finally {
			state.loading.set(false);
		}
	}

	async function cancelSession() {
		const session = get(state.currentSession);
		if (!session) return;
		state.error.set(null);

		try {
			if (get(state.offlineMode) || session.id < 0) {
				const key = sessionKeyForId(session.id);
				const existing = await loadOfflineSession(key).catch(() => null);
				if (existing?.server_workout_id) {
					await saveOfflineSession({
						...existing,
						status: 'pending_sync',
						cancel_workout: true,
						updated_at: new Date().toISOString()
					});
				} else {
					await deleteOfflineSession(key).catch(() => undefined);
				}
				state.currentSession.set(null);
				state.sessionNotes.set('');
				state.openExerciseId.set(null);
				resetLocalSessionUi(state);
				await refreshPendingSyncCount(state);
				state.notice.set('Session canceled locally.');
				return;
			}

			state.loading.set(true);
			await cancelWorkout(session.id);
			state.currentSession.set(null);
			state.sessionNotes.set('');
			state.openExerciseId.set(null);
			resetLocalSessionUi(state);
			await refreshFromBackend();
		} catch (e) {
			if (isNetworkFailure(e)) {
				setOffline(state);
				await persistInProgressSession(state);
				await hydrateOfflineState(state);
			} else {
				state.error.set(getErrorMessage(e));
			}
		} finally {
			state.loading.set(false);
		}
	}

	function openEndModal() {
		const session = get(state.currentSession);
		if (!session) return;
		state.endNotes.set(get(state.sessionNotes));
		state.endMood.set(null);
		state.endModalOpen.set(true);
	}

	async function submitEndSession() {
		const session = get(state.currentSession);
		const mood = get(state.endMood);
		if (!session || !mood) return;
		state.error.set(null);

		try {
			const endedAt = new Date().toISOString();

			if (get(state.offlineMode) || session.id < 0) {
				const endedExercises = session.exercises.map((e) =>
					e.status === 'done' ? e : { ...e, status: 'done' as const, endedAt }
				);
				const ended: UiSession = { ...session, endedAt, mood, exercises: endedExercises };
				state.currentSession.set(null);
				state.sessionNotes.set('');
				state.openExerciseId.set(null);
				resetLocalSessionUi(state);

				const key = sessionKeyForId(session.id);
				const existing = await loadOfflineSession(key).catch(() => null);
				await saveOfflineSession({
					key,
					status: 'pending_sync',
					updated_at: new Date().toISOString(),
					session: ended,
					end_mood: mood,
					end_notes: get(state.endNotes).trim() || undefined,
					server_workout_id: existing?.server_workout_id,
					server_exercise_ids_by_local: existing?.server_exercise_ids_by_local ?? {},
					deleted_server_exercise_ids: existing?.deleted_server_exercise_ids ?? []
				});
				await refreshPendingSyncCount(state);
				setOffline(state, 'Saved locally. Sync when you’re back online.');
				state.endModalOpen.set(false);
				return;
			}

			state.loading.set(true);
			await Promise.all(
				session.exercises
					.filter((e) => e.status !== 'done')
					.map((e) =>
						endExercise(e.id, {
							end_time: endedAt,
							notes: e.notes || undefined,
							per_side_weight: e.perSideWeight,
							split_weight: e.splitWeight,
							settings: [
								...e.settings.map((s) => ({ key: s.key, value: s.value })),
								trackingFieldsSetting({
									reps: e.tracksReps ?? true,
									time: e.tracksTime ?? false,
									weight: e.tracksWeight ?? true
								})
							]
						})
					)
			);

			await endWorkout(session.id, {
				end_time: endedAt,
				notes: get(state.endNotes).trim() || undefined,
				feedback: mood
			});

			state.currentSession.set(null);
			state.sessionNotes.set('');
			state.openExerciseId.set(null);
			resetLocalSessionUi(state);
			await refreshFromBackend();
		} catch (e) {
			if (isNetworkFailure(e)) {
				setOffline(state);
				await persistInProgressSession(state);
				await hydrateOfflineState(state);
			} else {
				state.error.set(getErrorMessage(e));
			}
		} finally {
			state.loading.set(false);
		}
	}

	return {
		startSession,
		startSessionFromTemplate,
		cancelSession,
		openEndModal,
		submitEndSession
	} satisfies SessionActions;
}
