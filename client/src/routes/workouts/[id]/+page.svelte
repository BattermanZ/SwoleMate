<script lang="ts">
	import { goto } from '$app/navigation';
	import EditWorkoutTimesModal from '$lib/components/history/EditWorkoutTimesModal.svelte';
	import { auth } from '$lib/auth';
	import {
		cancelWorkout,
		createWorkoutTemplateFromWorkout,
		getWorkout,
		updateWorkoutTimes
	} from '$lib/api';
	import type { WorkoutWithExercises } from '$lib/types';
	import WorkoutDetailsCard from '$lib/components/history/WorkoutDetailsCard.svelte';

	export let data: { workout: WorkoutWithExercises | null; error: string | null };

	let workout = data.workout;
	let error = data.error;
	let editTimesOpen = false;
	let editTimesError: string | null = null;
	let editTimesSaving = false;
	let deleting = false;
	let savingTemplate = false;
	const authState = auth.state;

	function openEditTimes() {
		editTimesError = null;
		editTimesOpen = true;
	}

	async function refreshWorkout(workoutId: number) {
		const loaded = await getWorkout(workoutId);
		workout = { ...loaded.workout, exercises: loaded.exercises };
	}

	async function handleSaveTimes(
		e: CustomEvent<{
			start_time: string;
			end_time: string;
			notes: string | null;
			feedback: '😊' | '😐' | '😞' | null;
		}>
	) {
		if (!workout || typeof workout.id !== 'number') return;
		const workoutId: number = workout.id;
		editTimesSaving = true;
		editTimesError = null;
		try {
			await updateWorkoutTimes(workoutId, {
				start_time: e.detail.start_time,
				end_time: e.detail.end_time,
				notes: e.detail.notes,
				feedback: e.detail.feedback
			});
			await refreshWorkout(workoutId);
			editTimesOpen = false;
		} catch (err) {
			editTimesError = err instanceof Error ? err.message : 'Failed to update times';
		} finally {
			editTimesSaving = false;
		}
	}

	async function handleDeleteWorkout() {
		if (!workout || typeof workout.id !== 'number') {
			error = 'Invalid workout ID';
			return;
		}
		const workoutId: number = workout.id;
		if ($authState.offline) {
			error = 'Offline mode: delete workouts when you are back online.';
			return;
		}
		if (!confirm('Are you sure you want to delete this workout? This action cannot be undone.')) {
			return;
		}

		deleting = true;
		error = null;
		try {
			await cancelWorkout(workoutId);
			await goto('/workouts');
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to delete workout';
		} finally {
			deleting = false;
		}
	}

	async function handleSaveAsTemplate() {
		if (!workout || typeof workout.id !== 'number') {
			error = 'Invalid workout ID';
			return;
		}
		if ($authState.offline) {
			error = 'Offline mode: templates are available when you are back online.';
			return;
		}

		const defaultName =
			workout.exercises.length > 0
				? `${workout.exercises[0].exercise.exercise_type} template`
				: 'Workout template';
		const name = prompt('Template name:', defaultName)?.trim();
		if (!name) return;

		savingTemplate = true;
		error = null;
		try {
			const created = await createWorkoutTemplateFromWorkout(workout.id, { name });
			await goto(`/templates?template=${created.template.id}`);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to save template';
		} finally {
			savingTemplate = false;
		}
	}
</script>

<div class="space-y-6">
	<header
		class="relative overflow-hidden rounded-2xl border border-surface-200/50 dark:border-surface-700/50 bg-gradient-to-br from-primary-500/10 via-transparent to-tertiary-500/10 p-5 sm:p-6"
	>
		<div
			class="pointer-events-none absolute -top-24 -right-24 size-72 rounded-full blur-3xl bg-primary-500/15"
		></div>
		<div
			class="pointer-events-none absolute -bottom-24 -left-24 size-72 rounded-full blur-3xl bg-secondary-500/15"
		></div>

		<div class="relative flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
			<div class="space-y-1">
				<h1 class="text-3xl sm:text-4xl font-black tracking-tight">Workout</h1>
				<p class="text-sm sm:text-base opacity-80 max-w-prose">Full session details.</p>
			</div>
			<div class="flex sm:justify-end">
				<a href="/workouts" class="btn variant-soft">← Back to History</a>
			</div>
		</div>
	</header>

	<WorkoutDetailsCard {workout} loading={!workout && !error} {error}>
		<svelte:fragment slot="actions">
			{#if workout}
				<button
					type="button"
					class="btn btn-sm variant-soft"
					on:click={handleSaveAsTemplate}
					disabled={editTimesSaving || deleting || savingTemplate}
				>
					Save as template
				</button>
				<button
					type="button"
					class="btn btn-sm variant-soft"
					on:click={openEditTimes}
					disabled={editTimesSaving || deleting || savingTemplate}
				>
					Edit times
				</button>
				<button
					type="button"
					class="btn btn-sm variant-soft-error"
					on:click={handleDeleteWorkout}
					disabled={deleting || editTimesSaving || savingTemplate}
				>
					Delete
				</button>
			{/if}
		</svelte:fragment>
	</WorkoutDetailsCard>

	<EditWorkoutTimesModal
		open={editTimesOpen}
		{workout}
		disabled={editTimesSaving}
		error={editTimesError}
		on:cancel={() => (editTimesOpen = false)}
		on:submit={handleSaveTimes}
	/>
</div>
