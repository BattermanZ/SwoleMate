<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import 'chartjs-adapter-date-fns';
	import type { ChartConfiguration } from 'chart.js';
	import type { WorkoutStats } from '$lib/types';
	import {
		baseOptions,
		formatMonthLabel,
		observeTheme,
		rgba,
		readTheme,
		upsertChart,
		type AnyChart,
		type ChartTheme
	} from '$lib/progress/charting';

	export let workoutStats: WorkoutStats | null = null;

	let feedbackCanvas: HTMLCanvasElement | null = null;
	let timeCanvas: HTMLCanvasElement | null = null;
	let durationCanvas: HTMLCanvasElement | null = null;
	let monthlySessionsCanvas: HTMLCanvasElement | null = null;
	let avgExerciseDurationCanvas: HTMLCanvasElement | null = null;

	let feedbackChart: AnyChart | null = null;
	let timeDistributionChart: AnyChart | null = null;
	let durationDistributionChart: AnyChart | null = null;
	let monthlySessionsChart: AnyChart | null = null;
	let avgExerciseDurationChart: AnyChart | null = null;

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
		feedbackChart?.destroy();
		timeDistributionChart?.destroy();
		durationDistributionChart?.destroy();
		monthlySessionsChart?.destroy();
		avgExerciseDurationChart?.destroy();
		feedbackChart = null;
		timeDistributionChart = null;
		durationDistributionChart = null;
		monthlySessionsChart = null;
		avgExerciseDurationChart = null;
	}

	function render(...deps: unknown[]) {
		if (deps.length < 0) return;
		if (!workoutStats) {
			destroyCharts();
			return;
		}

		const base = baseOptions(theme);
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

		const avgExerciseSeries = workoutStats.avg_exercise_duration_series ?? [];
		if (avgExerciseSeries.length) {
			avgExerciseDurationChart = upsertChart(avgExerciseDurationChart, avgExerciseDurationCanvas, {
				type: 'line',
				data: {
					datasets: [
						{
							label: 'Avg minutes / exercise',
							data: avgExerciseSeries.map((p) => ({
								x: new Date(p.start_time).getTime(),
								y: p.avg_minutes
							})),
							borderColor: theme.secondary,
							backgroundColor: rgba(theme.secondary, theme.isDark ? 0.22 : 0.14),
							pointRadius: 2,
							tension: 0.25,
							fill: false
						}
					]
				},
				options: {
					...base,
					scales: {
						x: {
							type: 'time',
							time: { unit: 'week' },
							ticks: { color: theme.mutedText },
							grid: { color: theme.grid },
							title: { display: true, text: 'Session', color: theme.mutedText }
						},
						y: {
							...(baseScales.y ?? {}),
							beginAtZero: true,
							title: { display: true, text: 'Min / exercise', color: theme.mutedText }
						}
					}
				}
			} as unknown as ChartConfiguration<'line'>);
		} else {
			avgExerciseDurationChart?.destroy();
			avgExerciseDurationChart = null;
		}

		const sessionStartSamples = workoutStats.session_start_samples ?? [];
		const sessionStartTimes = workoutStats.session_start_times ?? [];
		const hasSamples = sessionStartSamples.length > 0;

		if (hasSamples || sessionStartTimes.length) {
			const labels = Array.from({ length: 48 }, (_, i) => {
				const hour = String(Math.floor(i / 2)).padStart(2, '0');
				const minute = i % 2 === 0 ? '00' : '30';
				return `${hour}:${minute}`;
			});
			const bins = Array.from({ length: 48 }, () => 0);

			if (hasSamples) {
				for (const sample of sessionStartSamples) {
					const msUtc = new Date(sample.start_time).getTime();
					if (!Number.isFinite(msUtc)) continue;

					const offsetMinutes = sample.timezone_offset_minutes;
					const hasOffset = typeof offsetMinutes === 'number' && Number.isFinite(offsetMinutes);
					const msForWorkoutLocal = hasOffset ? msUtc - offsetMinutes * 60_000 : msUtc;

					const date = new Date(msForWorkoutLocal);
					const hour = hasOffset ? date.getUTCHours() : date.getHours();
					const minute = hasOffset ? date.getUTCMinutes() : date.getMinutes();

					const idx = hour * 2 + (minute >= 30 ? 1 : 0);
					if (idx < 0 || idx >= bins.length) continue;
					bins[idx] += 1;
				}
			} else {
				for (const iso of sessionStartTimes) {
					const ms = new Date(iso).getTime();
					if (!Number.isFinite(ms)) continue;
					const date = new Date(ms);
					const hour = date.getHours();
					const minute = date.getMinutes();
					const idx = hour * 2 + (minute >= 30 ? 1 : 0);
					if (idx < 0 || idx >= bins.length) continue;
					bins[idx] += 1;
				}
			}

			const nonEmpty = bins
				.map((count, index) => ({ label: labels[index], count }))
				.filter((point) => point.count > 0);

			if (nonEmpty.length) {
				timeDistributionChart = upsertChart(timeDistributionChart, timeCanvas, {
					type: 'bar',
					data: {
						labels: nonEmpty.map((p) => p.label),
						datasets: [
							{
								label: 'Workouts',
								data: nonEmpty.map((p) => p.count),
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
								title: { display: true, text: 'Workout local time', color: theme.mutedText }
							},
							y: {
								...(baseScales.y ?? {}),
								beginAtZero: true,
								title: { display: true, text: 'Workouts', color: theme.mutedText }
							}
						}
					}
				});
			} else {
				timeDistributionChart?.destroy();
				timeDistributionChart = null;
			}
		} else {
			timeDistributionChart?.destroy();
			timeDistributionChart = null;
		}

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

	onMount(() => {
		theme = readTheme();
		observer = observeTheme(() => {
			theme = readTheme();
		});
	});

	$: render(
		workoutStats,
		feedbackCanvas,
		timeCanvas,
		durationCanvas,
		monthlySessionsCanvas,
		avgExerciseDurationCanvas,
		theme
	);

	onDestroy(() => {
		destroyCharts();
		observer?.disconnect();
		observer = null;
	});
</script>

<div class="space-y-4 min-w-0">
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
				<h3 class="text-base font-semibold">Avg time per exercise</h3>
				<p class="text-sm opacity-70">Session pace over time.</p>
			</div>
		</div>
		<div class="mt-3 h-64">
			<canvas bind:this={avgExerciseDurationCanvas}></canvas>
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
</div>
