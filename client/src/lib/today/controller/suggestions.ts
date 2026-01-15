import type { UiSession } from '$lib/today/types';

export function getQuickPicks(sessions: UiSession[]): string[] {
	const picks: string[] = [];
	const seen = new Set<string>();
	for (const session of sessions) {
		for (const ex of session.exercises) {
			if (seen.has(ex.name)) continue;
			seen.add(ex.name);
			picks.push(ex.name);
		}
	}
	return picks.slice(0, 6);
}

export function getSuggestions(
	query: string,
	sessions: UiSession[],
	activeSession: UiSession | null,
	library: string[]
): string[] {
	const term = query.trim().toLowerCase();
	if (!term) return [];

	const recentSet = new Set(getQuickPicks(sessions));
	const inSession = new Set((activeSession?.exercises ?? []).map((e) => e.name.toLowerCase()));

	const matches = library.filter((name) => {
		if (inSession.has(name.toLowerCase())) return false;
		return name.toLowerCase().includes(term);
	});

	const MAX_SUGGESTIONS = 10;
	return matches
		.sort((a, b) => {
			const aRecent = recentSet.has(a);
			const bRecent = recentSet.has(b);
			if (aRecent && !bRecent) return -1;
			if (!aRecent && bRecent) return 1;

			const aStarts = a.toLowerCase().startsWith(term);
			const bStarts = b.toLowerCase().startsWith(term);
			if (aStarts && !bStarts) return -1;
			if (!aStarts && bStarts) return 1;

			return a.localeCompare(b);
		})
		.slice(0, MAX_SUGGESTIONS);
}
