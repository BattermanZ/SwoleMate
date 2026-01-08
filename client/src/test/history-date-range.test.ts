import { describe, expect, it } from 'vitest';
import { isWithinRange, resolveDateRange } from '$lib/history/dateRange';

describe('history date range', () => {
	it('treats end date as inclusive (endExclusive next day)', () => {
		const range = resolveDateRange('custom', '2026-01-01', '2026-01-01', Date.UTC(2026, 0, 10));
		const start = new Date(2026, 0, 1, 0, 0, 0, 0).toISOString();
		const end = new Date(2026, 0, 1, 23, 59, 59, 999).toISOString();
		const next = new Date(2026, 0, 2, 0, 0, 0, 0).toISOString();
		expect(isWithinRange(start, range)).toBe(true);
		expect(isWithinRange(end, range)).toBe(true);
		expect(isWithinRange(next, range)).toBe(false);
	});

	it('accepts open-ended custom ranges', () => {
		const onlyFrom = resolveDateRange('custom', '2026-01-05', '', Date.UTC(2026, 0, 10));
		const beforeFrom = new Date(2026, 0, 4, 23, 59, 59, 999).toISOString();
		const fromStart = new Date(2026, 0, 5, 0, 0, 0, 0).toISOString();
		expect(isWithinRange(beforeFrom, onlyFrom)).toBe(false);
		expect(isWithinRange(fromStart, onlyFrom)).toBe(true);

		const onlyTo = resolveDateRange('custom', '', '2026-01-05', Date.UTC(2026, 0, 10));
		const toEnd = new Date(2026, 0, 5, 23, 59, 59, 999).toISOString();
		const afterTo = new Date(2026, 0, 6, 0, 0, 0, 0).toISOString();
		expect(isWithinRange(toEnd, onlyTo)).toBe(true);
		expect(isWithinRange(afterTo, onlyTo)).toBe(false);
	});
});
