import type { ChangelogEntry } from '$lib/changelog';

/**
 * Compare two semver strings numerically. Missing or non-numeric parts count as
 * 0. Returns >0 if a>b, 0 if equal, <0 if a<b.
 */
export function compareVersions(a: string, b: string): number {
	const pa = a.split('.');
	const pb = b.split('.');
	const len = Math.max(pa.length, pb.length);
	for (let i = 0; i < len; i++) {
		const na = Number.parseInt(pa[i] ?? '0', 10) || 0;
		const nb = Number.parseInt(pb[i] ?? '0', 10) || 0;
		if (na !== nb) return na - nb;
	}
	return 0;
}

/**
 * Given the last-seen version (or null for a first-ever visit) and the
 * changelog, return the entries the user has not seen yet, newest first. A
 * first-ever visit (null) has seen nothing, so every entry counts as unseen.
 */
export function entriesToShow(
	lastSeen: string | null,
	changelog: ChangelogEntry[]
): ChangelogEntry[] {
	return changelog.filter((e) => lastSeen === null || compareVersions(e.version, lastSeen) > 0);
}
