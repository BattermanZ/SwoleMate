import { describe, expect, it } from 'vitest';
import {
	formatMonthLabel,
	observeTheme,
	readTheme,
	rgba,
	sqliteWeekKeyToTimestamp
} from '$lib/progress/charting';

describe('progress charting helpers', () => {
	it('converts rgb, rgba, and hex colors to rgba', () => {
		expect(rgba('rgb(10, 20, 30)', 0.5)).toBe('rgba(10, 20, 30, 0.5)');
		expect(rgba('rgba(1, 2, 3, 0.9)', 0.25)).toBe('rgba(1, 2, 3, 0.25)');
		expect(rgba('#0a141e', 0.75)).toBe('rgba(10, 20, 30, 0.75)');
		expect(rgba('#abc', 0.4)).toBe('rgba(170, 187, 204, 0.4)');
	});

	it('returns original color when format is unsupported', () => {
		expect(rgba('hsl(200, 10%, 10%)', 0.4)).toBe('hsl(200, 10%, 10%)');
	});

	it('formats month labels and preserves invalid values', () => {
		expect(formatMonthLabel('2026-03')).toMatch(/26$/);
		expect(formatMonthLabel('not-a-month')).toBe('not-a-month');
	});

	it('converts SQLite week keys to week start timestamps', () => {
		expect(sqliteWeekKeyToTimestamp('2026-15')).toBe(Date.UTC(2026, 3, 13));
		expect(sqliteWeekKeyToTimestamp('2026-W15')).toBe(Date.UTC(2026, 3, 13));
		expect(sqliteWeekKeyToTimestamp('2026-00')).toBe(Date.UTC(2026, 0, 1));
		expect(sqliteWeekKeyToTimestamp('not-a-week')).toBeNull();
	});

	it('reads theme and reacts to dark mode (via class or data-theme)', () => {
		document.documentElement.classList.remove('dark');
		document.documentElement.removeAttribute('data-theme');
		let theme = readTheme();
		expect(theme.isDark).toBe(false);
		// Default fallback for --ink-2 in light mode (jsdom has no real CSS vars resolved).
		expect(theme.text).toBe('#443c30');

		document.documentElement.classList.add('dark');
		theme = readTheme();
		expect(theme.isDark).toBe(true);
		expect(theme.text).toBe('#d4c8b0');

		document.documentElement.classList.remove('dark');
		document.documentElement.setAttribute('data-theme', 'dark');
		theme = readTheme();
		expect(theme.isDark).toBe(true);
		document.documentElement.removeAttribute('data-theme');
	});

	it('observes class changes on documentElement', async () => {
		let calls = 0;
		const observer = observeTheme(() => {
			calls += 1;
		});

		document.documentElement.classList.toggle('dark');
		await Promise.resolve();

		expect(calls).toBeGreaterThan(0);
		observer.disconnect();
	});
});
