import { endExercise, replaceSets } from '$lib/api';
import { get } from 'svelte/store';
import { persistInProgressSession, setOffline } from '../../offline';
import type { TodayState } from '../../state';
import { getErrorMessage, isNetworkFailure } from '../../utils';

export type ExerciseWeightModeActions = {
	toggleExercisePerSideWeight: (exerciseId: number, enabled: boolean) => Promise<void>;
	toggleExerciseSplitWeight: (exerciseId: number, enabled: boolean) => Promise<void>;
};

export function createExerciseWeightModeActions(args: {
	state: TodayState;
	refreshFromBackend: () => Promise<void>;
}) {
	const { state, refreshFromBackend } = args;

	async function toggleExercisePerSideWeight(exerciseId: number, enabled: boolean) {
		const session = get(state.currentSession);
		if (!session) return;
		const ex = session.exercises.find((e) => e.id === exerciseId);
		if (!ex || ex.status === 'done') return;
		if (enabled === ex.perSideWeight) return;

		let nextSets = ex.sets;
		let nextSplit = ex.splitWeight;

		if (!enabled) {
			nextSplit = false;
			nextSets = ex.sets.map((s) => {
				if (!ex.perSideWeight) return s;
				if (!ex.splitWeight) {
					return { ...s, weight: s.weight * 2, weightLeft: undefined, weightRight: undefined };
				}
				const left = s.weightLeft ?? s.weight;
				const right = s.weightRight ?? s.weight;
				return { ...s, weight: left + right, weightLeft: undefined, weightRight: undefined };
			});
		} else {
			nextSets = ex.sets.map((s) => ({
				...s,
				weight: s.weight / 2,
				weightLeft: undefined,
				weightRight: undefined
			}));
		}

		state.currentSession.set({
			...session,
			exercises: session.exercises.map((e) =>
				e.id === exerciseId
					? { ...e, perSideWeight: enabled, splitWeight: nextSplit, sets: nextSets }
					: e
			)
		});

		if (get(state.offlineMode) || session.id < 0 || exerciseId < 0) {
			await persistInProgressSession(state);
			return;
		}

		try {
			await endExercise(exerciseId, {
				end_time: ex.endedAt,
				notes: ex.notes || undefined,
				per_side_weight: enabled,
				split_weight: nextSplit,
				settings: ex.settings.map((s) => ({ key: s.key, value: s.value }))
			});

			if (nextSets.length) {
				const replaced = await replaceSets(
					exerciseId,
					nextSets.map((s) => ({
						reps: s.reps,
						weight: s.weight,
						weight_left: s.weightLeft,
						weight_right: s.weightRight,
						notes: undefined
					}))
				);

				const updated = get(state.currentSession);
				if (!updated) return;
				state.currentSession.set({
					...updated,
					exercises: updated.exercises.map((e) =>
						e.id === exerciseId
							? {
									...e,
									sets: replaced.map((s) => ({
										id: s.id ?? 0,
										reps: Number(s.reps),
										weight: s.weight,
										weightLeft: s.weight_left ?? undefined,
										weightRight: s.weight_right ?? undefined
									}))
								}
							: e
					)
				});
			}
		} catch (e) {
			if (isNetworkFailure(e)) {
				setOffline(state);
				await persistInProgressSession(state);
			} else {
				state.error.set(getErrorMessage(e));
				await refreshFromBackend();
			}
		}
	}

	async function toggleExerciseSplitWeight(exerciseId: number, enabled: boolean) {
		const session = get(state.currentSession);
		if (!session) return;
		const ex = session.exercises.find((e) => e.id === exerciseId);
		if (!ex || ex.status === 'done') return;
		if (!ex.perSideWeight) return;
		if (enabled === ex.splitWeight) return;

		const nextSets = ex.sets.map((s) => {
			if (!enabled) {
				const left = s.weightLeft ?? s.weight;
				const right = s.weightRight ?? s.weight;
				return {
					...s,
					weight: Math.max(left, right),
					weightLeft: undefined,
					weightRight: undefined
				};
			}
			return {
				...s,
				weightLeft: s.weightLeft ?? s.weight,
				weightRight: s.weightRight ?? s.weight
			};
		});

		state.currentSession.set({
			...session,
			exercises: session.exercises.map((e) =>
				e.id === exerciseId ? { ...e, splitWeight: enabled, sets: nextSets } : e
			)
		});

		if (get(state.offlineMode) || session.id < 0 || exerciseId < 0) {
			await persistInProgressSession(state);
			return;
		}

		try {
			await endExercise(exerciseId, {
				end_time: ex.endedAt,
				notes: ex.notes || undefined,
				per_side_weight: true,
				split_weight: enabled,
				settings: ex.settings.map((s) => ({ key: s.key, value: s.value }))
			});

			if (nextSets.length) {
				const replaced = await replaceSets(
					exerciseId,
					nextSets.map((s) => ({
						reps: s.reps,
						weight: s.weight,
						weight_left: s.weightLeft,
						weight_right: s.weightRight,
						notes: undefined
					}))
				);

				const updated = get(state.currentSession);
				if (!updated) return;
				state.currentSession.set({
					...updated,
					exercises: updated.exercises.map((e) =>
						e.id === exerciseId
							? {
									...e,
									sets: replaced.map((s) => ({
										id: s.id ?? 0,
										reps: Number(s.reps),
										weight: s.weight,
										weightLeft: s.weight_left ?? undefined,
										weightRight: s.weight_right ?? undefined
									}))
								}
							: e
					)
				});
			}
		} catch (e) {
			if (isNetworkFailure(e)) {
				setOffline(state);
				await persistInProgressSession(state);
			} else {
				state.error.set(getErrorMessage(e));
				await refreshFromBackend();
			}
		}
	}

	return {
		toggleExercisePerSideWeight,
		toggleExerciseSplitWeight
	} satisfies ExerciseWeightModeActions;
}
