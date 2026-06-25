<script lang="ts">
	import { onMount } from 'svelte';
	import {
		getExerciseProgress,
		getExerciseTypes,
		getProgressOverview,
		getVolumeStats,
		getWorkoutStats
	} from '$lib/api';
	import type { ExerciseProgress, ProgressOverview, VolumeStats, WorkoutStats } from '$lib/types';
	import { logger } from '$lib/logger';
	import { Card, SegmentedTabs } from '$lib/components/ui';
	import { isDesktop, isDesktopView } from '$lib/stores/viewport';
	import ProgressHero from '$lib/components/progress/ProgressHero.svelte';
	import PeriodCard from '$lib/components/progress/PeriodCard.svelte';
	import RecordsFeed from '$lib/components/progress/RecordsFeed.svelte';
	import ExerciseFocusPanel from '$lib/components/progress/ExerciseFocusPanel.svelte';
	import ExerciseCharts from '$lib/components/progress/ExerciseCharts.svelte';
	import RecentExerciseSessions from '$lib/components/progress/RecentExerciseSessions.svelte';
	import OverallCharts from '$lib/components/progress/OverallCharts.svelte';
	import ProgressDesktop from '$lib/components/progress/ProgressDesktop.svelte';
	import SessionHeatmap from '$lib/components/progress/SessionHeatmap.svelte';

	type ProgressTab = 'overview' | 'exercise' | 'trends';

	let selectedTab = $state<ProgressTab>('overview');
	let selectedExercise = $state('');
	let loadedExercise = $state('');
	let exerciseTypes = $state<string[]>([]);
	let workoutStats = $state<WorkoutStats | null>(null);
	let progressOverview = $state<ProgressOverview | null>(null);
	let volumeStats = $state<VolumeStats | null>(null);
	let exerciseProgress = $state<ExerciseProgress[] | null>(null);

	let loadingOverall = $state(false);
	let loadingExercise = $state(false);
	let errorOverall = $state<string | null>(null);
	let errorExercise = $state<string | null>(null);

	let exerciseRequestId = 0;
	let exerciseTypesLoaded = false;

	function isProgressTab(v: string | null): v is ProgressTab {
		return v === 'overview' || v === 'exercise' || v === 'trends';
	}

	function getErrorMessage(e: unknown): string {
		return e instanceof Error ? e.message : 'Something went wrong';
	}

	async function loadOverall() {
		loadingOverall = true;
		errorOverall = null;
		try {
			const [stats, overview] = await Promise.all([getWorkoutStats(), getProgressOverview()]);
			workoutStats = stats;
			progressOverview = overview;
		} catch (e) {
			errorOverall = getErrorMessage(e);
			progressOverview = null;
			logger.error('progress', 'workout stats failed', { error: e });
		} finally {
			loadingOverall = false;
		}
	}

	async function loadExerciseTypes(): Promise<string> {
		try {
			exerciseTypes = await getExerciseTypes();
			exerciseTypesLoaded = true;
			const stored = localStorage.getItem('progress.selectedExercise');
			selectedExercise =
				stored && exerciseTypes.includes(stored) ? stored : (exerciseTypes[0] ?? '');
			return selectedExercise;
		} catch (e) {
			exerciseTypes = [];
			selectedExercise = loadedExercise = '';
			volumeStats = exerciseProgress = null;
			exerciseTypesLoaded = true;
			errorExercise = getErrorMessage(e);
			return '';
		}
	}

	async function loadExercise(exercise = selectedExercise) {
		const requestId = ++exerciseRequestId;

		if (!exercise) {
			loadedExercise = '';
			volumeStats = exerciseProgress = null;
			loadingExercise = false;
			return;
		}

		loadingExercise = true;
		errorExercise = null;
		loadedExercise = '';
		volumeStats = exerciseProgress = null;

		try {
			localStorage.setItem('progress.selectedExercise', exercise);
			const [v, ep] = await Promise.all([getVolumeStats(exercise), getExerciseProgress(exercise)]);
			if (requestId !== exerciseRequestId || exercise !== selectedExercise) return;
			volumeStats = v;
			exerciseProgress = ep;
			loadedExercise = exercise;
		} catch (e) {
			if (requestId !== exerciseRequestId) return;
			errorExercise = getErrorMessage(e);
		} finally {
			if (requestId === exerciseRequestId) loadingExercise = false;
		}
	}

	async function refreshAll() {
		const [, exercise] = await Promise.all([loadOverall(), loadExerciseTypes()]);
		if (selectedTab === 'exercise' && exercise) await loadExercise(exercise);
	}

	async function ensureExerciseTabLoaded() {
		const exercise = exerciseTypesLoaded ? selectedExercise : await loadExerciseTypes();
		if (exercise && exercise !== loadedExercise) {
			await loadExercise(exercise);
		}
	}

	function onTabSelect(id: ProgressTab) {
		selectedTab = id;
		localStorage.setItem('progress.selectedTab', id);
		if (id === 'exercise') void ensureExerciseTabLoaded();
	}

	function onSelectExercise(exercise: string) {
		selectedExercise = exercise;
		if (exercise && exercise !== loadedExercise) void loadExercise(exercise);
	}

	onMount(async () => {
		const stored = localStorage.getItem('progress.selectedTab');
		if (isProgressTab(stored)) selectedTab = stored;
		await refreshAll();
		// Desktop shows the exercise deep-dive inline (no tabs), so its data must
		// be loaded regardless of the persisted mobile tab.
		if (selectedTab === 'exercise' || isDesktopView($isDesktop)) await ensureExerciseTabLoaded();
	});

	let desktop = $derived(isDesktopView($isDesktop));

	// When the viewport crosses into desktop after mount (e.g. window resize),
	// the deep-dive becomes visible — make sure its data is present.
	$effect(() => {
		if (desktop) void ensureExerciseTabLoaded();
	});

	// Hero stats
	let perWeek = $derived(workoutStats?.workout_frequency.average_per_week ?? 0);
	let perWeekTrend = $derived(workoutStats?.workout_frequency.trend);
	let avgDuration = $derived(Math.round(workoutStats?.average_duration_minutes ?? 0));
	let avgDurationDelta = $derived(
		workoutStats?.duration_trend !== undefined ? Math.round(workoutStats.duration_trend) : undefined
	);
	let last30Workouts = $derived(progressOverview?.last_30_days.workouts ?? 0);
	// Consistency: days trained in last 30. We don't have that directly so approximate
	// as last_30_days.workouts (assumes one session = one day, close enough).
	let consistencyDone = $derived(last30Workouts);
	let sessionSamples = $derived(workoutStats?.session_start_samples ?? []);
