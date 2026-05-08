import { createSet, endExercise } from '$lib/api';
import type { TodayState } from '../../state';
import { get } from 'svelte/store';
import { hydrateOfflineState, persistInProgressSession, setOffline } from '../../offline';
import { getErrorMessage, isNetworkFailure, makeLocalNumericId } from '../../utils';
import { trackingFieldsSetting } from '$lib/today/tracking';

export type ExerciseSetActions = {
	markExerciseDone: (exerciseId: number) => Promise<void>;
	addSet: (
		exerciseId: number,
		reps: number,
		weight: number,
		weightLeft?: number,
		weightRight?: number,
		durationSeconds?: number
	) => Promise<void>;
};

export function createExerciseSetActions(args: {
	state: TodayState;
	refreshFromBackend?: () => Promise<void>;
}) {
	const { state, refreshFromBackend } = args;

	async function markExerciseDone(exerciseId: number) {
		const session = get(state.currentSession);
		if (!session) return;
		state.error.set(null);

		try {
			const endedAt = new Date().toISOString();
			const ex = session.exercises.find((e) => e.id === exerciseId);
			if (!ex) return;

			if (get(state.offlineMode) || session.id < 0 || exerciseId < 0) {
				state.currentSession.set({
					...session,
					exercises: session.exercises.map((e) =>
						e.id === exerciseId ? { ...e, status: 'done' as const, endedAt } : e
					)
				});
				state.openExerciseIds.update((current) => current.filter((id) => id !== exerciseId));
				await persistInProgressSession(state);
				return;
			}

			state.loading.set(true);
			await endExercise(exerciseId, {
				end_time: endedAt,
				notes: ex.notes || undefined,
				per_side_weight: ex.perSideWeight,
				split_weight: ex.splitWeight,
				settings: [
					...ex.settings.map((s) => ({ key: s.key, value: s.value })),
					trackingFieldsSetting({
						reps: ex.tracksReps ?? true,
						time: ex.tracksTime ?? false,
						weight: ex.tracksWeight ?? true
					})
				]
			});

			state.currentSession.set({
				...session,
				exercises: session.exercises.map((e) =>
					e.id === exerciseId ? { ...e, status: 'done' as const, endedAt } : e
				)
			});
			state.openExerciseIds.update((current) => current.filter((id) => id !== exerciseId));
		} catch (e) {
			if (isNetworkFailure(e)) {
				setOffline(state);
				await persistInProgressSession(state);
				await markExerciseDone(exerciseId);
			} else {
				state.error.set(getErrorMessage(e));
			}
		} finally {
			state.loading.set(false);
		}
	}

	async function addSet(
		exerciseId: number,
		reps: number,
		weight: number,
		weightLeft?: number,
		weightRight?: number,
		durationSeconds?: number
	) {
		const session = get(state.currentSession);
		if (!session) return;
		state.error.set(null);

		try {
			if (get(state.offlineMode) || session.id < 0 || exerciseId < 0) {
				const setId = makeLocalNumericId();
				state.currentSession.set({
					...session,
					exercises: session.exercises.map((e) => {
						if (e.id !== exerciseId) return e;
						return {
							...e,
							sets: [
								...e.sets,
								{
									id: setId,
									reps,
									weight,
									weightLeft,
									weightRight,
									durationSeconds
								}
							]
						};
					})
				});
				await persistInProgressSession(state);
				return;
			}

			state.loading.set(true);
			const created = await createSet(exerciseId, {
				reps,
				weight,
				weight_left: weightLeft,
				weight_right: weightRight,
				duration_seconds: durationSeconds,
				notes: undefined
			});

			state.currentSession.set({
				...session,
				exercises: session.exercises.map((e) => {
					if (e.id !== exerciseId) return e;
					return {
						...e,
						sets: [
							...e.sets,
							{
								id: created.id,
								reps,
								weight,
								weightLeft,
								weightRight,
								durationSeconds
							}
						]
					};
				})
			});
			await refreshFromBackend?.();
		} catch (e) {
			if (isNetworkFailure(e)) {
				setOffline(state);
				await persistInProgressSession(state);
				await hydrateOfflineState(state);
				await addSet(exerciseId, reps, weight, weightLeft, weightRight, durationSeconds);
			} else {
				state.error.set(getErrorMessage(e));
			}
		} finally {
			state.loading.set(false);
		}
	}

	return { addSet, markExerciseDone } satisfies ExerciseSetActions;
}
