<script lang="ts">
	import { onMount } from 'svelte';
	import { getExerciseProgress, getExerciseTypes, getVolumeStats, getWorkoutStats } from '$lib/api';
	import type { ExerciseProgress, VolumeStats, WorkoutStats } from '$lib/types';
	import { logger } from '$lib/logger';
	import ProgressHeader from '$lib/components/progress/ProgressHeader.svelte';
	import ExerciseFocusPanel from '$lib/components/progress/ExerciseFocusPanel.svelte';
	import ExerciseCharts from '$lib/components/progress/ExerciseCharts.svelte';
	import RecentExerciseSessions from '$lib/components/progress/RecentExerciseSessions.svelte';
	import OverallCharts from '$lib/components/progress/OverallCharts.svelte';

	let selectedExercise = '';
	let loadedExercise = '';
	let requestedExercise = '';
	let exerciseTypes: string[] = [];
	let workoutStats: WorkoutStats | null = null;
	let volumeStats: VolumeStats | null = null;
	let exerciseProgress: ExerciseProgress[] | null = null;

	let loadingOverall = false;
	let loadingExercise = false;
	let errorOverall: string | null = null;
	let errorExercise: string | null = null;

	let exerciseRequestId = 0;
	let exerciseWatcherEnabled = false;

	function getErrorMessage(e: unknown): string {
		if (e instanceof Error) return e.message;
		return 'Something went wrong';
	}

	async function loadOverall() {
		loadingOverall = true;
		errorOverall = null;
		try {
			workoutStats = await getWorkoutStats();
		} catch (e) {
			errorOverall = getErrorMessage(e);
			logger.error('progress', 'Error loading workout stats', { error: e });
		} finally {
			loadingOverall = false;
		}
	}

	async function loadExerciseTypes(): Promise<string> {
		try {
			exerciseTypes = await getExerciseTypes();
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
			logger.error('progress', 'Error loading exercise data', { error: e, selectedExercise: exercise });
		} finally {
			if (requestId === exerciseRequestId) {
				loadingExercise = false;
				requestedExercise = '';
			}
		}
	}

	async function refreshAll() {
		exerciseWatcherEnabled = false;
		const [, exercise] = await Promise.all([loadOverall(), loadExerciseTypes()]);
		if (exercise) await loadExercise(exercise);
		exerciseWatcherEnabled = true;
	}

	onMount(async () => {
		await refreshAll();
	});

	$: if (
		exerciseWatcherEnabled &&
		selectedExercise &&
		selectedExercise !== loadedExercise &&
		selectedExercise !== requestedExercise
	) {
		void loadExercise(selectedExercise);
	}
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

	<div class="grid gap-6 md:grid-cols-12">
		<section class="md:col-span-7 lg:col-span-8 space-y-4 min-w-0">
			<ExerciseFocusPanel
				bind:selectedExercise
				{exerciseTypes}
				{volumeStats}
				{loadingExercise}
				{errorExercise}
			/>

			{#if volumeStats && loadedExercise === selectedExercise}
				<ExerciseCharts {volumeStats} {exerciseProgress} />
				<RecentExerciseSessions {exerciseProgress} />
			{/if}
		</section>

		<aside class="md:col-span-5 lg:col-span-4 min-w-0">
			<OverallCharts {workoutStats} />
		</aside>
	</div>
</div>
