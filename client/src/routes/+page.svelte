<!-- YOU CAN DELETE EVERYTHING IN THIS PAGE -->

<script lang="ts">
	import { createWorkout, createExercise, createSet } from '$lib/api';
	import type { Workout, Exercise, Set } from '$lib/types';
	import { ProgressRadial } from '@skeletonlabs/skeleton';

	let currentWorkout: Workout | null = null;
	let exercises: Array<{
		id?: number;
		name: string;
		sets: Array<{
			id?: number;
			reps: number;
			weight: number;
			isEditing: boolean;
			isConfirmed: boolean;
		}>;
		showSetForm: boolean;
	}> = [];
	let loading = false;
	let error: string | null = null;
	let showExerciseForm = false;
	let newExerciseName = '';

	async function startWorkout() {
		try {
			loading = true;
			error = null;
			const result = await createWorkout({
				date: new Date().toISOString(),
				notes: "Today's workout"
			});
			currentWorkout = {
				id: result.id,
				date: new Date().toISOString(),
				notes: "Today's workout"
			};
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to start workout';
		} finally {
			loading = false;
		}
	}

	async function addExercise() {
		if (!currentWorkout?.id) return;
		if (!newExerciseName.trim()) return;

		try {
			loading = true;
			error = null;
			const result = await createExercise(currentWorkout.id, {
				exercise_type: newExerciseName,
				notes: ''
			});
			exercises = [...exercises, {
				id: result.id,
				name: newExerciseName,
				sets: [],
				showSetForm: true
			}];
			newExerciseName = '';
			showExerciseForm = false;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to add exercise';
		} finally {
			loading = false;
		}
	}

	async function addSet(exerciseIndex: number) {
		const exercise = exercises[exerciseIndex];
		if (!exercise.id) return;

		exercises[exerciseIndex].sets = [...exercise.sets, {
			reps: 12,
			weight: 0,
			isEditing: true,
			isConfirmed: false
		}];
		exercises = [...exercises];
	}

	async function confirmSet(exerciseIndex: number, setIndex: number) {
		const exercise = exercises[exerciseIndex];
		const set = exercise.sets[setIndex];
		if (!exercise.id || !set) return;

		try {
			loading = true;
			error = null;
			const result = await createSet(exercise.id, {
				reps: set.reps,
				weight: set.weight,
				notes: ''
			});
			exercises[exerciseIndex].sets[setIndex] = {
				...set,
				id: result.id,
				isEditing: false,
				isConfirmed: true
			};
			exercises = [...exercises];
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to confirm set';
		} finally {
			loading = false;
		}
	}

	async function endWorkout() {
		currentWorkout = null;
		exercises = [];
		showExerciseForm = false;
	}
</script>

<style>
	/* Remove spinner arrows from number inputs */
	input[type="number"]::-webkit-inner-spin-button,
	input[type="number"]::-webkit-outer-spin-button {
		-webkit-appearance: none;
		margin: 0;
	}
	input[type="number"] {
		-moz-appearance: textfield;
	}
</style>

<div class="container mx-auto p-4 space-y-8">
	<header class="text-center space-y-4">
		<h1 class="h1">Today's Workout</h1>
		{#if !currentWorkout}
			<button class="btn variant-filled-primary w-full md:w-auto" on:click={startWorkout} disabled={loading}>
				{#if loading}
					<ProgressRadial width="w-6" stroke={150} meter="stroke-primary-500" track="stroke-primary-500/30"/>
				{:else}
					Start New Session
				{/if}
			</button>
		{/if}
	</header>

	{#if error}
		<div class="alert variant-filled-error">
			{error}
		</div>
	{/if}

	{#if currentWorkout}
		<div class="card p-4 space-y-4">
			<div class="flex justify-between items-center">
				<h2 class="h2">Current Session</h2>
				<button class="btn variant-soft-error" on:click={endWorkout}>End Session</button>
			</div>

			{#each exercises as exercise, exerciseIndex}
				<div class="card variant-soft p-4">
					<div class="flex flex-wrap gap-4 items-center">
						<span class="font-bold flex-grow">{exercise.name}</span>
						{#each exercise.sets as set, setIndex}
							{#if set.isEditing}
								<div class="flex gap-2 items-center">
									<span class="text-sm font-medium">Set {setIndex + 1}</span>
									<input
										type="number"
										inputmode="numeric"
										pattern="[0-9]*"
										class="input w-16 text-center"
										bind:value={set.reps}
										min="0"
									/>
									<span class="text-sm">×</span>
									<input
										type="number"
										inputmode="numeric"
										pattern="[0-9]*"
										class="input w-16 text-center"
										bind:value={set.weight}
										min="0"
										step="0.5"
									/>
									<span class="text-sm">kg</span>
									<button 
										class="btn variant-filled-success btn-sm"
										on:click={() => confirmSet(exerciseIndex, setIndex)}
										disabled={loading}
									>
										✓
									</button>
								</div>
							{:else}
								<div class="chip variant-filled">
									{set.reps}×{set.weight}kg
								</div>
							{/if}
						{/each}
						{#if !exercise.sets.some(s => s.isEditing)}
							<button 
								class="btn variant-filled-secondary btn-sm" 
								on:click={() => addSet(exerciseIndex)}
							>
								+
							</button>
						{/if}
					</div>
				</div>
			{/each}

			{#if showExerciseForm}
				<div class="card variant-ghost p-4">
					<form on:submit|preventDefault={addExercise} class="flex gap-2">
						<input
							type="text"
							class="input flex-grow"
							placeholder="Exercise name"
							bind:value={newExerciseName}
							required
						/>
						<button type="submit" class="btn variant-filled-primary">Add</button>
						<button type="button" class="btn variant-soft" on:click={() => showExerciseForm = false}>Cancel</button>
					</form>
				</div>
			{:else}
				<button class="btn variant-filled w-full" on:click={() => showExerciseForm = true}>
					Add Exercise
				</button>
			{/if}
		</div>
	{/if}
</div>
