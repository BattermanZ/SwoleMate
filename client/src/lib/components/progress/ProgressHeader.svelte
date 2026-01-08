<script lang="ts">
	import type { WorkoutStats } from '$lib/types';
	import { createEventDispatcher } from 'svelte';

	export let workoutStats: WorkoutStats | null = null;
	export let selectedExercise = '';
	export let loadingOverall = false;
	export let loadingExercise = false;
	export let errorOverall: string | null = null;
	export let errorExercise: string | null = null;

	const dispatch = createEventDispatcher<{ refresh: void }>();
</script>

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
				on:click={() => dispatch('refresh')}
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
