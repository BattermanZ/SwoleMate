import {
	Chart,
	registerables,
	type ChartConfiguration,
	type ChartOptions,
	type ChartTypeRegistry
} from 'chart.js';

Chart.register(...registerables);

export type ChartTheme = {
	isDark: boolean;
	text: string;
	mutedText: string;
	grid: string;
	primary: string;
	secondary: string;
	tertiary: string;
	success: string;
	warning: string;
	error: string;
};

export type AnyChart = Chart<keyof ChartTypeRegistry, unknown[], unknown>;

export function rgba(color: string, alpha: number): string {
	const rgbMatch = color.match(/^rgb\(\s*(?<r>\d+)\s*,\s*(?<g>\d+)\s*,\s*(?<b>\d+)\s*\)$/);
	if (rgbMatch?.groups) {
		const { r, g, b } = rgbMatch.groups;
		return `rgba(${r}, ${g}, ${b}, ${alpha})`;
	}

	const rgbaMatch = color.match(
		/^rgba\(\s*(?<r>\d+)\s*,\s*(?<g>\d+)\s*,\s*(?<b>\d+)\s*,\s*(?<a>[\d.]+)\s*\)$/
	);
	if (rgbaMatch?.groups) {
		const { r, g, b } = rgbaMatch.groups;
		return `rgba(${r}, ${g}, ${b}, ${alpha})`;
	}

	const hex = color.trim();
	if (/^#([0-9a-f]{3}|[0-9a-f]{6})$/i.test(hex)) {
		const normalized =
			hex.length === 4 ? `#${hex[1]}${hex[1]}${hex[2]}${hex[2]}${hex[3]}${hex[3]}` : hex;
		const r = parseInt(normalized.slice(1, 3), 16);
		const g = parseInt(normalized.slice(3, 5), 16);
		const b = parseInt(normalized.slice(5, 7), 16);
		return `rgba(${r}, ${g}, ${b}, ${alpha})`;
	}

	return color;
}

function resolveCssVarColor(varName: string, fallback: string): string {
	const styles = getComputedStyle(document.documentElement);
	const raw = styles.getPropertyValue(varName).trim();
	if (!raw) return fallback;

	const probe = document.createElement('span');
	probe.style.color = `var(${varName})`;
	probe.style.position = 'absolute';
	probe.style.left = '-9999px';
	probe.style.top = '0';
	probe.style.visibility = 'hidden';
	(document.body ?? document.documentElement).appendChild(probe);
	const resolved = getComputedStyle(probe).color;
	probe.remove();
	return resolved?.trim() || fallback;
}

export function readTheme(): ChartTheme {
	const isDark = document.documentElement.classList.contains('dark');

	return {
		isDark,
		text: isDark ? '#e2e8f0' : '#0f172a',
		mutedText: isDark ? 'rgba(226, 232, 240, 0.72)' : 'rgba(15, 23, 42, 0.65)',
		grid: isDark ? 'rgba(148, 163, 184, 0.22)' : 'rgba(15, 23, 42, 0.12)',
		primary: resolveCssVarColor('--color-primary-500', '#0ea5e9'),
		secondary: resolveCssVarColor('--color-secondary-500', '#14b8a6'),
		tertiary: resolveCssVarColor('--color-tertiary-500', '#f59e0b'),
		success: resolveCssVarColor('--color-success-500', '#22c55e'),
		warning: resolveCssVarColor('--color-warning-500', '#f59e0b'),
		error: resolveCssVarColor('--color-error-500', '#ef4444')
	};
}

export function baseOptions(theme: ChartTheme): ChartOptions<keyof ChartTypeRegistry> {
	return {
		responsive: true,
		maintainAspectRatio: false,
		plugins: {
			legend: {
				labels: {
					color: theme.text
				}
			},
			tooltip: {
				backgroundColor: theme.isDark ? 'rgba(2, 6, 23, 0.9)' : 'rgba(255, 255, 255, 0.92)',
				titleColor: theme.text,
				bodyColor: theme.text,
				borderColor: theme.grid,
				borderWidth: 1
			}
		},
		scales: {
			x: {
				ticks: { color: theme.mutedText },
				grid: { color: theme.grid }
			},
			y: {
				ticks: { color: theme.mutedText },
				grid: { color: theme.grid }
			}
		}
	} as unknown as ChartOptions<keyof ChartTypeRegistry>;
}

export function upsertChart(
	current: AnyChart | null,
	canvas: HTMLCanvasElement | null,
	config: ChartConfiguration
): AnyChart | null {
	if (!canvas) return current;
	current?.destroy();
	return new Chart(canvas, config) as unknown as AnyChart;
}

export function formatMonthLabel(month: string): string {
	const [year, monthStr] = month.split('-');
	const y = Number(year);
	const m = Number(monthStr);
	if (!Number.isFinite(y) || !Number.isFinite(m)) return month;
	const date = new Date(Date.UTC(y, m - 1, 1));
	const label = date.toLocaleString(undefined, { month: 'short' });
	return `${label} ${String(y).slice(-2)}`;
}

export function sqliteWeekKeyToTimestamp(weekKey: string): number | null {
	const match = weekKey.match(/^(?<year>\d{4})-(?:W)?(?<week>\d{1,2})$/);
	if (!match?.groups) return null;

	const year = Number(match.groups.year);
	const week = Number(match.groups.week);
	if (!Number.isInteger(year) || !Number.isInteger(week) || week < 0 || week > 53) return null;

	const janFirst = new Date(Date.UTC(year, 0, 1));
	if (week === 0) return janFirst.getTime();

	const janFirstDay = janFirst.getUTCDay();
	const daysUntilFirstMonday = janFirstDay === 1 ? 0 : janFirstDay === 0 ? 1 : 8 - janFirstDay;
	const weekStart = new Date(janFirst);
	weekStart.setUTCDate(janFirst.getUTCDate() + daysUntilFirstMonday + (week - 1) * 7);
	return weekStart.getTime();
}

export function observeTheme(onChange: () => void): MutationObserver {
	const observer = new MutationObserver(() => onChange());
	observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] });
	return observer;
}
