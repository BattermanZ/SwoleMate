<script lang="ts">
	import { getWorkouts } from '$lib/api';
	import type { FeedbackEmoji, Workout } from '$lib/types';
	import { logger } from '$lib/logger';
	import { formatDateRelative } from '$lib/utils/date';
	import { isWithinRange, resolveDateRange, type DateRangePreset } from '$lib/history/dateRange';
	import { Btn, Card, Badge, PageHero, MetricTile, Notice } from '$lib/components/ui';
	import HistoryDesktop from '$lib/components/history/HistoryDesktop.svelte';
	import { isDesktop, isDesktopView } from '$lib/stores/viewport';

	interface Props {
		data: { workouts: Workout[] };
	}
	let { data }: Props = $props();

	let desktop = $derived(isDesktopView($isDesktop));

	let workouts = $derived(data.workouts);

	let loading = $state(false);
	let error = $state<string | null>(null);

	type SortOrder = 'newest' | 'oldest' | 'longest' | 'shortest';

	let query = $state('');
	let rangePreset = $state<DateRangePreset>('all');
	let customFrom = $state('');
	let customTo = $state('');
	let mood = $state<'all' | FeedbackEmoji>('all');
	let sort = $state<SortOrder>('newest');
	let pageIndex = $state(0);
	const PAGE_SIZE = 20;

	function workoutDurationMinutes(w: Workout): number | null {
		const s = new Date(w.start_time).getTime();
		const e = new Date(w.end_time).getTime();
		if (!Number.isFinite(s) || !Number.isFinite(e) || e <= s) return null;
		return Math.round((e - s) / 60_000);
	}

	let dateRange = $derived(resolveDateRange(rangePreset, customFrom, customTo));

	function matchesQuery(w: Workout, q: string): boolean {
		const term = q.trim().toLowerCase();
		if (!term) return true;
		const hay = `${w.date} ${w.notes ?? ''} ${formatDateRelative(w.start_time)}`.toLowerCase();
		return hay.includes(term);
	}

	let sortedWorkouts = $derived(
		[...workouts].sort((a, b) => {
			if (sort === 'longest' || sort === 'shortest') {
				const ad = workoutDurationMinutes(a);
				const bd = workoutDurationMinutes(b);
				const aV = ad ?? Number.POSITIVE_INFINITY;
				const bV = bd ?? Number.POSITIVE_INFINITY;
				return sort === 'shortest' ? aV - bV : bV - aV;
			}
			const at = new Date(a.start_time).getTime();
			const bt = new Date(b.start_time).getTime();
			if (sort === 'oldest') return (Number.isFinite(at) ? at : 0) - (Number.isFinite(bt) ? bt : 0);
			return (Number.isFinite(bt) ? bt : 0) - (Number.isFinite(at) ? at : 0);
		})
	);

	let filteredWorkouts = $derived(
		sortedWorkouts.filter((w) => {
			if (mood !== 'all' && w.feedback !== mood) return false;
			if (!isWithinRange(w.start_time, dateRange)) return false;
			if (!matchesQuery(w, query)) return false;
			return true;
		})
	);

	let pageCount = $derived(Math.max(1, Math.ceil(filteredWorkouts.length / PAGE_SIZE)));
	let pagedWorkouts = $derived(
		filteredWorkouts.slice(pageIndex * PAGE_SIZE, pageIndex * PAGE_SIZE + PAGE_SIZE)
	);

	// reset page on filter changes
	let lastKey = $state('');
	$effect(() => {
		const k = [query, rangePreset, customFrom, customTo, mood, sort].join('|');
		if (k !== lastKey) {
			pageIndex = 0;
			lastKey = k;
		}
	});
	$effect(() => {
		// keep pageIndex valid when page count changes
		if (pageIndex > pageCount - 1) pageIndex = Math.max(0, pageCount - 1);
	});

	let summary = $derived.by(() => {
		const total = filteredWorkouts.length;
		const durations = filteredWorkouts
			.map(workoutDurationMinutes)
			.filter((d): d is number => d !== null);
		const avg = durations.length
			? Math.round(durations.reduce((a, b) => a + b, 0) / durations.length)
			: null;
		const last30 = filteredWorkouts.filter((w) => {
			const t = new Date(w.start_time).getTime();
			return Number.isFinite(t) && t >= Date.now() - 30 * 24 * 60 * 60 * 1000;
		}).length;
		return { total, avg, last30 };
	});

	let pageStart = $derived(filteredWorkouts.length ? pageIndex * PAGE_SIZE + 1 : 0);
	let pageEnd = $derived(Math.min(filteredWorkouts.length, (pageIndex + 1) * PAGE_SIZE));
	let canPrev = $derived(pageIndex > 0);
	let canNext = $derived(pageIndex < pageCount - 1);

	async function refresh() {
		loading = true;
		error = null;
		try {
			workouts = await getWorkouts();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load workouts';
			logger.error('workout', 'refresh failed', { error: e });
		} finally {
			loading = false;
		}
	}
