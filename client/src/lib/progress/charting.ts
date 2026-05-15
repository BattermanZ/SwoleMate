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
	const root = document.documentElement;
	const isDark =
		root.getAttribute('data-theme') === 'dark' || root.classList.contains('dark');

	return {
		isDark,
		// Map to design-system tokens. ink-2 reads as strong body text in both modes;
		// secondary trend lines use --ink directly via resolveCssVarColor below.
		text: resolveCssVarColor('--ink-2', isDark ? '#d4c8b0' : '#443c30'),
		mutedText: resolveCssVarColor('--ink-soft', isDark ? '#978a72' : '#847562'),
		grid: isDark ? 'rgba(151, 138, 114, 0.28)' : 'rgba(132, 117, 98, 0.28)',
		primary: resolveCssVarColor('--clay', isDark ? '#ff7a3d' : '#ff5e1f'),
		// secondary = ink itself: dark-on-light, cream-on-dark — used for overlay trend lines
		secondary: resolveCssVarColor('--ink', isDark ? '#f3ece1' : '#18130d'),
		tertiary: resolveCssVarColor('--gold', isDark ? '#e3b64f' : '#d5a23a'),
		success: resolveCssVarColor('--sage', isDark ? '#6fa074' : '#4f7d54'),
		warning: resolveCssVarColor('--warn', isDark ? '#e0a460' : '#c87f1a'),
		error: resolveCssVarColor('--clay-text', isDark ? '#ff924d' : '#c83a05')
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
	// Watch BOTH legacy .dark class and the new data-theme attribute so charts re-render
	// regardless of which trigger flipped.
	observer.observe(document.documentElement, {
		attributes: true,
		attributeFilter: ['class', 'data-theme']
	});
	return observer;
}
