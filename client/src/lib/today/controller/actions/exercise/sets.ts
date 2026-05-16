import { createSet, endExercise, replaceSets } from '$lib/api';
import type { CreateSetRequest } from '$lib/types';
import type { UiSet } from '$lib/today/types';
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
	updateSet: (exerciseId: number, setId: number, patch: Omit<UiSet, 'id'>) => Promise<void>;
	removeSet: (exerciseId: number, setId: number) => Promise<void>;
};

export function createExerciseSetActions(args: { state: TodayState }) {
	const { state } = args;

	function toSetRequest(set: Omit<UiSet, 'id'>): CreateSetRequest {
		return {
			reps: set.reps,
			weight: set.weight,
			weight_left: set.weightLeft,
			weight_right: set.weightRight,
			duration_seconds: set.durationSeconds,
			notes: undefined
		};
	}

	function fromApiSet(set: Awaited<ReturnType<typeof replaceSets>>[number]): UiSet {
		return {
			id: set.id ?? makeLocalNumericId(),
			reps: Number(set.reps),
			weight: set.weight,
			weightLeft: set.weight_left ?? undefined,
			weightRight: set.weight_right ?? undefined,
			durationSeconds: set.duration_seconds ?? undefined
		};
	}

	async function replaceExerciseSets(
		exerciseId: number,
		nextSetsForExercise: (sets: UiSet[]) => UiSet[]
	) {
		const session = get(state.currentSession);
		if (!session) return;
		const exercise = session.exercises.find((e) => e.id === exerciseId);
		if (!exercise) return;

		const nextSets = nextSetsForExercise(exercise.sets);
		const nextSession = {
			...session,
			exercises: session.exercises.map((e) => (e.id === exerciseId ? { ...e, sets: nextSets } : e))
		};

		state.error.set(null);
		state.currentSession.set(nextSession);
		await persistInProgressSession(state);

		if (get(state.offlineMode) || session.id < 0 || exerciseId < 0) return;

		try {
			state.loading.set(true);
			const replaced = await replaceSets(exerciseId, nextSets.map(toSetRequest));
			state.currentSession.update((current) => {
				if (!current) return current;
				return {
					...current,
					exercises: current.exercises.map((e) =>
						e.id === exerciseId ? { ...e, sets: replaced.map(fromApiSet) } : e
					)
				};
			});
			await persistInProgressSession(state);
		} catch (e) {
			if (isNetworkFailure(e)) {
				setOffline(state);
				await persistInProgressSession(state);
			} else {
				state.currentSession.set(session);
				await persistInProgressSession(state);
				state.error.set(getErrorMessage(e));
			}
		} finally {
			state.loading.set(false);
		}
	}

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
								id: created.id ?? 0,
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

	async function updateSet(exerciseId: number, setId: number, patch: Omit<UiSet, 'id'>) {
		await replaceExerciseSets(exerciseId, (sets) =>
			sets.map((set) => (set.id === setId ? { ...patch, id: setId } : set))
		);
	}

	async function removeSet(exerciseId: number, setId: number) {
		await replaceExerciseSets(exerciseId, (sets) => sets.filter((set) => set.id !== setId));
	}

	return { addSet, markExerciseDone, removeSet, updateSet } satisfies ExerciseSetActions;
}