</script>

{#if desktop}
	<ProgressDesktop
		{consistencyDone}
		totalWorkouts={workoutStats?.total_workouts ?? 0}
		{perWeek}
		perWeekDelta={perWeekTrend}
		{avgDuration}
		{avgDurationDelta}
		{workoutStats}
		{progressOverview}
		{volumeStats}
		{exerciseProgress}
		{exerciseTypes}
		bind:selectedExercise
		{loadedExercise}
		{loadingOverall}
		{loadingExercise}
		{errorOverall}
		{errorExercise}
		onRefresh={refreshAll}
		{onSelectExercise}
	/>
{:else}
	<div class="page">
		<ProgressHero
			{consistencyDone}
			consistencyWindow={30}
			totalWorkouts={workoutStats?.total_workouts ?? 0}
			{perWeek}
			perWeekDelta={perWeekTrend}
			avgDurationMin={avgDuration}
			{avgDurationDelta}
			focusExercise={selectedExercise || undefined}
			loading={loadingOverall}
			error={errorOverall}
			onRefresh={refreshAll}
		/>

		<SegmentedTabs
			items={[
				{ id: 'overview' as const, label: 'Overview' },
				{ id: 'exercise' as const, label: 'Exercise' },
				{ id: 'trends' as const, label: 'Trends' }
			]}
			bind:selected={selectedTab}
			onselect={onTabSelect}
			aria-label="Progress sections"
		/>

		{#if selectedTab === 'overview'}
			{#if sessionSamples.length > 0}
				<SessionHeatmap samples={sessionSamples} />
			{/if}
			{#if errorOverall}
				<Card><div class="err">{errorOverall}</div></Card>
			{:else if loadingOverall && !progressOverview}
				<Card>
					{#snippet title()}Current progress{/snippet}
					<div class="skel"></div>
				</Card>
			{:else if progressOverview}
				<Card>
					{#snippet title()}Current progress <em>— vs previous matching period</em>{/snippet}
					{#snippet lede()}Recent work compared to the matching window before it.{/snippet}
					<PeriodCard period={progressOverview.last_7_days} variant="primary" />
					<PeriodCard period={progressOverview.last_30_days} variant="secondary" />
				</Card>

				<RecordsFeed
					prs={progressOverview.recent_prs}
					recentBests={progressOverview.recent_bests}
				/>
			{:else}
				<Card><div class="empty">No progress summary yet.</div></Card>
			{/if}
		{:else if selectedTab === 'exercise'}
			<ExerciseFocusPanel
				bind:selectedExercise
				{exerciseTypes}
				{volumeStats}
				{loadingExercise}
				{errorExercise}
				onSelect={onSelectExercise}
			/>
			{#if volumeStats && loadedExercise === selectedExercise}
				<ExerciseCharts {volumeStats} {exerciseProgress} />
				<RecentExerciseSessions {exerciseProgress} />
			{/if}
		{:else}
			<OverallCharts {workoutStats} />
		{/if}
	</div>
{/if}

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.skel {
		height: 60px;
		border-radius: 12px;
		background: var(--card-3);
	}
	.empty {
		font:
			500 13px/1.4 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
		text-align: center;
	}
	.err {
		font:
			600 13px/1.4 'Onest',
			system-ui,
			sans-serif;
		color: var(--clay-text);
	}
</style>
