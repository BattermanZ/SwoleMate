/**
 * User-facing release notes, newest first. Hand-edited at release time. Keep
 * the wording plain and friendly — these are read by people, not developers.
 */
export interface ChangelogEntry {
	/** semver, e.g. "3.2.0" */
	version: string;
	/** ISO date, e.g. "2026-06-25" */
	date: string;
	/** short headline */
	title: string;
	/** plain-language bullet points */
	items: string[];
}

export const CHANGELOG: ChangelogEntry[] = [
	{
		version: '3.2.0',
		date: '2026-06-25',
		title: 'Your training calendar, now on mobile',
		items: [
			'See your whole year on your phone — the training calendar is now in the Progress tab on mobile. Swipe across it to look back through earlier months.',
			'The rest-timer chime no longer interrupts your music or podcasts.'
		]
	}
];
