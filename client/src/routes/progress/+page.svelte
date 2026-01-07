<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import {
		Chart,
		registerables,
		type ChartConfiguration,
		type ChartOptions,
		type ChartTypeRegistry
	} from 'chart.js';
	import 'chartjs-adapter-date-fns';
	import { getWorkoutStats, getExerciseTypes, getVolumeStats, getExerciseProgress } from '$lib/api';
	import type { WorkoutStats, VolumeStats, ExerciseProgress } from '$lib/types';
	import { logger } from '$lib/logger';
	import { summarizeRepPrs, type RepPr } from '$lib/progress/repPrs';
	import SetPillsHybrid from '$lib/components/ui/SetPillsHybrid.svelte';
	import { formatDateRelative, formatTime } from '$lib/utils/date';

	Chart.register(...registerables);

	type ChartTheme = {
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

	let selectedExercise = '';
	let exerciseTypes: string[] = [];
	let workoutStats: WorkoutStats | null = null;
	let volumeStats: VolumeStats | null = null;
	let exerciseProgress: ExerciseProgress[] | null = null;

	let loadingOverall = false;
	let loadingExercise = false;
	let errorOverall: string | null = null;
	let errorExercise: string | null = null;

	let volumeCanvas: HTMLCanvasElement | null = null;
	let progressCanvas: HTMLCanvasElement | null = null;
	let feedbackCanvas: HTMLCanvasElement | null = null;
	let timeCanvas: HTMLCanvasElement | null = null;
	let durationCanvas: HTMLCanvasElement | null = null;
	let monthlyVolumeCanvas: HTMLCanvasElement | null = null;
	let monthlySessionsCanvas: HTMLCanvasElement | null = null;

	type AnyChart = Chart<keyof ChartTypeRegistry, unknown[], unknown>;

	let volumeChart: AnyChart | null = null;
	let progressChart: AnyChart | null = null;
	let feedbackChart: AnyChart | null = null;
	let timeDistributionChart: AnyChart | null = null;
	let durationDistributionChart: AnyChart | null = null;
	let monthlyVolumeChart: AnyChart | null = null;
	let monthlySessionsChart: AnyChart | null = null;

	let theme: ChartTheme = {
		isDark: false,
		text: '#0f172a',
		mutedText: 'rgba(15, 23, 42, 0.65)',
		grid: 'rgba(15, 23, 42, 0.12)',
		primary: '#0ea5e9',
		secondary: '#14b8a6',
		tertiary: '#f59e0b',
		success: '#22c55e',
		warning: '#f59e0b',
		error: '#ef4444'
	};

	function rgba(color: string, alpha: number): string {
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

	function getErrorMessage(e: unknown): string {
		if (e instanceof Error) return e.message;
		return 'Something went wrong';
	}

	function readTheme(): ChartTheme {
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

	function baseOptions(): ChartOptions<keyof ChartTypeRegistry> {
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

	function formatMonthLabel(month: string): string {
		const [year, monthStr] = month.split('-');
		const y = Number(year);
		const m = Number(monthStr);
		if (!Number.isFinite(y) || !Number.isFinite(m)) return month;
		const date = new Date(Date.UTC(y, m - 1, 1));
		const label = date.toLocaleString(undefined, { month: 'short' });
		return `${label} ${String(y).slice(-2)}`;
	}

	function destroyAllCharts() {
		volumeChart?.destroy();
		progressChart?.destroy();
		feedbackChart?.destroy();
		monthlySessionsChart?.destroy();
		timeDistributionChart?.destroy();
		durationDistributionChart?.destroy();
		monthlyVolumeChart?.destroy();
		volumeChart = null;
		progressChart = null;
		feedbackChart = null;
		monthlySessionsChart = null;
		timeDistributionChart = null;
		durationDistributionChart = null;
		monthlyVolumeChart = null;
	}

	function upsertChart(
		current: AnyChart | null,
		canvas: HTMLCanvasElement | null,
		config: ChartConfiguration
	): AnyChart | null {
		if (!canvas) return current;
		current?.destroy();
		return new Chart(canvas, config) as unknown as AnyChart;
	}

	function renderOverallCharts() {
		if (workoutStats) {
			const base = baseOptions();
			const basePlugins = (base.plugins ?? {}) as unknown as Record<string, unknown>;
			const baseScales = (base.scales ?? {}) as unknown as {
				x?: Record<string, unknown>;
				y?: Record<string, unknown>;
			};

			feedbackChart = upsertChart(feedbackChart, feedbackCanvas, {
				type: 'doughnut',
				data: {
					labels: ['Good', 'Neutral', 'Bad'],
					datasets: [
						{
							data: [
								workoutStats.feedback_distribution.good,
								workoutStats.feedback_distribution.neutral,
								workoutStats.feedback_distribution.bad
							],
							backgroundColor: [
								rgba(theme.success, 0.88),
								rgba(theme.warning, 0.88),
								rgba(theme.error, 0.88)
							],
							borderColor: theme.isDark ? 'rgba(2, 6, 23, 0.5)' : 'rgba(255, 255, 255, 0.8)',
							borderWidth: 2
						}
					]
				},
				options: {
					...base,
					cutout: '65%',
					plugins: {
						...basePlugins,
						legend: {
							position: 'bottom',
							labels: { color: theme.mutedText }
						}
					}
				}
			} as unknown as ChartConfiguration<'doughnut'>);

			const sessionsPerMonth = workoutStats.sessions_per_month ?? [];
			if (sessionsPerMonth.length) {
				monthlySessionsChart = upsertChart(monthlySessionsChart, monthlySessionsCanvas, {
					type: 'bar',
					data: {
						labels: sessionsPerMonth.map((m) => formatMonthLabel(m.month)),
						datasets: [
							{
								label: 'Sessions',
								data: sessionsPerMonth.map((m) => m.count),
								backgroundColor: rgba(theme.tertiary, theme.isDark ? 0.72 : 0.6),
								borderColor: rgba(theme.tertiary, theme.isDark ? 0.92 : 0.85),
								borderWidth: 1,
								borderRadius: 8
							}
						]
					},
					options: {
						...base,
						scales: {
							x: {
								...(baseScales.x ?? {}),
								title: { display: true, text: 'Month', color: theme.mutedText }
							},
							y: {
								...(baseScales.y ?? {}),
								beginAtZero: true,
								title: { display: true, text: 'Sessions', color: theme.mutedText }
							}
						}
					}
				});
			} else {
				monthlySessionsChart?.destroy();
				monthlySessionsChart = null;
			}

			const popularHours = [...workoutStats.popular_hours].sort(
				(a, b) => Number(a.hour) - Number(b.hour)
			);

			timeDistributionChart = upsertChart(timeDistributionChart, timeCanvas, {
				type: 'bar',
				data: {
					labels: popularHours.map((h) => `${h.hour}:00`),
					datasets: [
						{
							label: 'Workouts',
							data: popularHours.map((h) => h.count),
							backgroundColor: rgba(theme.primary, theme.isDark ? 0.72 : 0.62),
							borderColor: rgba(theme.primary, theme.isDark ? 0.92 : 0.85),
							borderWidth: 1,
							borderRadius: 8
						}
					]
				},
				options: {
					...base,
					scales: {
						x: {
							...(baseScales.x ?? {}),
							title: { display: true, text: 'Hour', color: theme.mutedText }
						},
						y: {
							...(baseScales.y ?? {}),
							beginAtZero: true,
							title: { display: true, text: 'Workouts', color: theme.mutedText }
						}
					}
				}
			});

			durationDistributionChart = upsertChart(durationDistributionChart, durationCanvas, {
				type: 'bar',
				data: {
					labels: workoutStats.duration_distribution.map((d) => `${d.range} min`),
					datasets: [
						{
							label: 'Workouts',
							data: workoutStats.duration_distribution.map((d) => d.count),
							backgroundColor: rgba(theme.secondary, theme.isDark ? 0.72 : 0.62),
							borderColor: rgba(theme.secondary, theme.isDark ? 0.92 : 0.85),
							borderWidth: 1,
							borderRadius: 8
						}
					]
				},
				options: {
					...base,
					scales: {
						x: {
							...(baseScales.x ?? {}),
							title: { display: true, text: 'Duration bucket', color: theme.mutedText }
						},
						y: {
							...(baseScales.y ?? {}),
							beginAtZero: true,
							title: { display: true, text: 'Workouts', color: theme.mutedText }
						}
					}
				}
			});
		}
	}

	function renderExerciseCharts() {
		if (!volumeStats) return;
		const base = baseOptions();
		const baseScales = (base.scales ?? {}) as unknown as {
			x?: Record<string, unknown>;
			y?: Record<string, unknown>;
		};

		volumeChart = upsertChart(volumeChart, volumeCanvas, {
			type: 'line',
			data: {
				labels: volumeStats.weekly_volume.map((v) => v.week),
				datasets: [
					{
						label: 'Weekly volume (kg)',
						data: volumeStats.weekly_volume.map((v) => v.total_volume),
						borderColor: theme.primary,
						backgroundColor: rgba(theme.primary, theme.isDark ? 0.22 : 0.14),
						pointRadius: 3,
						tension: 0.25,
						fill: false,
						yAxisID: 'y'
					},
					{
						label: 'Best estimated 1RM (kg)',
						data: volumeStats.weekly_volume.map((v) => v.max_estimated_1rm),
						borderColor: theme.tertiary,
						backgroundColor: rgba(theme.tertiary, theme.isDark ? 0.18 : 0.12),
						pointRadius: 3,
						tension: 0.25,
						fill: false,
						yAxisID: 'y1'
					}
				]
			},
			options: {
				...base,
				scales: {
					x: {
						...(baseScales.x ?? {}),
						title: { display: true, text: 'Week', color: theme.mutedText }
					},
					y: {
						...(baseScales.y ?? {}),
						beginAtZero: true,
						title: { display: true, text: 'Volume (kg)', color: theme.mutedText }
					},
					y1: {
						position: 'right',
						beginAtZero: true,
						ticks: { color: theme.mutedText },
						grid: { drawOnChartArea: false },
						title: { display: true, text: 'Estimated 1RM (kg)', color: theme.mutedText }
					}
				}
			}
		});

		if (exerciseProgress && exerciseProgress.length > 0) {
			const data = exerciseProgress
				.map((ep) => {
					const maxWeight = Math.max(...ep.sets.map((s) => Number(s.weight)));
					return {
						x: new Date(ep.exercise.start_time).getTime(),
						y: Number.isFinite(maxWeight) ? maxWeight : 0
					};
				})
				.filter((p) => p.y > 0);

			progressChart = upsertChart(progressChart, progressCanvas, {
				type: 'scatter',
				data: {
					datasets: [
						{
							label: 'Best weight per session (kg)',
							data,
							backgroundColor: rgba(theme.secondary, theme.isDark ? 0.28 : 0.22),
							borderColor: theme.secondary,
							pointBackgroundColor: theme.secondary,
							pointBorderColor: theme.isDark ? 'rgba(2, 6, 23, 0.7)' : 'rgba(255, 255, 255, 0.9)',
							pointBorderWidth: 1,
							showLine: true,
							tension: 0.25
						}
					]
				},
				options: {
					...baseOptions(),
					scales: {
						x: {
							type: 'time',
							time: {
								unit: 'week'
							},
							ticks: { color: theme.mutedText },
							grid: { color: theme.grid },
							title: { display: true, text: 'Date', color: theme.mutedText }
						},
						y: {
							...(baseScales.y ?? {}),
							beginAtZero: true,
							title: { display: true, text: 'Weight (kg)', color: theme.mutedText }
						}
					}
				}
			});
		} else {
			progressChart?.destroy();
			progressChart = null;
		}

		monthlyVolumeChart = upsertChart(monthlyVolumeChart, monthlyVolumeCanvas, {
			type: 'bar',
			data: {
				labels: volumeStats.monthly_volume.map((v) => v.month),
				datasets: [
					{
						label: 'Monthly volume (kg)',
						data: volumeStats.monthly_volume.map((v) => v.total_volume),
						backgroundColor: rgba(theme.primary, theme.isDark ? 0.7 : 0.58),
						borderColor: rgba(theme.primary, theme.isDark ? 0.92 : 0.85),
						borderWidth: 1,
						borderRadius: 8
					}
				]
			},
			options: {
				...base,
				scales: {
					x: {
						...(baseScales.x ?? {}),
						title: { display: true, text: 'Month', color: theme.mutedText }
					},
					y: {
						...(baseScales.y ?? {}),
						beginAtZero: true,
						title: { display: true, text: 'Volume (kg)', color: theme.mutedText }
					}
				}
			}
		});
	}

	async function loadOverall() {
		loadingOverall = true;
		errorOverall = null;
		try {
			workoutStats = await getWorkoutStats();
		} catch (e) {
			errorOverall = getErrorMessage(e);
			logger.error('progress', 'Error loading workout stats', { error: e });
		} finally {
			loadingOverall = false;
		}
	}

	async function loadExerciseTypes() {
		try {
			exerciseTypes = await getExerciseTypes();
			const stored = localStorage.getItem('progress.selectedExercise');
			selectedExercise =
				stored && exerciseTypes.includes(stored) ? stored : (exerciseTypes[0] ?? '');
		} catch (e) {
			logger.error('progress', 'Error loading exercise types', { error: e });
		}
	}

	async function loadExercise() {
		if (!selectedExercise) return;
		loadingExercise = true;
		errorExercise = null;
		try {
			localStorage.setItem('progress.selectedExercise', selectedExercise);
			[volumeStats, exerciseProgress] = await Promise.all([
				getVolumeStats(selectedExercise),
				getExerciseProgress(selectedExercise)
			]);
		} catch (e) {
			errorExercise = getErrorMessage(e);
			logger.error('progress', 'Error loading exercise data', { error: e, selectedExercise });
		} finally {
			loadingExercise = false;
		}
	}

	async function refreshAll() {
		await Promise.all([loadOverall(), loadExerciseTypes()]);
		await loadExercise();
		renderOverallCharts();
		renderExerciseCharts();
	}

	let lastLoadedExercise = '';
	let showAllRepPrs = false;
	let repPrs: RepPr[] = [];
	let recentExerciseSessions: ExerciseProgress[] = [];

	$: repPrs = volumeStats?.personal_records.rep_prs?.length
		? summarizeRepPrs(volumeStats.personal_records.rep_prs)
		: [];

	$: recentExerciseSessions = (exerciseProgress ?? []).slice(-5).reverse();

	let observer: MutationObserver | null = null;

	onMount(async () => {
		theme = readTheme();
		observer = new MutationObserver(() => {
			const next = readTheme();
			if (next.isDark !== theme.isDark) {
				theme = next;
				renderOverallCharts();
				renderExerciseCharts();
			}
		});
		observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] });

		await refreshAll();
		lastLoadedExercise = selectedExercise;
	});

	$: if (selectedExercise) {
		if (selectedExercise !== lastLoadedExercise) {
			lastLoadedExercise = selectedExercise;
			showAllRepPrs = false;
			void loadExercise().then(() => renderExerciseCharts());
		}
	}

	onDestroy(() => {
		destroyAllCharts();
		observer?.disconnect();
		observer = null;
	});

	$: if (workoutStats) renderOverallCharts();
	$: if (volumeStats) renderExerciseCharts();
</script>

<div class="space-y-6">
	<header
		class="relative overflow-hidden rounded-2xl border border-surface-200/50 dark:border-surface-700/50 bg-gradient-to-br from-primary-500/10 via-transparent to-tertiary-500/10 p-5 sm:p-6"
	>
		<div
			class="pointer-events-none absolute -top-24 -right-24 size-72 rounded-full blur-3xl bg-primary-500/15"
		></div>
		<div
			class="pointer-events-none absolute -bottom-24 -left-24 size-72 rounded-full blur-3xl bg-secondary-500/15"
		></div>

		<div class="relative flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
			<div class="space-y-1">
				<h1 class="text-3xl sm:text-4xl font-black tracking-tight">Progress</h1>
				<p class="text-sm sm:text-base opacity-80 max-w-prose">
					Trends, personal records, and consistency signals — tuned for quick scanning.
				</p>
			</div>

			<div class="flex flex-col sm:items-end gap-2">
				<button
					type="button"
					class="btn variant-soft"
					on:click={refreshAll}
					disabled={loadingOverall || loadingExercise}
				>
					Refresh
				</button>
				{#if errorOverall || errorExercise}
					<div class="text-sm text-error-500">{errorOverall ?? errorExercise}</div>
				{/if}
			</div>
		</div>

		{#if workoutStats}
			<div class="relative mt-5 grid gap-3 grid-cols-2 sm:grid-cols-4">
				<div class="card variant-glass-surface p-3 border-l-4 border-primary-500/70">
					<div class="text-xs font-semibold opacity-70">Total workouts</div>
					<div class="text-lg font-bold">{workoutStats.total_workouts}</div>
				</div>
				<div class="card variant-glass-surface p-3 border-l-4 border-secondary-500/70">
					<div class="text-xs font-semibold opacity-70">Workouts / week</div>
					<div class="text-lg font-bold">{workoutStats.workout_frequency.average_per_week}</div>
					{#if workoutStats.workout_frequency.trend !== undefined}
						<div class="text-xs opacity-70">
							{workoutStats.workout_frequency.trend > 0 ? '+' : ''}{workoutStats.workout_frequency
								.trend}
							last 4w
						</div>
					{/if}
				</div>
				<div class="card variant-glass-surface p-3 border-l-4 border-tertiary-500/70">
					<div class="text-xs font-semibold opacity-70">Avg duration</div>
					<div class="text-lg font-bold">{Math.round(workoutStats.average_duration_minutes)}m</div>
					{#if workoutStats.duration_trend !== undefined}
						<div class="text-xs opacity-70">
							{workoutStats.duration_trend > 0 ? '+' : ''}{Math.round(workoutStats.duration_trend)}m
							last 4w
						</div>
					{/if}
				</div>
				<div class="card variant-glass-surface p-3 border-l-4 border-success-500/70">
					<div class="text-xs font-semibold opacity-70">Focus exercise</div>
					<div class="text-lg font-bold truncate">{selectedExercise || '—'}</div>
				</div>
			</div>
		{:else if loadingOverall}
			<div class="relative mt-5 grid gap-3 grid-cols-2 sm:grid-cols-4">
				{#each [0, 1, 2, 3] as i (i)}
					<div class="card variant-glass-surface p-3 animate-pulse">
						<div class="h-3 w-24 bg-surface-200/60 dark:bg-surface-700/50 rounded"></div>
						<div class="mt-2 h-6 w-14 bg-surface-200/60 dark:bg-surface-700/50 rounded"></div>
					</div>
				{/each}
			</div>
		{/if}
	</header>

	<div class="grid gap-6 md:grid-cols-12">
		<section class="md:col-span-7 lg:col-span-8 space-y-4 min-w-0">
			<div class="card variant-glass-surface p-4 space-y-3 min-w-0">
				<div class="flex flex-col sm:flex-row sm:items-end sm:justify-between gap-3">
					<div>
						<h2 class="text-lg font-semibold tracking-tight">Exercise focus</h2>
						<p class="text-sm opacity-70">Pick an exercise to see volume, strength and PRs.</p>
					</div>
					<label class="block min-w-0">
						<span class="sr-only">Exercise</span>
						<select
							class="select w-full sm:w-72 min-w-0"
							bind:value={selectedExercise}
							disabled={loadingExercise}
						>
							{#if exerciseTypes.length === 0}
								<option value="">No exercises yet</option>
							{:else}
								{#each exerciseTypes as type}
									<option value={type}>{type}</option>
								{/each}
							{/if}
						</select>
					</label>
				</div>

				{#if volumeStats}
					<div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
						<div class="card variant-glass-surface p-3 border-l-4 border-primary-500/60">
							<div class="text-xs font-semibold opacity-70">All‑time max</div>
							<div class="text-lg font-bold">
								{volumeStats.personal_records.all_time_max_weight}kg
							</div>
						</div>
						<div class="card variant-glass-surface p-3 border-l-4 border-tertiary-500/60">
							<div class="text-xs font-semibold opacity-70">Estimated 1RM</div>
							<div class="text-lg font-bold">
								{volumeStats.personal_records.estimated_max_1rm}kg
							</div>
						</div>
						<div class="card variant-glass-surface p-3 border-l-4 border-secondary-500/60">
							<div class="text-xs font-semibold opacity-70">Max session volume</div>
							<div class="text-lg font-bold">{volumeStats.personal_records.max_volume}kg</div>
						</div>
					</div>

					{#if repPrs.length}
						<div
							class="rounded-xl border border-surface-200/50 bg-surface-50/60 p-3 dark:border-surface-700/50 dark:bg-surface-950/30"
						>
							<div class="flex items-center justify-between gap-3">
								<div class="text-sm font-semibold opacity-80">Rep PRs</div>
								{#if repPrs.length > 10}
									<button
										type="button"
										class="btn variant-ghost text-xs"
										on:click={() => (showAllRepPrs = !showAllRepPrs)}
									>
										{showAllRepPrs ? 'Show less' : `Show all (${repPrs.length})`}
									</button>
								{/if}
							</div>

							<div class="mt-2 flex flex-wrap gap-2">
								{#each showAllRepPrs ? repPrs : repPrs.slice(0, 10) as pr (pr.reps)}
									<span
										class="inline-flex overflow-hidden rounded-full border border-surface-200/50 dark:border-surface-700/60"
									>
										<span
											class="px-2 py-1 text-xs font-extrabold tracking-tight bg-secondary-500/25 text-surface-950 dark:text-surface-50"
											>{pr.reps} reps</span
										>
										<span
											class="px-2 py-1 text-xs font-extrabold tracking-tight bg-primary-500/30 text-surface-950 dark:text-surface-50 border-l border-surface-200/50 dark:border-surface-700/60"
											>{pr.weight}kg</span
										>
									</span>
								{/each}
							</div>
						</div>
					{/if}
				{:else if loadingExercise}
					<div class="card variant-glass-surface p-4 animate-pulse">
						<div class="h-4 w-36 bg-surface-200/60 dark:bg-surface-700/50 rounded"></div>
						<div class="mt-3 h-8 w-48 bg-surface-200/60 dark:bg-surface-700/50 rounded"></div>
					</div>
				{:else if errorExercise}
					<div class="alert variant-filled-error">{errorExercise}</div>
				{:else if selectedExercise}
					<div class="card variant-ghost p-4 text-center opacity-80">
						No progress data yet for <span class="font-semibold">{selectedExercise}</span>.
					</div>
				{/if}
			</div>

			{#if volumeStats}
				<div class="grid gap-4 lg:grid-cols-2 min-w-0">
					<div class="card variant-glass-surface p-4 min-w-0">
						<div class="flex items-start justify-between gap-3">
							<div>
								<h3 class="text-base font-semibold">Weekly volume + 1RM</h3>
								<p class="text-sm opacity-70">Volume and best estimated 1RM per week.</p>
							</div>
						</div>
						<div class="mt-3 h-72">
							<canvas bind:this={volumeCanvas}></canvas>
						</div>
					</div>

					<div class="card variant-glass-surface p-4 min-w-0">
						<div class="flex items-start justify-between gap-3">
							<div>
								<h3 class="text-base font-semibold">Best weight per session</h3>
								<p class="text-sm opacity-70">Tracks your top weight for the chosen exercise.</p>
							</div>
						</div>
						<div class="mt-3 h-72">
							<canvas bind:this={progressCanvas}></canvas>
						</div>
					</div>
				</div>

				<div class="card variant-glass-surface p-4 min-w-0">
					<div class="flex items-start justify-between gap-3">
						<div>
							<h3 class="text-base font-semibold">Monthly volume</h3>
							<p class="text-sm opacity-70">Smoother long-term trend.</p>
						</div>
					</div>
					<div class="mt-3 h-64">
						<canvas bind:this={monthlyVolumeCanvas}></canvas>
					</div>
				</div>

				<div class="card variant-glass-surface p-4">
					<div class="flex items-start justify-between gap-3">
						<div>
							<h3 class="text-base font-semibold">Last 5 sessions</h3>
							<p class="text-sm opacity-70">Set patterns for {selectedExercise}.</p>
						</div>
					</div>

					{#if recentExerciseSessions.length}
						<div class="mt-3 space-y-3">
							{#each recentExerciseSessions as session (session.exercise.id)}
								<div
									class="rounded-xl border border-surface-200/50 bg-surface-50/60 p-3 dark:border-surface-700/50 dark:bg-surface-950/30"
								>
									<div class="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1">
										<div class="font-semibold">
											{formatDateRelative(session.exercise.start_time)}
										</div>
										<div class="text-xs opacity-70">
											{formatTime(session.exercise.start_time)}
										</div>
									</div>

									{#if session.exercise.notes}
										<div class="mt-1 text-sm opacity-80">{session.exercise.notes}</div>
									{/if}

									<div class="mt-2">
										<SetPillsHybrid
											sets={session.sets.map((s) => ({
												reps: s.reps,
												weight: s.weight,
												weightLeft: s.weight_left,
												weightRight: s.weight_right
											}))}
											perSideWeight={session.exercise.per_side_weight ?? false}
											splitWeight={session.exercise.split_weight ?? false}
											size="xs"
										/>
									</div>
								</div>
							{/each}
						</div>
					{:else}
						<div class="mt-3 text-sm opacity-70">No sessions yet for this exercise.</div>
					{/if}
				</div>
			{/if}
		</section>

		<aside class="md:col-span-5 lg:col-span-4 space-y-4 min-w-0">
			<div class="card variant-glass-surface p-4 min-w-0">
				<div class="flex items-start justify-between gap-3">
					<div>
						<h3 class="text-base font-semibold">Session feel</h3>
						<p class="text-sm opacity-70">How sessions have been rated.</p>
					</div>
				</div>
				<div class="mt-3 h-64">
					<canvas bind:this={feedbackCanvas}></canvas>
				</div>
			</div>

			<div class="card variant-glass-surface p-4 min-w-0">
				<div class="flex items-start justify-between gap-3">
					<div>
						<h3 class="text-base font-semibold">Sessions per month</h3>
						<p class="text-sm opacity-70">Rolling last 12 months.</p>
					</div>
				</div>
				<div class="mt-3 h-64">
					<canvas bind:this={monthlySessionsCanvas}></canvas>
				</div>
			</div>

			<div class="card variant-glass-surface p-4 min-w-0">
				<div class="flex items-start justify-between gap-3">
					<div>
						<h3 class="text-base font-semibold">Time of day</h3>
						<p class="text-sm opacity-70">When you most often train.</p>
					</div>
				</div>
				<div class="mt-3 h-64">
					<canvas bind:this={timeCanvas}></canvas>
				</div>
			</div>

			<div class="card variant-glass-surface p-4 min-w-0">
				<div class="flex items-start justify-between gap-3">
					<div>
						<h3 class="text-base font-semibold">Duration distribution</h3>
						<p class="text-sm opacity-70">Your typical session length.</p>
					</div>
				</div>
				<div class="mt-3 h-64">
					<canvas bind:this={durationCanvas}></canvas>
				</div>
			</div>
		</aside>
	</div>
</div>
