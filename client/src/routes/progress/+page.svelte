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
	import ProgressHeader from '$lib/components/progress/ProgressHeader.svelte';
	import ProgressTabs, { type ProgressTab } from '$lib/components/progress/ProgressTabs.svelte';
	import OverviewTab from '$lib/components/progress/OverviewTab.svelte';
	import ExerciseTab from '$lib/components/progress/ExerciseTab.svelte';
	import TrendsTab from '$lib/components/progress/TrendsTab.svelte';

	let selectedTab: ProgressTab = 'overview';
	let selectedExercise = '';
	let loadedExercise = '';
	let requestedExercise = '';
	let exerciseTypes: string[] = [];
	let workoutStats: WorkoutStats | null = null;
	let progressOverview: ProgressOverview | null = null;
	let volumeStats: VolumeStats | null = null;
	let exerciseProgress: ExerciseProgress[] | null = null;

	let loadingOverall = false;
	let loadingExercise = false;
	let errorOverall: string | null = null;
	let errorExercise: string | null = null;

	let exerciseRequestId = 0;
	let exerciseTypesLoaded = false;

	function isProgressTab(value: string | null): value is ProgressTab {
		return value === 'overview' || value === 'exercise' || value === 'trends';
	}

	function getErrorMessage(e: unknown): string {
		if (e instanceof Error) return e.message;
		return 'Something went wrong';
	}

	async function loadOverall() {
		loadingOverall = true;
		errorOverall = null;
		try {
			const [nextWorkoutStats, nextProgressOverview] = await Promise.all([
				getWorkoutStats(),
				getProgressOverview()
			]);
			workoutStats = nextWorkoutStats;
			progressOverview = nextProgressOverview;
		} catch (e) {
			errorOverall = getErrorMessage(e);
			progressOverview = null;
			logger.error('progress', 'Error loading workout stats', { error: e });
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
			selectedExercise = '';
			loadedExercise = '';
			requestedExercise = '';
			volumeStats = null;
			exerciseProgress = null;
			exerciseTypesLoaded = true;
			errorExercise = getErrorMessage(e);
			logger.error('progress', 'Error loading exercise types', { error: e });
			return '';
		}
	}

	async function loadExercise(exercise = selectedExercise) {
		const requestId = ++exerciseRequestId;
		requestedExercise = exercise;

		if (!exercise) {
			loadedExercise = '';
			requestedExercise = '';
			volumeStats = null;
			exerciseProgress = null;
			loadingExercise = false;
			return;
		}

		loadingExercise = true;
		errorExercise = null;
		loadedExercise = '';
		volumeStats = null;
		exerciseProgress = null;

		try {
			localStorage.setItem('progress.selectedExercise', exercise);
			const [nextVolumeStats, nextExerciseProgress] = await Promise.all([
				getVolumeStats(exercise),
				getExerciseProgress(exercise)
			]);

			if (requestId !== exerciseRequestId || exercise !== selectedExercise) return;

			volumeStats = nextVolumeStats;
			exerciseProgress = nextExerciseProgress;
			loadedExercise = exercise;
		} catch (e) {
			if (requestId !== exerciseRequestId) return;
			errorExercise = getErrorMessage(e);
			loadedExercise = '';
			volumeStats = null;
			exerciseProgress = null;
			logger.error('progress', 'Error loading exercise data', {
				error: e,
				selectedExercise: exercise
			});
		} finally {
			if (requestId === exerciseRequestId) {
				loadingExercise = false;
				requestedExercise = '';
			}
		}
	}

	async function refreshAll() {
		const [, exercise] = await Promise.all([loadOverall(), loadExerciseTypes()]);
		if (selectedTab === 'exercise' && exercise) await loadExercise(exercise);
	}

	function selectExercise(event: CustomEvent<string>) {
		const exercise = event.detail;
		selectedExercise = exercise;
		if (exercise && exercise !== loadedExercise && exercise !== requestedExercise) {
			void loadExercise(exercise);
		}
	}

	async function ensureExerciseTabLoaded() {
		const exercise = exerciseTypesLoaded ? selectedExercise : await loadExerciseTypes();
		if (exercise && exercise !== loadedExercise && exercise !== requestedExercise) {
			await loadExercise(exercise);
		}
	}

	function selectTab(event: CustomEvent<ProgressTab>) {
		const tab = event.detail;
		selectedTab = tab;
		localStorage.setItem('progress.selectedTab', tab);
		if (tab === 'exercise') void ensureExerciseTabLoaded();
	}

	onMount(async () => {
		const storedTab = localStorage.getItem('progress.selectedTab');
		if (isProgressTab(storedTab)) selectedTab = storedTab;
		await refreshAll();
		if (selectedTab === 'exercise') await ensureExerciseTabLoaded();
	});
</script>

<div class="space-y-6">
	<ProgressHeader
		{workoutStats}
		{selectedExercise}
		{loadingOverall}
		{loadingExercise}
		{errorOverall}
		{errorExercise}
		on:refresh={refreshAll}
	/>

	<ProgressTabs {selectedTab} on:select={selectTab} />

	{#if selectedTab === 'overview'}
		<OverviewTab {progressOverview} {loadingOverall} {errorOverall} />
	{:else if selectedTab === 'exercise'}
		<ExerciseTab
			bind:selectedExercise
			{loadedExercise}
			{exerciseTypes}
			{volumeStats}
			{exerciseProgress}
			{loadingExercise}
			{errorExercise}
			on:select={selectExercise}
		/>
	{:else}
		<TrendsTab {workoutStats} />
	{/if}
</div>
