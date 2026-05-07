<script lang="ts">
	import type { ProgressOverview } from '$lib/types';
	import { formatDuration, formatSignedDuration, formatSignedNumber } from '$lib/progress/format';

	export let overview: ProgressOverview | null = null;
	export let loading = false;
	export let error: string | null = null;

	const metricBase =
		'relative overflow-hidden rounded-xl border p-3 shadow-[0_10px_24px_-18px_rgba(15,23,42,0.45)] dark:shadow-none';

	const metrics = {
		workouts: `${metricBase} border-primary-500/20 bg-primary-500/10 dark:bg-primary-500/10`,
		time: `${metricBase} border-warning-500/25 bg-warning-500/10 dark:bg-warning-500/10`,
		volume: `${metricBase} border-success-500/20 bg-success-500/10 dark:bg-success-500/10`,
		records: `${metricBase} border-tertiary-500/25 bg-tertiary-500/10 dark:bg-tertiary-500/10`
	};

	const periodCardClass = (index: number) =>
		index === 0
			? 'relative overflow-hidden rounded-2xl border border-primary-500/25 bg-gradient-to-br from-primary-500/10 via-surface-50/85 to-warning-500/10 p-3 shadow-[0_18px_45px_-28px_rgba(37,99,235,0.55)] dark:from-primary-500/15 dark:via-surface-900/80 dark:to-warning-500/10'
			: 'relative overflow-hidden rounded-2xl border border-surface-200/70 bg-surface-50/70 p-3 shadow-[0_14px_34px_-30px_rgba(15,23,42,0.5)] dark:border-surface-700/70 dark:bg-surface-950/35';

	function volume(value: number): string {
		return `${Math.round(value)}kg`;
	}
</script>

<section
	class="card variant-glass-surface relative overflow-hidden p-4 space-y-3 shadow-[0_18px_50px_-36px_rgba(15,23,42,0.55)]"
>
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
			{#each [overview.last_7_days, overview.last_30_days] as period, index}
				<article class={periodCardClass(index)}>
					<div
						class="absolute inset-x-0 top-0 h-1 {index === 0
							? 'bg-gradient-to-r from-primary-500 via-warning-400 to-tertiary-500'
							: 'bg-surface-200 dark:bg-surface-700'}"
					></div>
					<div class="flex items-baseline justify-between gap-3">
						<h3 class="font-semibold">{period.label}</h3>
						<div
							class="rounded-full border border-surface-200/60 bg-surface-50/70 px-2 py-1 text-xs font-semibold text-surface-700 dark:border-surface-700/60 dark:bg-surface-950/40 dark:text-surface-200"
						>
							{formatSignedNumber(period.comparison.recent_best_count_delta)} recent bests
						</div>
					</div>

					<div class="mt-3 grid grid-cols-2 gap-2">
						<div class={metrics.workouts}>
							<div class="absolute inset-y-3 left-0 w-1 rounded-r-full bg-primary-500/70"></div>
							<div class="text-xs font-semibold opacity-70">Workouts</div>
							<div class="text-lg font-bold">{period.workouts}</div>
							<div class="text-xs opacity-70">
								{formatSignedNumber(period.comparison.workouts_delta)}
							</div>
						</div>
						<div class={metrics.time}>
							<div class="absolute inset-y-3 left-0 w-1 rounded-r-full bg-warning-500/70"></div>
							<div class="text-xs font-semibold opacity-70">Training time</div>
							<div class="text-lg font-bold">{period.total_training_minutes}m</div>
							<div class="text-xs opacity-70">
								{formatSignedNumber(period.comparison.total_training_minutes_delta, 'm')}
							</div>
						</div>
						<div class={metrics.volume}>
							<div class="absolute inset-y-3 left-0 w-1 rounded-r-full bg-success-500/70"></div>
							<div class="text-xs font-semibold opacity-70">Volume</div>
							<div class="text-lg font-bold">{volume(period.total_volume)}</div>
							<div class="text-xs opacity-70">
								{formatSignedNumber(period.comparison.total_volume_delta, 'kg')}
							</div>
						</div>
						<div class={metrics.records}>
							<div class="absolute inset-y-3 left-0 w-1 rounded-r-full bg-tertiary-500/70"></div>
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
