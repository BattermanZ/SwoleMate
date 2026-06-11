import { getVolumeStats } from '$lib/api';
import { get } from 'svelte/store';
import type { TodayState } from '../../state';

export type ExercisePrActions = {
	loadEstimated1RmBaseline: (name: string) => Promise<void>;
};

export function createExercisePrActions(args: { state: TodayState }): ExercisePrActions {
	const { state } = args;

	async function loadEstimated1RmBaseline(name: string) {
		const exerciseName = name.trim();
		if (!exerciseName || get(state.offlineMode)) return;

		const current = get(state.estimated1RmBaselines);
		if (Object.hasOwn(current, exerciseName)) return;

		try {
			// Exclude the active workout so the baseline reflects prior sessions only;
			// otherwise the current session's own sets would be compared against a
			// baseline that already contains them and the PR marker would vanish on reload.
			const activeSessionId = get(state.currentSession)?.id;
			const stats = await getVolumeStats(exerciseName, {
				excludeWorkoutId:
					activeSessionId != null && activeSessionId > 0 ? activeSessionId : undefined
			});
			state.estimated1RmBaselines.update((baselines) => ({
				...baselines,
				[exerciseName]: stats.personal_records.estimated_max_1rm
			}));
		} catch {
			state.estimated1RmBaselines.update((baselines) => ({ ...baselines, [exerciseName]: null }));
		}
	}

	return { loadEstimated1RmBaseline };
}
