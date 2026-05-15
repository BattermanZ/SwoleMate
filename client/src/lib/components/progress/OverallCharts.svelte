<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import 'chartjs-adapter-date-fns';
	import type { ChartConfiguration } from 'chart.js';
	import type { WorkoutStats } from '$lib/types';
	import {
		baseOptions,
		observeTheme,
		rgba,
		readTheme,
		formatMonthLabel,
		upsertChart,
		type AnyChart,
		type ChartTheme
	} from '$lib/progress/charting';
	import ChartCard from './ChartCard.svelte';

	interface Props {
		workoutStats: WorkoutStats | null;
	}
	let { workoutStats }: Props = $props();

	let feedbackCanvas: HTMLCanvasElement | null = $state(null);
	let monthlyCanvas: HTMLCanvasElement | null = $state(null);
	let exerciseDurCanvas: HTMLCanvasElement | null = $state(null);
	let timeCanvas: HTMLCanvasElement | null = $state(null);
	let durationCanvas: HTMLCanvasElement | null = $state(null);

	let feedbackChart: AnyChart | null = null;
	let monthlyChart: AnyChart | null = null;
	let exerciseDurChart: AnyChart | null = null;
	let timeChart: AnyChart | null = null;
	let durationChart: AnyChart | null = null;

	let theme: ChartTheme = readTheme();
	let observer: MutationObserver | null = null;

	function destroyAll() {
		feedbackChart?.destroy();
		monthlyChart?.destroy();
		exerciseDurChart?.destroy();
		timeChart?.destroy();
		durationChart?.destroy();
		feedbackChart = monthlyChart = exerciseDurChart = timeChart = durationChart = null;
	}

	function render() {
		if (!workoutStats) {
			destroyAll();
			return;
		}
		const base = baseOptions(theme);

		// 1. Session feel (doughnut)
		if (feedbackCanvas) {
			const d = workoutStats.feedback_distribution;
			feedbackChart = upsertChart(feedbackChart, feedbackCanvas, {
				type: 'doughnut',
				data: {
					labels: ['Good', 'Neutral', 'Bad'],
					datasets: [
						{
							data: [d.good, d.neutral, d.bad],
							backgroundColor: [theme.success, theme.tertiary, theme.primary],
							borderColor: 'transparent',
							borderWidth: 0
						}
					]
				},
				options: {
					...base,
					cutout: '64%',
					plugins: {
						...(base.plugins ?? {}),
						legend: {
							position: 'right',
							labels: { color: theme.text, font: { family: 'Onest', size: 12, weight: 600 } }
						}
					},
					scales: {}
				} as unknown as ChartConfiguration['options']
			});
		}

		// 2. Sessions per month
		if (workoutStats.sessions_per_month && monthlyCanvas) {
			const rows = workoutStats.sessions_per_month;
			monthlyChart = upsertChart(monthlyChart, monthlyCanvas, {
				type: 'bar',
				data: {
					labels: rows.map((r) => formatMonthLabel(r.month)),
					datasets: [
						{
							label: 'Sessions',
							data: rows.map((r) => r.count),
							backgroundColor: rgba(theme.primary, 0.85),
							borderColor: theme.primary,
							borderRadius: 4
						}
					]
				},
				options: base
			});
		}

		// 3. Avg time per exercise (horizontal bar) — using the series if present
		if (workoutStats.avg_exercise_duration_series && exerciseDurCanvas) {
			const rows = workoutStats.avg_exercise_duration_series.slice(-12);
			exerciseDurChart = upsertChart(exerciseDurChart, exerciseDurCanvas, {
				type: 'line',
				data: {
					datasets: [
						{
							label: 'Avg minutes / exercise',
							data: rows.map((r) => ({
								x: new Date(r.start_time).getTime(),
								y: r.avg_minutes
							})),
							borderColor: theme.tertiary,
							backgroundColor: rgba(theme.tertiary, 0.2),
							fill: true,
							borderWidth: 2.4,
							tension: 0.35,
							pointRadius: 3
						}
					]
				},
				options: {
					...base,
					scales: {
						x: {
							type: 'time',
							time: { unit: 'month' },
							ticks: { color: theme.mutedText },
							grid: { color: theme.grid }
						},
						y: {
							beginAtZero: true,
							ticks: { color: theme.mutedText },
							grid: { color: theme.grid }
						}
					}
				}
			});
		}

		// 4. Time of day
		if (timeCanvas) {
			timeChart = upsertChart(timeChart, timeCanvas, {
				type: 'bar',
				data: {
					labels: workoutStats.popular_hours.map((p) => p.hour),
					datasets: [
						{
							label: 'Workouts',
							data: workoutStats.popular_hours.map((p) => p.count),
							backgroundColor: rgba(theme.primary, 0.85),
							borderColor: theme.primary,
							borderRadius: 4
						}
					]
				},
				options: base
			});
		}

		// 5. Duration distribution
		if (durationCanvas) {
			durationChart = upsertChart(durationChart, durationCanvas, {
				type: 'bar',
				data: {
					labels: workoutStats.duration_distribution.map((d) => d.range),
					datasets: [
						{
							label: 'Sessions',
							data: workoutStats.duration_distribution.map((d) => d.count),
							backgroundColor: rgba(theme.success, 0.85),
							borderColor: theme.success,
							borderRadius: 4
						}
					]
				},
				options: base
			});
		}
	}

	function refreshTheme() {
		theme = readTheme();
		render();
	}

	onMount(() => {
		observer = observeTheme(refreshTheme);
		render();
	});

	onDestroy(() => {
		observer?.disconnect();
		destroyAll();
	});

	$effect(() => {
		workoutStats;
		render();
	});
</script>

{#if workoutStats}
	<ChartCard headline="Session feel" titleEm="last 30 days" height={180}>
		<canvas bind:this={feedbackCanvas}></canvas>
	</ChartCard>

	{#if workoutStats.sessions_per_month && workoutStats.sessions_per_month.length > 0}
		<ChartCard headline="Sessions per month">
			<canvas bind:this={monthlyCanvas}></canvas>
		</ChartCard>
	{/if}

	{#if workoutStats.avg_exercise_duration_series && workoutStats.avg_exercise_duration_series.length > 0}
		<ChartCard headline="Avg time per exercise" titleEm="rolling">
			<canvas bind:this={exerciseDurCanvas}></canvas>
		</ChartCard>
	{/if}

	<ChartCard headline="Time of day" titleEm="when you train">
		<canvas bind:this={timeCanvas}></canvas>
	</ChartCard>

	<ChartCard headline="Duration distribution">
		<canvas bind:this={durationCanvas}></canvas>
	</ChartCard>
{/if}

<style>
</style>
