<script lang="ts">
	import { Card, SetPillList } from '$lib/components/ui';
	import { formatDateRelative, formatTime } from '$lib/utils/date';
	import type { ExerciseProgress } from '$lib/types';

	interface Props {
		exerciseProgress: ExerciseProgress[] | null;
	}
	let { exerciseProgress }: Props = $props();

	let sessions = $derived((exerciseProgress ?? []).slice(-5).reverse());
</script>

<Card>
	{#snippet title()}Last 5 sessions{/snippet}
	{#snippet lede()}Set patterns for this exercise.{/snippet}

	{#if sessions.length === 0}
		<div class="empty">No sessions yet for this exercise.</div>
	{:else}
		<div class="list">
			{#each sessions as s (s.exercise.id)}
				<div class="row">
					<div class="top">
						<span class="day">{formatDateRelative(s.exercise.start_time)}</span>
						<span class="time">{formatTime(s.exercise.start_time)}</span>
					</div>
					{#if s.exercise.notes}<p class="notes">Notes: {s.exercise.notes}</p>{/if}
					<div class="pills">
						<SetPillList
							sets={s.sets.map((x) => ({
								reps: x.reps,
								weight: x.weight,
								weightLeft: x.weight_left,
								weightRight: x.weight_right,
								durationSeconds: x.duration_seconds
							}))}
							perSideWeight={s.exercise.per_side_weight ?? false}
							splitWeight={s.exercise.split_weight ?? false}
							size="xs"
						/>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</Card>

<style>
	.list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.row {
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 14px;
		padding: 12px;
	}
	.top {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 10px;
	}
	.day {
		font:
			800 13px/1 'Onest',
			system-ui,
			sans-serif;
	}
	.time {
		font:
			600 11px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}
	.notes {
		margin: 6px 0 0;
		font: italic 400 12px/1.4 'Instrument Serif';
		color: var(--ink-2);
	}
	.pills {
		margin-top: 8px;
	}
	.empty {
		font:
			500 13px/1.4 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}
</style>
