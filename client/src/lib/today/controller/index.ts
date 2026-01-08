import { createTodayActions } from './actions';
import { createTodayState } from './state';

export function createTodayController() {
	const state = createTodayState();
	const actions = createTodayActions(state);

	return {
		// stores
		currentSession: state.currentSession,
		elapsedLabel: state.elapsedLabel,
		endModalOpen: state.endModalOpen,
		endMood: state.endMood,
		endNotes: state.endNotes,
		error: state.error,
		exerciseQuery: state.exerciseQuery,
		loading: state.loading,
		notice: state.notice,
		offlineMode: state.offlineMode,
		openExerciseId: state.openExerciseId,
		pendingSyncCount: state.pendingSyncCount,
		quickPicks: state.quickPicks,
		recentSessions: state.recentSessions,
		sessionNotes: state.sessionNotes,
		suggestions: state.suggestions,
		totalSets: state.totalSets,
		totalVolumeKg: state.totalVolumeKg,

		// actions
		...actions
	};
}
