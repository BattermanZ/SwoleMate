<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import 'chartjs-adapter-date-fns';
	import type { ChartConfiguration } from 'chart.js';
	import type { ExerciseProgress, VolumeStats } from '$lib/types';
	import {
		baseOptions,
		observeTheme,
		rgba,
		readTheme,
		sqliteWeekKeyToTimestamp,
		upsertChart,
		formatMonthLabel,
		type AnyChart,
		type ChartTheme
	} from '$lib/progress/charting';
	import { getEffectiveWeight } from '$lib/progress/weights';
	import ChartCard from './ChartCard.svelte';

	interface Props {
		volumeStats: VolumeStats | null;
		exerciseProgress: ExerciseProgress[] | null;
	}
	let { volumeStats, exerciseProgress }: Props = $props();

	let weeklyCanvas: HTMLCanvasElement | null = $state(null);
	let progressCanvas: HTMLCanvasElement | null = $state(null);
	let monthlyCanvas: HTMLCanvasElement | null = $state(null);

	let weeklyChart: AnyChart | null = null;
	let progressChart: AnyChart | null = null;
	let monthlyChart: AnyChart | null = null;

	let theme: ChartTheme = readTheme();
	let observer: MutationObserver | null = null;

	function destroyAll() {
		weeklyChart?.destroy();
		progressChart?.destroy();
		monthlyChart?.destroy();
		weeklyChart = progressChart = monthlyChart = null;
	}

	function render() {
		if (!volumeStats) {
			destroyAll();
			return;
		}
		const base = baseOptions(theme);

		// 1. Weekly volume + 1RM
		const weeklyRows = volumeStats.weekly_volume.filter(
			(v) => v.total_volume > 0 || v.max_estimated_1rm > 0
		);
		if (weeklyRows.length > 0 && weeklyCanvas) {
			const volumeData = weeklyRows.map((v) => ({
				x: sqliteWeekKeyToTimestamp(v.week) ?? 0,
				y: v.total_volume
			}));
			const oneRmData = weeklyRows.map((v) => ({
				x: sqliteWeekKeyToTimestamp(v.week) ?? 0,
				y: v.max_estimated_1rm
			}));
			const cfg: ChartConfiguration = {
				type: 'bar',
				data: {
					datasets: [
						{
							type: 'bar',
							label: 'Volume (kg)',
							data: volumeData,
							backgroundColor: rgba(theme.primary, 0.85),
							borderColor: theme.primary,
							borderRadius: 4,
							yAxisID: 'y'
						},
						{
							type: 'line',
							label: '1RM (kg)',
							data: oneRmData,
							borderColor: theme.secondary,
							backgroundColor: theme.secondary,
							borderWidth: 2.4,
							tension: 0.35,
							pointRadius: 3,
							pointBackgroundColor: theme.secondary,
							yAxisID: 'y1'
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
							grid: { color: theme.grid }
						},
						y: {
							beginAtZero: true,
							ticks: { color: theme.mutedText },
							grid: { color: theme.grid }
						},
						y1: {
							beginAtZero: true,
							position: 'right',
							ticks: { color: theme.mutedText },
							grid: { display: false }
						}
					}
				}
			};
			weeklyChart = upsertChart(weeklyChart, weeklyCanvas, cfg);
		}

		// 2. Best weight per session
		const sessions = (exerciseProgress ?? []).filter((ep) =>
			ep.sets.some((s) => getEffectiveWeight(s, ep.exercise) > 0)
		);
		if (sessions.length > 0 && progressCanvas) {
			const data = sessions
				.map((ep) => {
					const t = new Date(ep.exercise.start_time).getTime();
					const top = Math.max(...ep.sets.map((s) => getEffectiveWeight(s, ep.exercise)));
					return { x: t, y: top };
				})
				.sort((a, b) => a.x - b.x);
			const cfg: ChartConfiguration = {
				type: 'line',
				data: {
					datasets: [
						{
							label: 'Top set (kg)',
							data,
							borderColor: theme.primary,
							backgroundColor: rgba(theme.primary, 0.18),
							fill: true,
							borderWidth: 2.4,
							pointRadius: 3,
							pointBackgroundColor: theme.primary,
							tension: 0.3
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
							beginAtZero: false,
							ticks: { color: theme.mutedText },
							grid: { color: theme.grid }
						}
					}
				}
			};
			progressChart = upsertChart(progressChart, progressCanvas, cfg);
		}

		// 3. Monthly volume — render every month in the series, even zero ones,
		// so gaps in training are visible instead of being hidden.
		const monthlyRows = volumeStats.monthly_volume;
		if (monthlyRows.length > 0 && monthlyCanvas) {
			const cfg: ChartConfiguration = {
				type: 'bar',
				data: {
					labels: monthlyRows.map((v) => formatMonthLabel(v.month)),
					datasets: [
						{
							label: 'Volume (kg)',
							data: monthlyRows.map((v) => v.total_volume),
							backgroundColor: rgba(theme.success, 0.85),
							borderColor: theme.success,
							borderRadius: 4
						}
					]
				},
				options: base
			};
			monthlyChart = upsertChart(monthlyChart, monthlyCanvas, cfg);
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
		render();
	});
</script>

{#if volumeStats}
	<ChartCard headline="Weekly volume" titleEm="+ 1RM trace">
		{#snippet legend()}
			<span class="dot" style="--dot: var(--clay)">Volume</span>
			<span class="dot" style="--dot: var(--ink)">1RM</span>
		{/snippet}
		<canvas bind:this={weeklyCanvas}></canvas>
	</ChartCard>

	<ChartCard headline="Best weight" titleEm="per session">
		{#snippet legend()}<span class="dot" style="--dot: var(--clay)">Top set</span>{/snippet}
		<canvas bind:this={progressCanvas}></canvas>
	</ChartCard>

	<ChartCard headline="Monthly volume">
		{#snippet legend()}<span class="dot" style="--dot: var(--sage)">Total kg</span>{/snippet}
		<canvas bind:this={monthlyCanvas}></canvas>
	</ChartCard>
{/if}

<style>
</style>
