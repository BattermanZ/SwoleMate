<script lang="ts">
	import { onMount } from 'svelte';
	import { createWorkout, createExercise, createSet } from '$lib/api';
	import type { Workout, Exercise, Set } from '$lib/types';

	let workouts: Workout[] = [];
	let loading = false;
	let error: string | null = null;

	// Form states
	let showWorkoutForm = false;
	let showExerciseForm = false;
	let showSetForm = false;
	let currentWorkoutId: number | null = null;
	let currentExerciseId: number | null = null;

	// Form data
	let newWorkout = {
		date: new Date().toISOString().split('T')[0],
		notes: ''
	};

	let newExercise = {
		exercise_type: '',
		notes: ''
	};

	let newSet = {
		reps: 0,
		weight: 0,
		notes: ''
	};

	async function handleCreateWorkout() {
		try {
			loading = true;
			error = null;
			const result = await createWorkout({
				date: new Date(newWorkout.date).toISOString(),
				notes: newWorkout.notes || undefined
			});
			currentWorkoutId = result.id;
			showWorkoutForm = false;
			showExerciseForm = true;
			// Reset form
			newWorkout = {
				date: new Date().toISOString().split('T')[0],
				notes: ''
			};
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to create workout';
		} finally {
			loading = false;
		}
	}

	async function handleCreateExercise() {
		if (!currentWorkoutId) return;
		try {
			loading = true;
			error = null;
			const result = await createExercise(currentWorkoutId, newExercise);
			currentExerciseId = result.id;
			showExerciseForm = false;
			showSetForm = true;
			// Reset form
			newExercise = {
				exercise_type: '',
				notes: ''
			};
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to create exercise';
		} finally {
			loading = false;
		}
	}

	async function handleCreateSet() {
		if (!currentExerciseId) return;
		try {
			loading = true;
			error = null;
			await createSet(currentExerciseId, newSet);
			showSetForm = false;
			// Reset form
			newSet = {
				reps: 0,
				weight: 0,
				notes: ''
			};
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to create set';
		} finally {
			loading = false;
		}
	}
</script>

<div class="space-y-8">
	<header class="flex justify-between items-center">
		<h2 class="h2">Workouts</h2>
		<button class="btn variant-filled-primary" on:click={() => showWorkoutForm = true}>
			New Workout
		</button>
	</header>

	{#if error}
		<div class="alert variant-filled-error">
			{error}
		</div>
	{/if}

	{#if showWorkoutForm}
		<div class="card p-4">
			<h3 class="h3 mb-4">New Workout</h3>
			<form on:submit|preventDefault={handleCreateWorkout} class="space-y-4">
				<label class="label">
					<span>Date</span>
					<input
						type="date"
						class="input"
						bind:value={newWorkout.date}
						required
					/>
				</label>
				<label class="label">
					<span>Notes</span>
					<textarea
						class="textarea"
						bind:value={newWorkout.notes}
						rows="3"
					></textarea>
				</label>
				<div class="flex justify-end space-x-2">
					<button type="button" class="btn variant-soft" on:click={() => showWorkoutForm = false}>
						Cancel
					</button>
					<button type="submit" class="btn variant-filled-primary" disabled={loading}>
						{loading ? 'Creating...' : 'Create Workout'}
					</button>
				</div>
			</form>
		</div>
	{/if}

	{#if showExerciseForm}
		<div class="card p-4">
			<h3 class="h3 mb-4">Add Exercise</h3>
			<form on:submit|preventDefault={handleCreateExercise} class="space-y-4">
				<label class="label">
					<span>Exercise Type</span>
					<input
						type="text"
						class="input"
						bind:value={newExercise.exercise_type}
						required
					/>
				</label>
				<label class="label">
					<span>Notes</span>
					<textarea
						class="textarea"
						bind:value={newExercise.notes}
						rows="3"
					></textarea>
				</label>
				<div class="flex justify-end space-x-2">
					<button type="button" class="btn variant-soft" on:click={() => showExerciseForm = false}>
						Cancel
					</button>
					<button type="submit" class="btn variant-filled-primary" disabled={loading}>
						{loading ? 'Adding...' : 'Add Exercise'}
					</button>
				</div>
			</form>
		</div>
	{/if}

	{#if showSetForm}
		<div class="card p-4">
			<h3 class="h3 mb-4">Add Set</h3>
			<form on:submit|preventDefault={handleCreateSet} class="space-y-4">
				<label class="label">
					<span>Reps</span>
					<input
						type="number"
						class="input"
						bind:value={newSet.reps}
						min="0"
						required
					/>
				</label>
				<label class="label">
					<span>Weight (lbs)</span>
					<input
						type="number"
						class="input"
						bind:value={newSet.weight}
						min="0"
						step="0.5"
						required
					/>
				</label>
				<label class="label">
					<span>Notes</span>
					<textarea
						class="textarea"
						bind:value={newSet.notes}
						rows="3"
					></textarea>
				</label>
				<div class="flex justify-end space-x-2">
					<button type="button" class="btn variant-soft" on:click={() => showSetForm = false}>
						Cancel
					</button>
					<button type="submit" class="btn variant-filled-primary" disabled={loading}>
						{loading ? 'Adding...' : 'Add Set'}
					</button>
				</div>
			</form>
		</div>
	{/if}

	<div class="card p-4">
		<div class="table-container">
			<table class="table table-hover">
				<thead>
					<tr>
						<th>Date</th>
						<th>Exercise</th>
						<th>Sets</th>
						<th>Reps</th>
						<th>Weight</th>
						<th>Notes</th>
					</tr>
				</thead>
				<tbody>
					{#if loading}
						<tr>
							<td colspan="6" class="text-center">Loading...</td>
						</tr>
					{:else if workouts.length === 0}
						<tr>
							<td colspan="6" class="text-center">No workouts yet. Create one to get started!</td>
						</tr>
					{/if}
				</tbody>
			</table>
		</div>
	</div>
</div> 