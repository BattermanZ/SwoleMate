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
			const stats = await getVolumeStats(exerciseName);
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