</script>

{#snippet hero()}
	<PageHero kicker="► History">
		{#snippet title()}Past <em>sessions.</em>{/snippet}
		{#snippet sub()}Filter, sort, and review notes + set schemes from every workout.{/snippet}
	</PageHero>
{/snippet}

{#snippet metrics()}
	<div class="metrics">
		<MetricTile label="Total" value={String(summary.total)} rail="clay" />
		<MetricTile label="Last 30d" value={String(summary.last30)} rail="warn" />
		<MetricTile
			label="Avg duration"
			value={summary.avg !== null ? `${summary.avg}` : '—'}
			unit="m"
			rail="sage"
		/>
	</div>
{/snippet}

{#snippet filters()}
	<Card>
		{#snippet title()}Filters{/snippet}
		{#snippet actions()}
			<Btn variant="soft" size="sm" onclick={refresh} disabled={loading}>
				{loading ? 'Loading…' : 'Refresh'}
			</Btn>
		{/snippet}

		<div class="filters">
			<label>
				<span class="lbl">Search</span>
				<input bind:value={query} placeholder="Notes, date…" disabled={loading} />
			</label>
			<label>
				<span class="lbl">Date range</span>
				<select bind:value={rangePreset} disabled={loading}>
					<option value="all">All</option>
					<option value="30d">Last 30 days</option>
					<option value="90d">Last 90 days</option>
					<option value="365d">Last year</option>
					<option value="custom">Custom…</option>
				</select>
			</label>
			{#if rangePreset === 'custom'}
				<label>
					<span class="lbl">From</span>
					<input type="date" bind:value={customFrom} disabled={loading} />
				</label>
				<label>
					<span class="lbl">To</span>
					<input type="date" bind:value={customTo} disabled={loading} />
				</label>
			{/if}
			<label>
				<span class="lbl">Mood</span>
				<select bind:value={mood} disabled={loading}>
					<option value="all">All</option>
					<option value="😊">😊 Good</option>
					<option value="😐">😐 Neutral</option>
					<option value="😞">😞 Bad</option>
				</select>
			</label>
			<label>
				<span class="lbl">Sort</span>
				<select bind:value={sort} disabled={loading}>
					<option value="newest">Newest first</option>
					<option value="oldest">Oldest first</option>
					<option value="longest">Longest first</option>
					<option value="shortest">Shortest first</option>
				</select>
			</label>
		</div>

		{#if error}<div class="err-wrap"><Notice tone="error">{error}</Notice></div>{/if}
	</Card>
{/snippet}

{#snippet list()}
	{#if filteredWorkouts.length === 0}
		<Card>
			<div class="empty">
				{#if workouts.length === 0}No workouts yet.{:else}No workouts match the filters.{/if}
			</div>
		</Card>
	{:else}
		<div class="pager">
			<div class="muted">Showing {pageStart}–{pageEnd} of {filteredWorkouts.length}</div>
			<div class="pager-btns">
				<Btn
					variant="soft"
					size="sm"
					disabled={!canPrev}
					onclick={() => (pageIndex = Math.max(0, pageIndex - 1))}>←</Btn
				>
				<span class="pager-pos">{pageIndex + 1}/{pageCount}</span>
				<Btn
					variant="soft"
					size="sm"
					disabled={!canNext}
					onclick={() => (pageIndex = Math.min(pageCount - 1, pageIndex + 1))}>→</Btn
				>
			</div>
		</div>

		<div class="list">
			{#each pagedWorkouts as w (w.id)}
				<a class="w-card" href="/workouts/{w.id}">
					<div class="row">
						<div class="left">
							<div class="day">
								{formatDateRelative(w.start_time)}
								{#if w.feedback}<span class="mood">{w.feedback}</span>{/if}
							</div>
							<div class="meta">
								<span>{workoutDurationMinutes(w) ?? '—'}m</span>
								{#if w.exercise_count}<span>· {w.exercise_count} exercises</span>{/if}
								{#if w.auto_closed_at}<Badge tone="warn">Auto-closed</Badge>{/if}
							</div>
							{#if w.notes}<p class="notes">{w.notes}</p>{/if}
						</div>
						<span class="arr" aria-hidden="true">→</span>
					</div>
				</a>
			{/each}
		</div>
	{/if}
{/snippet}

{#if desktop}
	<HistoryDesktop {hero} {metrics} {filters} {list} />
{:else}
	<div class="page">
		{@render hero()}
		{@render metrics()}
		{@render filters()}
		{@render list()}
	</div>
{/if}

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.metrics {
		display: grid;
		grid-template-columns: 1fr 1fr 1fr;
		gap: 8px;
	}
	@media (max-width: 480px) {
		.metrics {
			grid-template-columns: 1fr 1fr;
		}
		.metrics :global(.tile:last-child) {
			grid-column: 1 / -1;
		}
	}
	.filters {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 10px;
	}
	@media (max-width: 480px) {
		.filters {
			grid-template-columns: 1fr;
		}
	}
	label {
		display: block;
	}
	.lbl {
		display: block;
		font:
			700 10px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--ink-soft);
		margin-bottom: 6px;
	}
	input,
	select {
		width: 100%;
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 10px;
		padding: 10px 12px;
		font:
			500 14px/1.2 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink);
		outline: 0;
	}
	input:focus,
	select:focus {
		border-color: var(--clay);
		box-shadow: 0 0 0 3px rgba(255, 94, 31, 0.16);
	}
	.err-wrap {
		margin-top: 10px;
	}

	.pager {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 10px;
		padding: 0 4px;
	}
	.pager-btns {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.pager-pos {
		font:
			700 12px/1 'JetBrains Mono',
			monospace;
		color: var(--ink-2);
	}
	.muted {
		font:
			500 12px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}

	.list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	/* Desktop main column is wide enough to flow session cards 2-up. */
	@media (min-width: 1024px) {
		.list {
			display: grid;
			grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
			align-items: start;
		}
	}
	.w-card {
		display: block;
		text-decoration: none;
		color: inherit;
		background: var(--card);
		border: 1px solid var(--line);
		border-radius: 16px;
		padding: 14px 16px;
		box-shadow: 0 4px 12px -8px var(--shadow-card);
	}
	.w-card:hover {
		border-color: color-mix(in oklab, var(--clay) 40%, var(--line));
	}
	.row {
		display: flex;
		justify-content: space-between;
		align-items: start;
		gap: 12px;
	}
	.left {
		min-width: 0;
	}
	.day {
		font:
			800 15px/1 'Onest',
			system-ui,
			sans-serif;
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.mood {
		font:
			400 14px/1 'Onest',
			system-ui,
			sans-serif;
		opacity: 0.85;
	}
	.meta {
		margin-top: 6px;
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
		font:
			500 12px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
		align-items: center;
	}
	.notes {
		margin: 6px 0 0;
		font: italic 400 13px/1.4 'Instrument Serif';
		color: var(--ink-2);
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}
	.arr {
		flex: none;
		font:
			800 18px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}
	.empty {
		font:
			500 13px/1.4 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
		text-align: center;
	}
</style>
