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

	const selectedButtonClass =
		'bg-primary-500 text-white shadow-[0_8px_18px_-12px_rgba(37,99,235,0.75)]';
	const quietButtonClass =
		'text-surface-700 hover:bg-surface-100 dark:text-surface-200 dark:hover:bg-surface-800/70';
	const recordRowClass =
		'relative overflow-hidden rounded-xl border bg-surface-50/80 p-3 shadow-[0_10px_28px_-24px_rgba(15,23,42,0.6)] dark:bg-surface-950/35';

	$: activeBadgeClass =
		selectedFeed === 'prs'
			? 'border-warning-500/30 bg-warning-500/15 text-warning-700 dark:text-warning-300'
			: 'border-success-500/30 bg-success-500/15 text-success-700 dark:text-success-300';
	$: activeRailClass = selectedFeed === 'prs' ? 'bg-warning-500/80' : 'bg-success-500/80';
	$: rowBorderClass =
		selectedFeed === 'prs'
			? 'border-warning-500/25 dark:border-warning-500/25'
			: 'border-success-500/25 dark:border-success-500/25';
</script>

<section
	class="card variant-glass-surface overflow-hidden p-4 shadow-[0_18px_50px_-36px_rgba(15,23,42,0.55)]"
>
	<div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
		<div>
			<h2 class="text-lg font-semibold tracking-tight">Records</h2>
			<p class="text-sm opacity-70">All-time milestones and rolling 90-day bests.</p>
		</div>

		<div
			class="grid grid-cols-2 rounded-xl border border-surface-200/70 bg-surface-100/70 p-1 shadow-inner dark:border-surface-700/70 dark:bg-surface-950/45"
			role="tablist"
			aria-label="Record type"
		>
			<button
				type="button"
				role="tab"
				aria-selected={selectedFeed === 'prs'}
				class="rounded-lg px-3 py-1.5 text-xs font-semibold transition {selectedFeed === 'prs'
					? selectedButtonClass
					: quietButtonClass}"
				on:click={() => (selectedFeed = 'prs')}
			>
				All-time PRs
			</button>
			<button
				type="button"
				role="tab"
				aria-selected={selectedFeed === 'recent-bests'}
				class="rounded-lg px-3 py-1.5 text-xs font-semibold transition {selectedFeed ===
				'recent-bests'
					? selectedButtonClass
					: quietButtonClass}"
				on:click={() => (selectedFeed = 'recent-bests')}
			>
				Recent bests
			</button>
		</div>
	</div>

	{#if visibleRecords.length}
		<div class="mt-3 space-y-2">
			{#each visibleRecords as pr (`${selectedFeed}-${pr.set_id}-${pr.pr_type}`)}
				<div class="{recordRowClass} {rowBorderClass}">
					<div class="absolute inset-y-3 left-0 w-1 rounded-r-full {activeRailClass}"></div>
					<div class="flex flex-wrap items-start justify-between gap-2">
						<div class="min-w-0">
							<div class="font-semibold">{pr.exercise_type}</div>
							<div class="mt-1 text-sm text-surface-700 dark:text-surface-200">
								<span
									class="mr-1 inline-flex rounded-full border px-2 py-0.5 text-[0.7rem] font-bold leading-none {activeBadgeClass}"
								>
									{badgeLabel}
								</span>
								<span class="font-semibold">{labels[pr.pr_type]}</span>
								<span
									class="text-base font-extrabold tracking-tight text-surface-950 dark:text-surface-50"
								>
									{formatValue(pr, pr.new_value)}
								</span>
								<span class="opacity-60">from {formatValue(pr, pr.previous_value)}</span>
							</div>
						</div>
						<div
							class="rounded-full border border-surface-200/70 bg-surface-50/70 px-2 py-1 text-xs font-semibold text-surface-600 dark:border-surface-700/70 dark:bg-surface-900/60 dark:text-surface-300"
						>
							{formatDateShort(pr.date)}
						</div>
					</div>
					<div class="mt-2 text-xs font-medium opacity-70">{details(pr)}</div>
				</div>
			{/each}
		</div>
	{:else}
		<div class="mt-3 text-sm opacity-70">{emptyMessage}</div>
	{/if}
</section>
