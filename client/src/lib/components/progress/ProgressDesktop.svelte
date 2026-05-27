<script lang="ts">
	import { Card } from '$lib/components/ui';
	import type { ExerciseProgress, ProgressOverview, VolumeStats, WorkoutStats } from '$lib/types';
	import PeriodCard from './PeriodCard.svelte';
	import RecordsFeed from './RecordsFeed.svelte';
	import ExerciseFocusPanel from './ExerciseFocusPanel.svelte';
	import ExerciseCharts from './ExerciseCharts.svelte';
	import RecentExerciseSessions from './RecentExerciseSessions.svelte';
	import OverallCharts from './OverallCharts.svelte';
	import SessionHeatmap from './SessionHeatmap.svelte';

	interface Props {
		// hero metrics
		consistencyDone: number;
		totalWorkouts: number;
		perWeek: number;
		perWeekDelta?: number;
		avgDuration: number;
		avgDurationDelta?: number;

		// data
		workoutStats: WorkoutStats | null;
		progressOverview: ProgressOverview | null;
		volumeStats: VolumeStats | null;
		exerciseProgress: ExerciseProgress[] | null;
		exerciseTypes: string[];

		// exercise focus
		selectedExercise: string;
		loadedExercise: string;

		// status
		loadingOverall: boolean;
		loadingExercise: boolean;
		errorOverall: string | null;
		errorExercise: string | null;

		// callbacks
		onRefresh: () => void;
		onSelectExercise: (exercise: string) => void;
	}

	let {
		consistencyDone,
		totalWorkouts,
		perWeek,
		perWeekDelta,
		avgDuration,
		avgDurationDelta,
		workoutStats,
		progressOverview,
		volumeStats,
		exerciseProgress,
		exerciseTypes,
		selectedExercise = $bindable(''),
		loadedExercise,
		loadingOverall,
		loadingExercise,
		errorOverall,
		errorExercise,
		onRefresh,
		onSelectExercise
	}: Props = $props();

	let sessionSamples = $derived(workoutStats?.session_start_samples ?? []);
	let showExerciseDetail = $derived(!!volumeStats && loadedExercise === selectedExercise);

	const RING_CIRC = 427;
	let pct = $derived(Math.min(1, consistencyDone / 30));
	let dashOffset = $derived(RING_CIRC * (1 - Math.max(0.06, pct)));

	function fmtSigned(n: number | undefined, suffix = ''): string | undefined {
		if (n === undefined || n === 0) return undefined;
		const sign = n > 0 ? '+' : '−';
		return `${sign}${Math.abs(n)}${suffix}`;
	}
</script>

