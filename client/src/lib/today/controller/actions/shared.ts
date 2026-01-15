import { persistInProgressSession } from '../offline';
import type { TodayState } from '../state';
import { get } from 'svelte/store';

export function resetLocalSessionUi(state: TodayState) {
	state.exerciseQuery.set('');
	state.endMood.set(null);
	state.endNotes.set('');
	state.endModalOpen.set(false);
}

export function createPersistScheduler(state: TodayState) {
	let persistTimer: number | null = null;

	function schedulePersist() {
		if (typeof window === 'undefined') return;
		if (persistTimer) window.clearTimeout(persistTimer);
		persistTimer = window.setTimeout(() => {
			persistTimer = null;
			void persistInProgressSession(state);
		}, 450);
	}

	return { schedulePersist };
}

export function attachSessionNotesPersistence(state: TodayState, schedulePersist: () => void) {
	state.sessionNotes.subscribe((notes) => {
		const session = get(state.currentSession);
		const offlineMode = get(state.offlineMode);
		if (!session) return;
		if (session.notes === notes) return;
		state.currentSession.set({ ...session, notes });
		if (offlineMode || session.id < 0) schedulePersist();
	});
}
