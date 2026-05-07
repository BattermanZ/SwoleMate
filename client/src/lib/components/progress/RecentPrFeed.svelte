<script lang="ts">
	import type { RecentPr } from '$lib/types';
	import { formatDateShort } from '$lib/utils/date';
	import { formatDuration } from '$lib/progress/format';

	export let prs: RecentPr[] = [];
	export let recentBests: RecentPr[] = [];

	type RecordFeed = 'prs' | 'recent-bests';
	let selectedFeed: RecordFeed = 'prs';

	const visibleLimit = 5;

	const labels: Record<RecentPr['pr_type'], string> = {
		max_weight: 'Max weight',
		estimated_1rm: 'Estimated 1RM',
		rep_pr: 'Rep PR',
		timed_duration: 'Timed duration',
		single_set_volume: 'Single-set volume'
	};

	function formatValue(pr: RecentPr, value: number): string {
		if (pr.pr_type === 'timed_duration') return formatDuration(value);
		if (pr.pr_type === 'rep_pr' || pr.pr_type === 'max_weight' || pr.pr_type === 'estimated_1rm') {
			return `${Math.round(value * 10) / 10}kg`;
		}
		return `${Math.round(value)}kg`;
	}

	function details(pr: RecentPr): string {
		const set = pr.set_details;
		if (set.duration_seconds && set.reps === 0 && set.weight === 0) return 'Timed set';
		if (set.duration_seconds && set.reps > 0) {
			return `${set.reps} reps · ${Math.round(set.weight * 10) / 10}kg · ${formatDuration(
				set.duration_seconds
			)}`;
		}
		if (set.duration_seconds) return formatDuration(set.duration_seconds);
		return `${set.reps} reps · ${Math.round(set.weight * 10) / 10}kg`;
	}

	$: activeRecords = selectedFeed === 'prs' ? prs : recentBests;
	$: visibleRecords = activeRecords.slice(0, visibleLimit);
	$: emptyMessage = selectedFeed === 'prs' ? 'No all-time PRs yet.' : 'No recent bests yet.';
	$: badgeLabel = selectedFeed === 'prs' ? 'All-time PR' : 'Recent best';
</script>

<section class="card variant-glass-surface p-4">
	<div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
		<div>
			<h2 class="text-lg font-semibold tracking-tight">Records</h2>
			<p class="text-sm opacity-70">All-time milestones and rolling 90-day bests.</p>
		</div>

		<div
			class="grid grid-cols-2 rounded-lg border border-surface-200/70 bg-surface-50/70 p-1 dark:border-surface-700/70 dark:bg-surface-950/40"
			role="tablist"
			aria-label="Record type"
		>
			<button
				type="button"
				role="tab"
				aria-selected={selectedFeed === 'prs'}
				class="rounded-md px-3 py-1.5 text-xs font-semibold transition-colors {selectedFeed ===
				'prs'
					? 'bg-primary-500 text-white shadow-sm'
					: 'text-surface-700 hover:bg-surface-100 dark:text-surface-200 dark:hover:bg-surface-800/70'}"
				on:click={() => (selectedFeed = 'prs')}
			>
				All-time PRs
			</button>
			<button
				type="button"
				role="tab"
				aria-selected={selectedFeed === 'recent-bests'}
				class="rounded-md px-3 py-1.5 text-xs font-semibold transition-colors {selectedFeed ===
				'recent-bests'
					? 'bg-primary-500 text-white shadow-sm'
					: 'text-surface-700 hover:bg-surface-100 dark:text-surface-200 dark:hover:bg-surface-800/70'}"
				on:click={() => (selectedFeed = 'recent-bests')}
			>
				Recent bests
			</button>
		</div>
	</div>

	{#if visibleRecords.length}
		<div class="mt-3 space-y-2">
			{#each visibleRecords as pr (`${selectedFeed}-${pr.set_id}-${pr.pr_type}`)}
				<div
					class="rounded-xl border border-surface-200/50 bg-surface-50/60 p-3 dark:border-surface-700/50 dark:bg-surface-950/30"
				>
					<div class="flex flex-wrap items-start justify-between gap-2">
						<div class="min-w-0">
							<div class="font-semibold">{pr.exercise_type}</div>
							<div class="text-sm opacity-75">
								<span class="badge variant-soft-primary mr-1">{badgeLabel}</span>
								{labels[pr.pr_type]} · {formatValue(pr, pr.new_value)}
								<span class="opacity-60">from {formatValue(pr, pr.previous_value)}</span>
							</div>
						</div>
						<div class="text-xs font-semibold opacity-60">{formatDateShort(pr.date)}</div>
					</div>
					<div class="mt-2 text-xs opacity-70">{details(pr)}</div>
				</div>
			{/each}
		</div>
	{:else}
		<div class="mt-3 text-sm opacity-70">{emptyMessage}</div>
	{/if}
</section>
