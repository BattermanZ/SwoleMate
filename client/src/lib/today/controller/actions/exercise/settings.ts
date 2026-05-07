import { endExercise } from '$lib/api';
import { createId } from '$lib/utils/id';
import { get } from 'svelte/store';
import { persistInProgressSession, setOffline } from '../../offline';
import type { TodayState } from '../../state';
import { getErrorMessage, isNetworkFailure } from '../../utils';
import { trackingFieldsSetting } from '$lib/today/tracking';

export type ExerciseSettingsActions = {
	updateExerciseNotes: (exerciseId: number, notes: string) => void;
	addExerciseSetting: (exerciseId: number, key: string, value: string) => void;
	removeExerciseSetting: (exerciseId: number, settingId: string) => void;
	updateExerciseSetting: (
		exerciseId: number,
		settingId: string,
		key: string,
		value: string
	) => void;
	updateExerciseTracking: (
		exerciseId: number,
		fields: { reps: boolean; time: boolean; weight: boolean }
	) => void;
};

export function createExerciseSettingsActions(args: {
	state: TodayState;
	schedulePersist: () => void;
}) {
	const { state, schedulePersist } = args;

	const syncTimers = new Map<number, number>();

	function scheduleExerciseSync(exerciseId: number) {
		if (typeof window === 'undefined') return;
		if (get(state.offlineMode) || exerciseId < 0) {
			schedulePersist();
			return;
		}
		const existing = syncTimers.get(exerciseId);
		if (existing) window.clearTimeout(existing);
		const timer = window.setTimeout(() => void syncExercise(exerciseId), 650);
		syncTimers.set(exerciseId, timer);
	}

	async function syncExercise(exerciseId: number) {
		if (get(state.offlineMode) || exerciseId < 0) return;
		const session = get(state.currentSession);
		if (!session) return;
		const ex = session.exercises.find((e) => e.id === exerciseId);
		if (!ex) return;

		try {
			await endExercise(exerciseId, {
				end_time: ex.endedAt,
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
		} catch (e) {
			if (isNetworkFailure(e)) {
				setOffline(state);
				await persistInProgressSession(state);
			} else {
				state.error.set(getErrorMessage(e));
			}
		}
	}

	function updateExerciseNotes(exerciseId: number, notes: string) {
		const session = get(state.currentSession);
		if (!session) return;
		state.currentSession.set({
			...session,
			exercises: session.exercises.map((e) => (e.id === exerciseId ? { ...e, notes } : e))
		});
		scheduleExerciseSync(exerciseId);
	}

	function addExerciseSetting(exerciseId: number, key: string, value: string) {
		const session = get(state.currentSession);
		if (!session) return;
		state.currentSession.set({
			...session,
			exercises: session.exercises.map((e) => {
				if (e.id !== exerciseId) return e;
				return {
					...e,
					settings: [...e.settings, { id: createId('setting'), key, value }]
				};
			})
		});
		scheduleExerciseSync(exerciseId);
	}

	function removeExerciseSetting(exerciseId: number, settingId: string) {
		const session = get(state.currentSession);
		if (!session) return;
		state.currentSession.set({
			...session,
			exercises: session.exercises.map((e) => {
				if (e.id !== exerciseId) return e;
				return {
					...e,
					settings: e.settings.filter((s) => s.id !== settingId)
				};
			})
		});
		scheduleExerciseSync(exerciseId);
	}

	function updateExerciseSetting(
		exerciseId: number,
		settingId: string,
		key: string,
		value: string
	) {
		const session = get(state.currentSession);
		if (!session) return;
		state.currentSession.set({
			...session,
			exercises: session.exercises.map((e) => {
				if (e.id !== exerciseId) return e;
				return {
					...e,
					settings: e.settings.map((s) => (s.id === settingId ? { ...s, key, value } : s))
				};
			})
		});
		scheduleExerciseSync(exerciseId);
	}

	function updateExerciseTracking(
		exerciseId: number,
		fields: { reps: boolean; time: boolean; weight: boolean }
	) {
		const session = get(state.currentSession);
		if (!session) return;
		const nextFields = fields.reps || fields.time ? fields : { ...fields, reps: true };
		state.currentSession.set({
			...session,
			exercises: session.exercises.map((e) =>
				e.id === exerciseId
					? {
							...e,
							tracksReps: nextFields.reps,
							tracksTime: nextFields.time,
							tracksWeight: nextFields.weight,
							perSideWeight: nextFields.weight ? e.perSideWeight : false,
							splitWeight: nextFields.weight ? e.splitWeight : false
						}
					: e
			)
		});
		scheduleExerciseSync(exerciseId);
	}

	return {
		updateExerciseNotes,
		addExerciseSetting,
		removeExerciseSetting,
		updateExerciseSetting,
		updateExerciseTracking
	} satisfies ExerciseSettingsActions;
}
