import { EXERCISE_LIBRARY } from '$lib/mocks/today';
import { getExerciseTypes } from '$lib/api';
import type { TodayState } from '../state';

export async function hydrateExerciseLibrary(state: TodayState) {
	try {
		const types = await getExerciseTypes();
		const merged = new Set<string>([...EXERCISE_LIBRARY, ...types]);
		state.exerciseLibrary.set(Array.from(merged).sort((a, b) => a.localeCompare(b)));
	} catch {
		// ignore: local library is good enough
	}
}
