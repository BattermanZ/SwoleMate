import type { UiSession } from '$lib/today/types';

export function calculateElapsedLabel(nowMs: number, session: UiSession | null): string {
	if (!session) return '';
	const diffMs = Math.max(0, nowMs - new Date(session.startedAt).getTime());
	const minutes = Math.floor(diffMs / 60_000);
	if (minutes < 60) return `${minutes}m`;
	const hours = Math.floor(minutes / 60);
	const rem = minutes % 60;
	return rem ? `${hours}h ${rem}m` : `${hours}h`;
}

export function calculateTotalSets(session: UiSession | null): number {
	if (!session) return 0;
	return session.exercises.reduce((count, e) => count + e.sets.length, 0);
}

export function calculateTotalVolumeKg(session: UiSession | null): number {
	if (!session) return 0;
	return session.exercises.reduce(
		(total, e) =>
			total +
			e.sets.reduce((t, s) => {
				if (!e.perSideWeight) return t + s.reps * s.weight;
				if (!e.splitWeight) return t + s.reps * (s.weight * 2);
				const left = s.weightLeft ?? s.weight;
				const right = s.weightRight ?? s.weight;
				return t + s.reps * (left + right);
			}, 0),
		0
	);
}
