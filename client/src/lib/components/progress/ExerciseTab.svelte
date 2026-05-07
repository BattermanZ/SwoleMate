<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import type { ExerciseProgress, VolumeStats } from '$lib/types';
	import ExerciseFocusPanel from '$lib/components/progress/ExerciseFocusPanel.svelte';
	import ExerciseCharts from '$lib/components/progress/ExerciseCharts.svelte';
	import RecentExerciseSessions from '$lib/components/progress/RecentExerciseSessions.svelte';

	export let selectedExercise = '';
	export let loadedExercise = '';
	export let exerciseTypes: string[] = [];
	export let volumeStats: VolumeStats | null = null;
	export let exerciseProgress: ExerciseProgress[] | null = null;
	export let loadingExercise = false;
	export let errorExercise: string | null = null;

	const dispatch = createEventDispatcher<{ select: string }>();
</script>

<div class="space-y-4">
	<ExerciseFocusPanel
		bind:selectedExercise
		{exerciseTypes}
		{volumeStats}
		{loadingExercise}
		{errorExercise}
		on:select={(event) => dispatch('select', event.detail)}
	/>

	{#if volumeStats && loadedExercise === selectedExercise}
		<ExerciseCharts {volumeStats} {exerciseProgress} />
		<RecentExerciseSessions {exerciseProgress} />
	{/if}
</div>
