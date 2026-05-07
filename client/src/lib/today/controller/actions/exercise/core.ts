import { cancelExercise, createExercise } from '$lib/api';
import { loadOfflineSession, sessionKeyForId } from '$lib/offline/todaySessions';
import { createId } from '$lib/utils/id';
import { get } from 'svelte/store';
import { hydrateOfflineState, persistInProgressSession, setOffline } from '../../offline';
import type { TodayState } from '../../state';
import { getErrorMessage, isNetworkFailure, makeLocalNumericId } from '../../utils';
import type { ExerciseSeedOptions, LastTime, SeedSet } from '../types';
import { trackingFieldsSetting } from '$lib/today/tracking';

export type ExerciseCoreActions = {
	toggleExercise: (exerciseId: number) => void;
	getLastTimeForExercise: (name: string) => LastTime | undefined;
	addExercise: (name: string, options?: ExerciseSeedOptions, seedSets?: SeedSet[]) => Promise<void>;
	removeExercise: (exerciseId: number) => Promise<void>;
};

export function createExerciseCoreActions(args: {
	state: TodayState;
	addSet: (
		exerciseId: number,
		reps: number,
		weight: number,
		weightLeft?: number,
		weightRight?: number,
		durationSeconds?: number
	) => Promise<void>;
	refreshFromBackend: () => Promise<void>;
}): ExerciseCoreActions {
	const { state, addSet, refreshFromBackend } = args;

	function toggleExercise(exerciseId: number) {
		state.openExerciseId.update((current) => (current === exerciseId ? null : exerciseId));
	}

	function getLastTimeForExercise(name: string): LastTime | undefined {
		const sessions = get(state.recentSessions);
		for (const session of sessions) {
			const match = session.exercises.find((e) => e.name === name);
			if (!match) continue;
			return {
				startedAt: session.startedAt,
				notes: match.notes,
				sets: match.sets,
				settings: match.settings,
				tracksReps: match.tracksReps,
				tracksTime: match.tracksTime,
				tracksWeight: match.tracksWeight,
				perSideWeight: match.perSideWeight,
				splitWeight: match.splitWeight
			};
		}
		return undefined;
	}

	async function addExercise(name: string, options?: ExerciseSeedOptions, seedSets?: SeedSet[]) {
		const session = get(state.currentSession);
		if (!session) return;
		const trimmed = name.trim();
		if (!trimmed) return;

		state.error.set(null);

		try {
			const last = getLastTimeForExercise(trimmed);
			const startIso = new Date().toISOString();
			const perSideWeight = options?.perSideWeight ?? last?.perSideWeight ?? false;
			const requestedSplit = options?.splitWeight ?? last?.splitWeight ?? false;
			const splitWeight = perSideWeight ? requestedSplit : false;
			const settings =
				options?.settings ?? last?.settings.map((s) => ({ key: s.key, value: s.value })) ?? [];
			const trackingFields = {
				reps: options?.tracksReps ?? last?.tracksReps ?? true,
				time: options?.tracksTime ?? last?.tracksTime ?? false,
				weight: options?.tracksWeight ?? last?.tracksWeight ?? true
			};
			const persistedSettings = [...settings, trackingFieldsSetting(trackingFields)];

			if (get(state.offlineMode) || session.id < 0) {
				const localExerciseId = makeLocalNumericId();
				const newExercise = {
					id: localExerciseId,
					name: trimmed,
					notes: options?.notes?.trim() ?? '',
					startedAt: startIso,
					endedAt: startIso,
					status: 'active' as const,
					perSideWeight,
					splitWeight,
					tracksReps: trackingFields.reps,
					tracksTime: trackingFields.time,
					tracksWeight: trackingFields.weight,
					settings: settings.map((s) => ({
						id: createId('setting'),
						key: s.key,
						value: s.value
					})),
					sets: []
				};

				state.currentSession.set({
					...session,
					exercises: [...session.exercises, newExercise]
				});
				state.openExerciseId.set(newExercise.id);
				state.exerciseQuery.set('');
				await persistInProgressSession(state);

				if (seedSets?.length) {
					for (const s of seedSets) {
						await addSet(
							newExercise.id,
							s.reps,
							s.weight,
							s.weightLeft,
							s.weightRight,
							s.durationSeconds
						);
					}
				}
				return;
			}

			state.loading.set(true);
			const created = await createExercise(session.id, {
				exercise_type: trimmed,
				start_time: startIso,
				notes: options?.notes?.trim() || undefined,
				per_side_weight: perSideWeight,
				split_weight: splitWeight,
				settings: settings.length
					? persistedSettings.map((s) => ({ key: s.key, value: s.value }))
					: [trackingFieldsSetting(trackingFields)]
			});

			const newExercise = {
				id: created.id,
				name: trimmed,
				notes: options?.notes?.trim() ?? '',
				startedAt: startIso,
				endedAt: startIso,
				status: 'active' as const,
				perSideWeight,
				splitWeight,
				tracksReps: trackingFields.reps,
				tracksTime: trackingFields.time,
				tracksWeight: trackingFields.weight,
				settings: settings.map((s) => ({
					id: createId('setting'),
					key: s.key,
					value: s.value
				})),
				sets: []
			};

			state.currentSession.set({
				...session,
				exercises: [...session.exercises, newExercise]
			});

			state.openExerciseId.set(newExercise.id);
			state.exerciseQuery.set('');

			if (seedSets?.length) {
				for (const s of seedSets) {
					await addSet(
						newExercise.id,
						s.reps,
						s.weight,
						s.weightLeft,
						s.weightRight,
						s.durationSeconds
					);
				}
			}
		} catch (e) {
			if (isNetworkFailure(e)) {
				setOffline(state);
				await persistInProgressSession(state);
				await addExercise(trimmed, options, seedSets);
			} else {
				state.error.set(getErrorMessage(e));
			}
		} finally {
			state.loading.set(false);
		}
	}

	async function removeExercise(exerciseId: number) {
		const session = get(state.currentSession);
		if (!session) return;
		state.error.set(null);

		try {
			if (get(state.offlineMode) || session.id < 0) {
				state.currentSession.set({
					...session,
					exercises: session.exercises.filter((e) => e.id !== exerciseId)
				});
				state.openExerciseId.update((current) => (current === exerciseId ? null : current));
				if (exerciseId > 0) {
					const key = sessionKeyForId(session.id);
					const existing = await loadOfflineSession(key).catch(() => null);
					const prev = existing?.deleted_server_exercise_ids ?? [];
					await persistInProgressSession(state, {
						deleted_server_exercise_ids: Array.from(new Set([...prev, exerciseId]))
					});
				} else {
					await persistInProgressSession(state);
				}
				return;
			}

			state.loading.set(true);
			await cancelExercise(exerciseId);
			state.currentSession.set({
				...session,
				exercises: session.exercises.filter((e) => e.id !== exerciseId)
			});
			state.openExerciseId.update((current) => (current === exerciseId ? null : current));
		} catch (e) {
			if (isNetworkFailure(e)) {
				setOffline(state);
				await persistInProgressSession(state);
				await hydrateOfflineState(state);
			} else {
				state.error.set(getErrorMessage(e));
				await refreshFromBackend();
			}
		} finally {
			state.loading.set(false);
		}
	}

	return { toggleExercise, getLastTimeForExercise, addExercise, removeExercise };
}
