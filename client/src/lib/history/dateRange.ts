export type DateRangePreset = 'all' | '30d' | '90d' | '365d' | 'custom';

export type DateRange = {
	startInclusiveMs?: number;
	endExclusiveMs?: number;
};

const MS_PER_DAY = 24 * 60 * 60 * 1000;

function parseLocalDateInput(value: string): { year: number; month: number; day: number } | null {
	const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value.trim());
	if (!match) return null;
	const year = Number(match[1]);
	const month = Number(match[2]);
	const day = Number(match[3]);
	if (!Number.isFinite(year) || !Number.isFinite(month) || !Number.isFinite(day)) return null;
	if (month < 1 || month > 12) return null;
	if (day < 1 || day > 31) return null;
	return { year, month, day };
}

function localDayStartMs(value: string): number | null {
	const parsed = parseLocalDateInput(value);
	if (!parsed) return null;
	const date = new Date(parsed.year, parsed.month - 1, parsed.day, 0, 0, 0, 0);
	const ms = date.getTime();
	return Number.isFinite(ms) ? ms : null;
}

export function resolveDateRange(
	preset: DateRangePreset,
	from: string,
	to: string,
	nowMs: number = Date.now()
): DateRange {
	if (preset === 'all') return {};

	if (preset !== 'custom') {
		const days = preset === '30d' ? 30 : preset === '90d' ? 90 : preset === '365d' ? 365 : 0;
		if (!days) return {};
		return { startInclusiveMs: nowMs - days * MS_PER_DAY };
	}

	const startInclusiveMs = localDayStartMs(from) ?? undefined;
	const toStartMs = localDayStartMs(to);
	const endExclusiveMs = toStartMs !== null ? toStartMs + MS_PER_DAY : undefined;

	return { startInclusiveMs, endExclusiveMs };
}

export function isWithinRange(isoDateTime: string, range: DateRange): boolean {
	const t = new Date(isoDateTime).getTime();
	if (!Number.isFinite(t)) return true;
	if (range.startInclusiveMs !== undefined && t < range.startInclusiveMs) return false;
	if (range.endExclusiveMs !== undefined && t >= range.endExclusiveMs) return false;
	return true;
}
