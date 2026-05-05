<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import type { VolumeStats } from '$lib/types';
	import RepPrs from '$lib/components/progress/RepPrs.svelte';
	import TimedRecords from '$lib/components/progress/TimedRecords.svelte';

	export let selectedExercise = '';
	export let exerciseTypes: string[] = [];
	export let volumeStats: VolumeStats | null = null;

	export let loadingExercise = false;
	export let errorExercise: string | null = null;

	const dispatch = createEventDispatcher<{ select: string }>();
</script>

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
				on:change={() => dispatch('select', selectedExercise)}
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

	{#if errorExercise}
		<div class="alert variant-filled-error">{errorExercise}</div>
	{:else if volumeStats}
		<div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
			<div class="card variant-glass-surface p-3 border-l-4 border-primary-500/60">
				<div class="text-xs font-semibold opacity-70">All‑time max</div>
				<div class="text-lg font-bold">{volumeStats.personal_records.all_time_max_weight}kg</div>
			</div>
			<div class="card variant-glass-surface p-3 border-l-4 border-tertiary-500/60">
				<div class="text-xs font-semibold opacity-70">Estimated 1RM</div>
				<div class="text-lg font-bold">{volumeStats.personal_records.estimated_max_1rm}kg</div>
			</div>
			<div class="card variant-glass-surface p-3 border-l-4 border-secondary-500/60">
				<div class="text-xs font-semibold opacity-70">Max session volume</div>
				<div class="text-lg font-bold">{volumeStats.personal_records.max_volume}kg</div>
			</div>
		</div>

		{#key selectedExercise}
			<RepPrs repPrs={volumeStats.personal_records.rep_prs ?? []} />
		{/key}

		{#if volumeStats.timed_records}
			<TimedRecords records={volumeStats.timed_records} />
		{/if}
	{:else if loadingExercise}
		<div class="card variant-glass-surface p-4 animate-pulse">
			<div class="h-4 w-36 bg-surface-200/60 dark:bg-surface-700/50 rounded"></div>
			<div class="mt-3 h-8 w-48 bg-surface-200/60 dark:bg-surface-700/50 rounded"></div>
		</div>
	{:else if selectedExercise}
		<div class="card variant-ghost p-4 text-center opacity-80">
			No progress data yet for <span class="font-semibold">{selectedExercise}</span>.
		</div>
	{/if}
</div>
