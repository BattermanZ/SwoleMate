<script lang="ts">
	import { MetricTile } from '$lib/components/ui';
	import { formatDuration, formatSignedNumber, formatSignedDuration } from '$lib/progress/format';
	import type { ProgressOverview } from '$lib/types';

	type Period = ProgressOverview['last_7_days'];

	interface Props {
		period: Period;
		variant?: 'primary' | 'secondary';
	}
	let { period, variant = 'secondary' }: Props = $props();
</script>

<article class="card v-{variant}">
	<div class="rail"></div>
	<header>
		<h3>{period.label}</h3>
		<span class="chip"
			>{formatSignedNumber(period.comparison.recent_best_count_delta)} recent bests</span
		>
	</header>

	<div class="metrics">
		<MetricTile
			label="Workouts"
			value={String(period.workouts)}
			delta={formatSignedNumber(period.comparison.workouts_delta)}
			deltaTone={period.comparison.workouts_delta > 0
				? 'up'
				: period.comparison.workouts_delta < 0
					? 'down'
					: 'neutral'}
			rail="clay"
		/>
		<MetricTile
			label="Time"
			value={`${period.total_training_minutes}`}
			unit="m"
			delta={formatSignedNumber(period.comparison.total_training_minutes_delta, 'm')}
			deltaTone={period.comparison.total_training_minutes_delta > 0
				? 'up'
				: period.comparison.total_training_minutes_delta < 0
					? 'down'
					: 'neutral'}
			rail="warn"
		/>
		<MetricTile
			label="Volume"
			value={`${Math.round(period.total_volume).toLocaleString()}`}
			unit="kg"
			delta={formatSignedNumber(period.comparison.total_volume_delta, 'kg')}
			deltaTone={period.comparison.total_volume_delta > 0
				? 'up'
				: period.comparison.total_volume_delta < 0
					? 'down'
					: 'neutral'}
			rail="sage"
		/>
		<MetricTile
			label="Records"
			value={`${period.pr_count}`}
			unit="PRs"
			delta={`${period.recent_best_count} recent bests`}
			rail="gold"
		/>
	</div>

	<div class="badges">
		<span class="badge">{period.sets} sets · {period.reps} reps</span>
		{#if period.timed_sets > 0}
			<span class="badge"
				>Timed {period.timed_sets} sets · {formatDuration(period.total_timed_duration_seconds)}
				<span class="dim"
					>{formatSignedDuration(period.comparison.total_timed_duration_seconds_delta)}</span
				></span
			>
		{/if}
		{#if period.comparison.pr_count_delta !== 0}
			<span class="badge primary">{formatSignedNumber(period.comparison.pr_count_delta)} PRs</span>
		{/if}
	</div>

	{#if period.workouts === 0}
		<div class="empty">No workouts in this period yet.</div>
	{/if}
</article>

<style>
	.card {
		position: relative;
		overflow: hidden;
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 18px;
		padding: 14px;
		min-width: 0;
	}
	.card + :global(.card) {
		margin-top: 10px;
	}
	.card.v-primary {
		background: color-mix(in oklab, var(--clay) 5%, var(--card-3));
		border-color: color-mix(in oklab, var(--clay) 30%, var(--line));
	}
	.rail {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		height: 3px;
	}
	.v-primary .rail {
		background: linear-gradient(90deg, var(--clay-2), var(--gold) 60%, var(--sage));
	}
	.v-secondary .rail {
		background: var(--line-2);
	}
	header {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 8px;
		margin-bottom: 12px;
	}
	h3 {
		margin: 0;
		font:
			800 15px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: -0.01em;
	}
	.chip {
		border-radius: 999px;
		padding: 5px 10px;
		border: 1px solid var(--line);
		background: var(--card);
		font:
			700 10px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.06em;
		color: var(--ink-2);
		white-space: nowrap;
	}

	.metrics {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 6px;
	}

	.badges {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		margin-top: 12px;
	}
	.badge {
		font:
			700 10px/1 'Onest',
			system-ui,
			sans-serif;
		padding: 6px 10px;
		border-radius: 999px;
		background: var(--card);
		color: var(--ink-2);
		border: 1px solid var(--line);
	}
	.badge.primary {
		background: color-mix(in oklab, var(--clay) 14%, var(--card));
		color: var(--clay-text);
		border-color: color-mix(in oklab, var(--clay) 30%, var(--line));
	}
	.badge .dim {
		color: var(--ink-soft);
		margin-left: 4px;
		font-weight: 500;
	}

	.empty {
		margin-top: 12px;
		font:
			500 13px/1.4 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}
</style>
