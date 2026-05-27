<script lang="ts">
	import { Card } from '$lib/components/ui';
	import type { ExerciseProgress, ProgressOverview, VolumeStats, WorkoutStats } from '$lib/types';
	import ProgressHero from './ProgressHero.svelte';
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

	let sessionDates = $derived(workoutStats?.session_start_times ?? []);
	let showExerciseDetail = $derived(!!volumeStats && loadedExercise === selectedExercise);
</script>

<div class="ws">
	<!-- Macro: who you are, this season -->
	<section class="hero-slot">
		<ProgressHero
			{consistencyDone}
			consistencyWindow={30}
			{totalWorkouts}
			{perWeek}
			{perWeekDelta}
			avgDurationMin={avgDuration}
			{avgDurationDelta}
			focusExercise={selectedExercise || undefined}
			loading={loadingOverall}
			error={errorOverall}
			{onRefresh}
		/>
	</section>

	<!-- Consistency centerpiece -->
	{#if sessionDates.length > 0}
		<section class="full">
			<SessionHeatmap dates={sessionDates} />
		</section>
	{/if}

	<!-- Current progress + records -->
	{#if progressOverview}
		<section class="band">
			<header class="band-head">
				<span class="tag">Momentum</span>
				<h2>Current progress <em>— vs the matching window before</em></h2>
			</header>
			<div class="two">
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

	<!-- Exercise deep-dive -->
	<section class="band">
		<header class="band-head">
			<span class="tag">Deep dive</span>
			<h2>
				Exercise analysis{#if selectedExercise}
					<em>— {selectedExercise}</em>{/if}
			</h2>
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
			<div class="charts">
				<ExerciseCharts {volumeStats} {exerciseProgress} />
			</div>
			<RecentExerciseSessions {exerciseProgress} />
		{/if}
	</section>

	<!-- Training patterns -->
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

	/* Two-up region for period comparison + records. */
	.two {
		display: grid;
		grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
		gap: 16px;
		align-items: start;
	}
	.periods {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	/* Multi-card charts flow into a responsive grid that fills the width. The
	   reused chart components each emit sibling ChartCards, so they become grid
	   items directly. */
	.charts {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(340px, 1fr));
		gap: 16px;
		margin-top: 16px;
	}
	.charts > :global(*) {
		min-width: 0;
	}

	.band > :global(:not(.band-head):not(.charts) + *) {
		margin-top: 16px;
	}

	/* Wide screens: give the deep-dive a touch more room per chart. */
	@media (min-width: 1500px) {
		.charts {
			grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
		}
	}
</style>
