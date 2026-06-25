import { describe, it, expect } from 'vitest';
import { compareVersions, entriesToShow, formatReleaseDate } from '$lib/whatsNew';
import type { ChangelogEntry } from '$lib/changelog';

const log: ChangelogEntry[] = [
	{ version: '3.2.0', date: '2026-06-25', title: 'C', features: ['c'] },
	{ version: '3.1.2', date: '2026-05-01', title: 'B', fixes: ['b'] },
	{ version: '3.1.0', date: '2026-04-01', title: 'A', features: ['a'], fixes: ['a2'] }
];

describe('compareVersions', () => {
	it('orders by major, minor, then patch', () => {
		expect(compareVersions('3.2.0', '3.1.9')).toBeGreaterThan(0);
		expect(compareVersions('3.1.0', '3.2.0')).toBeLessThan(0);
		expect(compareVersions('4.0.0', '3.9.9')).toBeGreaterThan(0);
	});

	it('returns 0 for equal versions', () => {
		expect(compareVersions('3.2.0', '3.2.0')).toBe(0);
	});

	it('treats missing parts as 0', () => {
		expect(compareVersions('3.2', '3.2.0')).toBe(0);
		expect(compareVersions('3.2.1', '3.2')).toBeGreaterThan(0);
	});

	it('treats non-numeric parts as 0', () => {
		expect(compareVersions('3.x.0', '3.0.0')).toBe(0);
	});
});

describe('entriesToShow', () => {
	it('shows every entry on first-ever visit (lastSeen null)', () => {
		expect(entriesToShow(null, log)).toEqual(log);
	});

	it('shows nothing on first-ever visit when the changelog is empty', () => {
		expect(entriesToShow(null, [])).toEqual([]);
	});

	it('shows only entries newer than lastSeen, newest first', () => {
		expect(entriesToShow('3.1.2', log)).toEqual([log[0]]);
	});

	it('stacks all entries when multiple versions were skipped', () => {
		expect(entriesToShow('3.1.0', log)).toEqual([log[0], log[1]]);
	});

	it('shows nothing when lastSeen equals the newest entry', () => {
		expect(entriesToShow('3.2.0', log)).toEqual([]);
	});

	it('shows nothing when lastSeen is newer than every entry', () => {
		expect(entriesToShow('9.9.9', log)).toEqual([]);
	});

	it('shows nothing for an empty changelog', () => {
		expect(entriesToShow('1.0.0', [])).toEqual([]);
	});
});

describe('formatReleaseDate', () => {
	it('formats a date-only ISO string as day month year', () => {
		expect(formatReleaseDate('2026-06-25')).toBe('25 Jun 2026');
	});

	it('never shifts the day (no UTC parsing), at either end of the year', () => {
		expect(formatReleaseDate('2026-01-01')).toBe('1 Jan 2026');
		expect(formatReleaseDate('2026-12-31')).toBe('31 Dec 2026');
	});

	it('returns the input unchanged when it is not a plain date', () => {
		expect(formatReleaseDate('not-a-date')).toBe('not-a-date');
	});
});
