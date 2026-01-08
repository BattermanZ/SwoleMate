<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import 'chartjs-adapter-date-fns';
	import type { ExerciseProgress, VolumeStats } from '$lib/types';
	import {
		baseOptions,
		observeTheme,
		rgba,
		readTheme,
		upsertChart,
		type AnyChart,
		type ChartTheme
	} from '$lib/progress/charting';

	export let volumeStats: VolumeStats | null = null;
	export let exerciseProgress: ExerciseProgress[] | null = null;

	let volumeCanvas: HTMLCanvasElement | null = null;
	let progressCanvas: HTMLCanvasElement | null = null;
	let monthlyVolumeCanvas: HTMLCanvasElement | null = null;

	let volumeChart: AnyChart | null = null;
	let progressChart: AnyChart | null = null;
	let monthlyVolumeChart: AnyChart | null = null;

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

	let observer: MutationObserver | null = null;

	function destroyCharts() {
		volumeChart?.destroy();
		progressChart?.destroy();
		monthlyVolumeChart?.destroy();
		volumeChart = null;
		progressChart = null;
		monthlyVolumeChart = null;
	}

	function render(...deps: unknown[]) {
		if (deps.length < 0) return;
		if (!volumeStats) {
			destroyCharts();
			return;
		}

		const base = baseOptions(theme);
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
					...baseOptions(theme),
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

	onMount(() => {
		theme = readTheme();
		observer = observeTheme(() => {
			theme = readTheme();
		});
	});

	$: render(
		volumeStats,
		exerciseProgress,
		volumeCanvas,
		progressCanvas,
		monthlyVolumeCanvas,
		theme
	);

	onDestroy(() => {
		destroyCharts();
		observer?.disconnect();
		observer = null;
	});
</script>

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
