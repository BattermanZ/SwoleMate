import { EXERCISE_LIBRARY } from '$lib/mocks/today';
import { getExerciseTypes } from '$lib/api';
import type { TodayState } from '../state';

export function mergeExerciseLibrary(baseTypes: string[], historyTypes: string[]): string[] {
	const byNormalizedName = new Map<string, string>();

	for (const name of baseTypes) {
		const trimmed = name.trim();
		if (!trimmed) continue;
		byNormalizedName.set(trimmed.toLowerCase(), trimmed);
	}

	for (const name of historyTypes) {
		const trimmed = name.trim();
		if (!trimmed) continue;
		byNormalizedName.set(trimmed.toLowerCase(), trimmed);
	}

	return Array.from(byNormalizedName.values()).sort((a, b) => a.localeCompare(b));
}

export async function hydrateExerciseLibrary(state: TodayState) {
	try {
		const types = await getExerciseTypes();
		state.exerciseLibrary.set(mergeExerciseLibrary(EXERCISE_LIBRARY, types));
	} catch {
		// ignore: local library is good enough
	}
}