<div class="ws">
	<!-- Headline numbers — a horizontal command band across the full width. -->
	<header class="dhero">
		<div class="ring-block">
			<div class="dial">
				<svg width="118" height="118" viewBox="0 0 150 150" aria-hidden="true">
					<circle
						cx="75"
						cy="75"
						r="68"
						stroke="rgba(243,236,225,0.1)"
						stroke-width="10"
						fill="none"
					/>
					<circle
						cx="75"
						cy="75"
						r="68"
						stroke="url(#dhero-ring)"
						stroke-width="10"
						fill="none"
						stroke-dasharray={RING_CIRC}
						stroke-dashoffset={dashOffset}
						stroke-linecap="round"
						transform="rotate(-90 75 75)"
					/>
					<defs>
						<linearGradient id="dhero-ring" x1="0" y1="0" x2="1" y2="1">
							<stop offset="0%" stop-color="#ff7a2a" />
							<stop offset="100%" stop-color="#ff5e1f" />
						</linearGradient>
					</defs>
				</svg>
				<div class="ring-center"><span class="big">{consistencyDone}<small>/30</small></span></div>
			</div>
			<span class="ring-cap">Days trained · 30d</span>
		</div>

		<div class="dstats">
			<div class="cell">
				<div class="k">Total workouts</div>
				<div class="v">{totalWorkouts}</div>
			</div>
			<div class="cell">
				<div class="k">Per week</div>
				<div class="v">{perWeek.toFixed(1)}</div>
				{#if fmtSigned(perWeekDelta)}
					<div class="d" class:up={(perWeekDelta ?? 0) > 0} class:down={(perWeekDelta ?? 0) < 0}>
						{fmtSigned(perWeekDelta)} last 4w
					</div>
				{/if}
			</div>
			<div class="cell">
				<div class="k">Avg duration</div>
				<div class="v">{avgDuration}<small>m</small></div>
				{#if fmtSigned(avgDurationDelta, 'm')}
					<div
						class="d"
						class:up={(avgDurationDelta ?? 0) > 0}
						class:down={(avgDurationDelta ?? 0) < 0}
					>
						{fmtSigned(avgDurationDelta, 'm')} last 4w
					</div>
				{/if}
			</div>
			<div class="cell">
				<div class="k">Focus</div>
				<div class="v fx">{selectedExercise || '—'}</div>
			</div>
		</div>

		<div class="dhero-foot">
			{#if errorOverall}<span class="err">{errorOverall}</span>{/if}
			<button class="refresh" type="button" onclick={onRefresh} disabled={loadingOverall}>
				↻ Refresh
			</button>
		</div>
	</header>

	<!-- Consistency centerpiece -->
	{#if sessionSamples.length > 0}
		<SessionHeatmap samples={sessionSamples} />
	{/if}

	<!-- Two-column workspace: per-exercise deep dive beside the aggregate story. -->
	<div class="split">
		<section class="band">
			<header class="band-head">
				<span class="tag">Deep dive</span>
				<h2>Exercise analysis</h2>
			</header>

			<ExerciseFocusPanel
				bind:selectedExercise
				{exerciseTypes}
				{volumeStats}
				{loadingExercise}
				{errorExercise}
				onSelect={onSelectExercise}
			/>

			{#if showExerciseDetail}
				<div class="grid-2">
					<ExerciseCharts {volumeStats} {exerciseProgress} />
					<RecentExerciseSessions {exerciseProgress} />
				</div>
			{/if}
		</section>

		{#if progressOverview}
			<section class="band">
				<header class="band-head">
					<span class="tag">Momentum</span>
					<h2>Current progress <em>— vs the window before</em></h2>
				</header>
				<div class="stack">
					<Card>
						{#snippet title()}Period comparison{/snippet}
						{#snippet lede()}Recent work against the matching window before it.{/snippet}
						<div class="periods">
							<PeriodCard period={progressOverview.last_7_days} variant="primary" />
							<PeriodCard period={progressOverview.last_30_days} variant="secondary" />
						</div>
					</Card>
					<RecordsFeed
						prs={progressOverview.recent_prs}
						recentBests={progressOverview.recent_bests}
					/>
				</div>
			</section>
		{/if}
	</div>

	<!-- Training patterns — full width, charts flow 2–3 up. -->
	{#if workoutStats}
		<section class="band">
			<header class="band-head">
				<span class="tag">Patterns</span>
				<h2>Training patterns <em>— how and when you train</em></h2>
			</header>
			<div class="charts">
				<OverallCharts {workoutStats} />
			</div>
		</section>
	{/if}
</div>

<style>
	.ws {
		max-width: 1480px;
		margin: 0 auto;
		display: flex;
		flex-direction: column;
		gap: 28px;
	}

	/* ── Horizontal hero ─────────────────────────────────────── */
	.dhero {
		position: relative;
		overflow: hidden;
		background: var(--surface-deep);
		color: var(--on-deep);
		border-radius: 28px;
		padding: 22px 26px;
		display: flex;
		align-items: center;
		gap: 28px;
		box-shadow: 0 24px 48px -16px var(--shadow-strong);
	}
	.dhero::before {
		content: '';
		position: absolute;
		top: -130px;
		right: -90px;
		width: 320px;
		height: 320px;
		border-radius: 50%;
		background: radial-gradient(circle, rgba(255, 94, 31, 0.5), transparent 65%);
		pointer-events: none;
	}
	.ring-block {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		flex: none;
		z-index: 1;
	}
	.dial {
		width: 118px;
		height: 118px;
		position: relative;
	}
	.ring-center {
		position: absolute;
		inset: 0;
		display: grid;
		place-items: center;
	}
	.big {
		font:
			800 30px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: -0.04em;
		font-variant-numeric: tabular-nums;
		color: var(--on-deep);
		display: flex;
		align-items: baseline;
	}
	.big small {
		font:
			500 13px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--on-deep-soft);
		margin-left: 1px;
	}
	.ring-cap {
		font:
			700 9px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.16em;
		text-transform: uppercase;
		color: var(--on-deep-soft);
	}

	.dstats {
		flex: 1;
		display: grid;
		grid-template-columns: repeat(4, minmax(0, 1fr));
		gap: 10px;
		z-index: 1;
	}
	.cell {
		background: color-mix(in oklab, var(--on-deep) 7%, transparent);
		border: 1px solid color-mix(in oklab, var(--on-deep) 12%, transparent);
		border-radius: 14px;
		padding: 14px 16px;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 7px;
	}
	.k {
		font:
			700 10px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.16em;
		text-transform: uppercase;
		color: var(--on-deep-soft);
	}
	.v {
		font:
			800 26px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: -0.025em;
		font-variant-numeric: tabular-nums;
		color: var(--on-deep);
		display: flex;
		align-items: baseline;
		gap: 3px;
	}
	.v small {
		font:
			600 12px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--on-deep-soft);
	}
	.v.fx {
		font-size: 17px;
		letter-spacing: -0.01em;
		color: var(--clay-2);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		display: block;
	}
	.d {
		font:
			600 11px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--on-deep-soft);
	}
	.d.up {
		color: var(--sage);
	}
	.d.down {
		color: var(--clay-2);
	}

	.dhero-foot {
		display: flex;
		align-items: center;
		gap: 10px;
		flex: none;
		z-index: 1;
	}
	.refresh {
		padding: 10px 16px;
		border-radius: 999px;
		border: 1px solid var(--on-deep-line);
		background: color-mix(in oklab, var(--on-deep) 6%, transparent);
		color: var(--on-deep);
		font:
			700 12px/1 'Onest',
			system-ui,
			sans-serif;
		cursor: pointer;
		white-space: nowrap;
	}
	.refresh:disabled {
		opacity: 0.55;
		cursor: not-allowed;
	}
	.err {
		font:
			600 12px/1.4 'Onest',
			system-ui,
			sans-serif;
		color: var(--clay-2);
		max-width: 220px;
	}

	/* ── Band headers ────────────────────────────────────────── */
	.band-head {
		display: flex;
		align-items: baseline;
		gap: 12px;
		margin-bottom: 14px;
	}
	.tag {
		font:
			800 10px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.22em;
		text-transform: uppercase;
		color: var(--clay-text);
		background: color-mix(in oklab, var(--clay) 9%, var(--card));
		border: 1px solid color-mix(in oklab, var(--clay) 22%, transparent);
		padding: 6px 10px;
		border-radius: 999px;
		flex: none;
	}
	.band-head h2 {
		margin: 0;
		font:
			800 19px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: -0.02em;
		color: var(--ink);
	}
	.band-head h2 em {
		font:
			italic 400 15px/1 'Instrument Serif',
			serif;
		color: var(--ink-soft);
		margin-left: 4px;
	}

	/* ── Two-column macro split ──────────────────────────────── */
	.split {
		display: grid;
		grid-template-columns: minmax(0, 1.5fr) minmax(0, 1fr);
		gap: 28px;
		align-items: start;
	}
	.stack {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
	.periods {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	/* Deep-dive charts + recent sessions in a balanced 2×2 (no orphan row). */
	.grid-2 {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
		gap: 16px;
		margin-top: 16px;
	}
	.grid-2 > :global(*) {
		min-width: 0;
	}

	/* Patterns charts fill the page width. */
	.charts {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
		gap: 16px;
	}
	.charts > :global(*) {
		min-width: 0;
	}

	/* Collapse the split below a comfortable two-pane width. */
	@media (max-width: 1279px) {
		.split {
			grid-template-columns: 1fr;
		}
	}
</style>
