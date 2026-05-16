<script lang="ts">
	import { Card, MetricTile, RPill } from '$lib/components/ui';
	import { summarizeRepPrs } from '$lib/progress/repPrs';
	import { formatDuration } from '$lib/progress/format';
	import type { VolumeStats } from '$lib/types';

	interface Props {
		selectedExercise: string;
		exerciseTypes: string[];
		volumeStats: VolumeStats | null;
		loadingExercise?: boolean;
		errorExercise?: string | null;
		onSelect?: (exercise: string) => void;
	}
	let {
		selectedExercise = $bindable(''),
		exerciseTypes,
		volumeStats,
		loadingExercise = false,
		errorExercise = null,
		onSelect
	}: Props = $props();

	let repSummary = $derived(summarizeRepPrs(volumeStats?.personal_records.rep_prs ?? []));
	let showAllReps = $state(false);
	let displayedReps = $derived(showAllReps ? repSummary : repSummary.slice(0, 10));

	function onChange(e: Event) {
		const next = (e.currentTarget as HTMLSelectElement).value;
		selectedExercise = next;
		onSelect?.(next);
	}
</script>

<Card>
	{#snippet title()}Exercise focus{/snippet}
	{#snippet lede()}Pick an exercise to see volume, strength and PRs.{/snippet}
	{#snippet actions()}
		<div class="picker">
			<select
				bind:value={selectedExercise}
				disabled={loadingExercise || exerciseTypes.length === 0}
				onchange={onChange}
				aria-label="Exercise"
			>
				{#if exerciseTypes.length === 0}
					<option value="">No exercises yet</option>
				{:else}
					{#each exerciseTypes as type (type)}
						<option value={type}>{type}</option>
					{/each}
				{/if}
			</select>
			<span class="caret" aria-hidden="true">▾</span>
		</div>
	{/snippet}

	{#if errorExercise}
		<div class="err">{errorExercise}</div>
	{:else if volumeStats}
		<div class="trio">
			<MetricTile
				label="All-time max"
				value={`${volumeStats.personal_records.all_time_max_weight}`}
				unit="kg"
				rail="clay"
			/>
			<MetricTile
				label="Estimated 1RM"
				value={`${volumeStats.personal_records.estimated_max_1rm}`}
				unit="kg"
				rail="gold"
			/>
			<MetricTile
				label="Max session volume"
				value={`${volumeStats.personal_records.max_volume}`}
				unit="kg"
				rail="sage"
			/>
		</div>

		{#if repSummary.length > 0}
			<div class="rep-prs">
				<div class="rep-head">
					<h4>Rep PRs</h4>
					{#if repSummary.length > 10}
						<button type="button" class="show-all" onclick={() => (showAllReps = !showAllReps)}>
							{showAllReps ? 'Show less' : `Show all (${repSummary.length})`}
						</button>
					{/if}
				</div>
				<div class="pills">
					{#each displayedReps as pr (pr.reps)}
						<RPill reps={pr.reps} weight={`${pr.weight} kg`} />
					{/each}
				</div>
			</div>
		{/if}

		{#if volumeStats.timed_records}
			{@const t = volumeStats.timed_records}
			<div class="timed">
				<MetricTile label="Longest set" value={formatDuration(t.longest_set_seconds)} rail="sage" />
				<MetricTile
					label="Best session"
					value={formatDuration(t.best_session_duration_seconds)}
					rail="warn"
				/>
				<MetricTile
					label="Lifetime timed"
					value={formatDuration(t.lifetime_duration_seconds)}
					rail="clay"
				/>
				<MetricTile
					label="Avg timed set"
					value={formatDuration(t.average_set_duration_seconds)}
					rail="gold"
				/>
			</div>
		{/if}
	{:else if loadingExercise}
		<div class="skel"></div>
	{:else if selectedExercise}
		<div class="empty">
			No progress data yet for <b>{selectedExercise}</b>.
		</div>
	{/if}
</Card>

<style>
	.picker {
		position: relative;
		display: inline-flex;
		align-items: center;
	}
	.picker select {
		appearance: none;
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 12px;
		padding: 10px 36px 10px 14px;
		font:
			700 13px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink);
		cursor: pointer;
		max-width: 220px;
		text-overflow: ellipsis;
	}
	.picker select:focus {
		outline: 0;
		border-color: var(--clay);
		box-shadow: 0 0 0 3px rgba(255, 94, 31, 0.16);
	}
	.caret {
		position: absolute;
		right: 12px;
		font:
			800 14px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
		pointer-events: none;
	}

	.trio {
		display: grid;
		grid-template-columns: 1fr;
		gap: 8px;
	}
	@media (min-width: 640px) {
		.trio {
			grid-template-columns: repeat(3, 1fr);
		}
	}

	.rep-prs {
		margin-top: 12px;
		padding: 12px;
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 14px;
	}
	.rep-head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 10px;
	}
	.rep-head h4 {
		margin: 0;
		font:
			700 11px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--ink-soft);
	}
	.show-all {
		background: transparent;
		border: 0;
		padding: 4px 6px;
		font:
			700 11px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--clay-text);
		cursor: pointer;
	}
	.pills {
		margin-top: 10px;
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}

	.timed {
		margin-top: 12px;
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 8px;
	}

	.err {
		font:
			600 13px/1.4 'Onest',
			system-ui,
			sans-serif;
		color: var(--clay-text);
	}
	.skel {
		height: 60px;
		border-radius: 12px;
		background: var(--card-3);
	}
	.empty {
		font:
			500 13px/1.4 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}
</style>
