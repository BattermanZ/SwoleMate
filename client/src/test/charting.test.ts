import { describe, expect, it } from 'vitest';
import { formatMonthLabel, observeTheme, readTheme, rgba } from '$lib/progress/charting';

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

	it('reads theme defaults and reacts to dark mode class', () => {
		document.documentElement.classList.remove('dark');
		let theme = readTheme();
		expect(theme.isDark).toBe(false);
		expect(theme.text).toBe('#0f172a');

		document.documentElement.classList.add('dark');
		theme = readTheme();
		expect(theme.isDark).toBe(true);
		expect(theme.text).toBe('#e2e8f0');
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
