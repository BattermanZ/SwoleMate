import { writable } from 'svelte/store';
import { CHANGELOG, type ChangelogEntry } from '$lib/changelog';
import { entriesToShow } from '$lib/whatsNew';
import { APP_VERSION } from '$lib/version';

const LAST_SEEN_KEY = 'swolemate.lastSeenVersion';

/** Entries currently shown in the What's New modal, or null when it's closed. */
export const whatsNewEntries = writable<ChangelogEntry[] | null>(null);

function writeLastSeen(version: string): void {
	try {
		localStorage.setItem(LAST_SEEN_KEY, version);
	} catch {
		// Storage unavailable (e.g. private mode) — nothing we can do; ignore.
	}
}

/**
 * Run once after the user is authenticated. Decides whether to auto-show the
 * changelog based on the per-device last-seen version, then marks the current
 * version as seen so it won't show again. First-ever visitors (no stored
 * version) see the full changelog once.
 */
export function maybeShowWhatsNew(): void {
	let lastSeen: string | null;
	try {
		lastSeen = localStorage.getItem(LAST_SEEN_KEY);
	} catch {
		return; // Storage unavailable — behave as "already seen".
	}

	const unseen = entriesToShow(lastSeen, CHANGELOG);
	if (unseen.length > 0) {
		whatsNewEntries.set(unseen);
	}
	writeLastSeen(APP_VERSION); // Mark current as seen (covers first-run seeding too).
}

/** Open the modal on demand (e.g. from the menu). Does not change last-seen. */
export function openWhatsNew(entries: ChangelogEntry[] = CHANGELOG): void {
	if (entries.length > 0) whatsNewEntries.set(entries);
}

/** Close the modal. */
export function closeWhatsNew(): void {
	whatsNewEntries.set(null);
}
