<script lang="ts">
	import type { ProgressOverview } from '$lib/types';
	import { formatDuration, formatSignedDuration, formatSignedNumber } from '$lib/progress/format';

	export let overview: ProgressOverview | null = null;
	export let loading = false;
	export let error: string | null = null;

	const metricClass = 'rounded-lg border border-surface-200/50 p-3 dark:border-surface-700/50';

	function volume(value: number): string {
		return `${Math.round(value)}kg`;
	}
</script>

<section class="card variant-glass-surface p-4 space-y-3">
	<div class="flex items-start justify-between gap-3">
		<div>
			<h2 class="text-lg font-semibold tracking-tight">Current progress</h2>
			<p class="text-sm opacity-70">Recent work compared with the previous matching period.</p>
		</div>
	</div>

	{#if error}
		<div class="alert variant-filled-error">{error}</div>
	{:else if loading}
		<div class="grid gap-3 md:grid-cols-2">
			<div class="h-40 rounded-xl bg-surface-200/60 dark:bg-surface-700/50 animate-pulse"></div>
			<div class="h-40 rounded-xl bg-surface-200/60 dark:bg-surface-700/50 animate-pulse"></div>
		</div>
	{:else if overview}
		<div class="grid gap-3 md:grid-cols-2">
			{#each [overview.last_7_days, overview.last_30_days] as period}
				<article class="rounded-xl border border-surface-200/60 p-3 dark:border-surface-700/60">
					<div class="flex items-baseline justify-between gap-3">
						<h3 class="font-semibold">{period.label}</h3>
						<div class="text-xs font-semibold opacity-60">
							{formatSignedNumber(period.comparison.recent_best_count_delta)} recent bests
						</div>
					</div>

					<div class="mt-3 grid grid-cols-2 gap-2">
						<div class={metricClass}>
							<div class="text-xs font-semibold opacity-70">Workouts</div>
							<div class="text-lg font-bold">{period.workouts}</div>
							<div class="text-xs opacity-70">
								{formatSignedNumber(period.comparison.workouts_delta)}
							</div>
						</div>
						<div class={metricClass}>
							<div class="text-xs font-semibold opacity-70">Training time</div>
							<div class="text-lg font-bold">{period.total_training_minutes}m</div>
							<div class="text-xs opacity-70">
								{formatSignedNumber(period.comparison.total_training_minutes_delta, 'm')}
							</div>
						</div>
						<div class={metricClass}>
							<div class="text-xs font-semibold opacity-70">Volume</div>
							<div class="text-lg font-bold">{volume(period.total_volume)}</div>
							<div class="text-xs opacity-70">
								{formatSignedNumber(period.comparison.total_volume_delta, 'kg')}
							</div>
						</div>
						<div class={metricClass}>
							<div class="text-xs font-semibold opacity-70">Records</div>
							<div class="text-lg font-bold">{period.pr_count} PRs</div>
							<div class="text-xs opacity-70">
								{period.recent_best_count} recent bests
							</div>
						</div>
					</div>

					<div class="mt-3 flex flex-wrap gap-2 text-xs">
						<span class="badge variant-soft">{period.sets} sets · {period.reps} reps</span>
						{#if period.timed_sets > 0}
							<span class="badge variant-soft">
								Timed work: {period.timed_sets} sets · {formatDuration(
									period.total_timed_duration_seconds
								)}
								<span class="opacity-70">
									{formatSignedDuration(period.comparison.total_timed_duration_seconds_delta)}
								</span>
							</span>
						{/if}
						{#if period.comparison.pr_count_delta !== 0}
							<span class="badge variant-soft-primary">
								{formatSignedNumber(period.comparison.pr_count_delta)} PRs
							</span>
						{/if}
					</div>

					{#if period.workouts === 0}
						<div class="mt-3 text-sm opacity-70">No workouts in this period yet.</div>
					{/if}
				</article>
			{/each}
		</div>
	{:else}
		<div class="card variant-ghost p-4 text-center opacity-80">No progress summary yet.</div>
	{/if}
</section>
