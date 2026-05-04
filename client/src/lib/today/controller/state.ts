import { EXERCISE_LIBRARY } from '$lib/mocks/today';
import type { UiMood, UiSession } from '$lib/today/types';
import { derived, writable } from 'svelte/store';
import {
	calculateElapsedLabel,
	calculateTotalDurationSeconds,
	calculateTotalSets,
	calculateTotalVolumeKg
} from './metrics';
import { getQuickPicks, getSuggestions } from './suggestions';

export function createTodayState() {
	const nowMs = writable(Date.now());

	const currentSession = writable<UiSession | null>(null);
	const recentSessions = writable<UiSession[]>([]);
	const openExerciseId = writable<number | null>(null);

	const exerciseQuery = writable('');
	const sessionNotes = writable('');
	const endMood = writable<UiMood | null>(null);
	const endModalOpen = writable(false);
	const endNotes = writable('');
	const loading = writable(false);
	const error = writable<string | null>(null);
	const notice = writable<string | null>(null);
	const offlineMode = writable(false);
	const pendingSyncCount = writable(0);

	const exerciseLibrary = writable<string[]>(EXERCISE_LIBRARY);

	const elapsedLabel = derived([nowMs, currentSession], ([$nowMs, $currentSession]) =>
		calculateElapsedLabel($nowMs, $currentSession)
	);

	const totalSets = derived(currentSession, ($currentSession) =>
		calculateTotalSets($currentSession)
	);

	const totalVolumeKg = derived(currentSession, ($currentSession) =>
		calculateTotalVolumeKg($currentSession)
	);

	const totalDurationSeconds = derived(currentSession, ($currentSession) =>
		calculateTotalDurationSeconds($currentSession)
	);

	const quickPicks = derived(recentSessions, ($recentSessions) => getQuickPicks($recentSessions));

	const suggestions = derived(
		[exerciseQuery, recentSessions, currentSession, exerciseLibrary],
		([$exerciseQuery, $recentSessions, $currentSession, $exerciseLibrary]) =>
			getSuggestions($exerciseQuery, $recentSessions, $currentSession, $exerciseLibrary)
	);

	return {
		// internal store
		nowMs,

		// state stores
		currentSession,
		recentSessions,
		openExerciseId,
		exerciseQuery,
		sessionNotes,
		endMood,
		endModalOpen,
		endNotes,
		loading,
		error,
		notice,
		offlineMode,
		pendingSyncCount,
		exerciseLibrary,

		// derived stores
		elapsedLabel,
		totalSets,
		totalVolumeKg,
		totalDurationSeconds,
		quickPicks,
		suggestions
	};
}

export type TodayState = ReturnType<typeof createTodayState>;
