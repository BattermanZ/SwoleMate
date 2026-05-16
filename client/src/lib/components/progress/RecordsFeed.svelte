<script lang="ts">
	import { Card, PRRow } from '$lib/components/ui';
	import { formatDateShort } from '$lib/utils/date';
	import { formatDuration } from '$lib/progress/format';
	import type { RecentPr } from '$lib/types';

	interface Props {
		prs: RecentPr[];
		recentBests: RecentPr[];
	}
	let { prs, recentBests }: Props = $props();

	type Feed = 'prs' | 'recent-bests';
	let feed = $state<Feed>('prs');
	const LIMIT = 5;

	const LABELS: Record<RecentPr['pr_type'], string> = {
		max_weight: 'Max weight',
		estimated_1rm: 'Estimated 1RM',
		rep_pr: 'Rep PR',
		timed_duration: 'Timed duration',
		single_set_volume: 'Single-set volume'
	};

	function formatValue(pr: RecentPr, value: number): string {
		if (pr.pr_type === 'timed_duration') return formatDuration(value);
		if (pr.pr_type === 'rep_pr' || pr.pr_type === 'max_weight' || pr.pr_type === 'estimated_1rm') {
			return `${Math.round(value * 10) / 10} kg`;
		}
		return `${Math.round(value)} kg`;
	}

	function details(pr: RecentPr): string {
		const s = pr.set_details;
		if (s.duration_seconds && s.reps === 0 && s.weight === 0) return 'Timed set';
		if (s.duration_seconds && s.reps > 0) {
			return `${s.reps} reps · ${Math.round(s.weight * 10) / 10}kg · ${formatDuration(s.duration_seconds)}`;
		}
		if (s.duration_seconds) return formatDuration(s.duration_seconds);
		return `${s.reps} reps · ${Math.round(s.weight * 10) / 10}kg`;
	}

	let active = $derived(feed === 'prs' ? prs : recentBests);
	let visible = $derived(active.slice(0, LIMIT));
	let empty = $derived(feed === 'prs' ? 'No all-time PRs yet.' : 'No recent bests yet.');
	let tagLabel = $derived(feed === 'prs' ? 'All-time PR' : 'Recent best');
	let tone = $derived(feed === 'prs' ? 'gold' : 'sage');
</script>

<Card>
	{#snippet title()}Records <em>— milestones + 90d bests</em>{/snippet}
	{#snippet lede()}All-time PRs that ever stood, and rolling 90-day bests.{/snippet}

	<div class="feed-toggle" role="tablist" aria-label="Record type">
		<button
			role="tab"
			aria-selected={feed === 'prs'}
			class:active={feed === 'prs'}
			onclick={() => (feed = 'prs')}
			type="button">All-time PRs</button
		>
		<button
			role="tab"
			aria-selected={feed === 'recent-bests'}
			class:active={feed === 'recent-bests'}
			onclick={() => (feed = 'recent-bests')}
			type="button">Recent bests</button
		>
	</div>

	{#if visible.length === 0}
		<div class="empty">{empty}</div>
	{:else}
		<div class="list">
			{#each visible as pr (`${feed}-${pr.set_id}-${pr.pr_type}`)}
				<PRRow
					exerciseName={pr.exercise_type}
					prTagLabel={tagLabel}
					prType={LABELS[pr.pr_type]}
					newValue={formatValue(pr, pr.new_value)}
					previousValue={formatValue(pr, pr.previous_value)}
					dateLabel={formatDateShort(pr.date)}
					details={details(pr)}
					tone={tone as 'gold' | 'sage'}
				/>
			{/each}
		</div>
	{/if}
</Card>

<style>
	.feed-toggle {
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 999px;
		padding: 3px;
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 2px;
		margin-bottom: 12px;
	}
	.feed-toggle button {
		border: 0;
		background: transparent;
		padding: 8px 0;
		font:
			700 11px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
		cursor: pointer;
		border-radius: 999px;
	}
	.feed-toggle button.active {
		background: linear-gradient(180deg, var(--clay-2), var(--clay));
		color: white;
		box-shadow: 0 4px 10px -3px rgba(255, 94, 31, 0.4);
	}

	.list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.empty {
		font:
			500 13px/1.4 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}
</style>
