<script lang="ts">
	import { onMount } from 'svelte';
	import { createWorkout, createExercise, createSet, getWorkouts, cancelWorkout } from '$lib/api';
	import type { Workout, Exercise, Set } from '$lib/types';
	import { logger } from '$lib/logger';

	export let data: { workouts: Workout[] };
	let workouts = data.workouts;
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

	async function refreshWorkouts() {
		try {
			loading = true;
			workouts = await getWorkouts();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load workouts';
		} finally {
			loading = false;
		}
	}

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
			await refreshWorkouts();
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
			const exerciseData = {
				exercise_type: newExercise.exercise_type,
				notes: newExercise.notes || undefined
			};
			const result = await createExercise(currentWorkoutId, exerciseData);
			currentExerciseId = result.id;
			showExerciseForm = false;
			showSetForm = true;
			// Reset form
			newExercise = {
				exercise_type: '',
				notes: ''
			};
			await refreshWorkouts();
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
			await refreshWorkouts();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to create set';
		} finally {
			loading = false;
		}
	}

	async function handleDeleteWorkout(workoutId: number | undefined) {
		if (!workoutId) {
			error = 'Invalid workout ID';
			return;
		}

		if (!confirm('Are you sure you want to delete this workout? This action cannot be undone.')) {
			return;
		}

		try {
			loading = true;
			error = null;
			await cancelWorkout(workoutId);
			logger.info('workout', 'Workout deleted', { workoutId });
			await refreshWorkouts();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to delete workout';
			logger.error('workout', 'Failed to delete workout', { error });
		} finally {
			loading = false;
		}
	}

	function formatDateRelative(dateString: string): string {
		const date = new Date(dateString);
		const now = new Date();
		const days = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];
		const months = ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December'];

		// Function to add ordinal suffix
		const getOrdinal = (n: number) => {
			const s = ['th', 'st', 'nd', 'rd'];
			const v = n % 100;
			return n + (s[(v - 20) % 10] || s[v] || s[0]);
		};

		// Check if it's today
		if (date.toDateString() === now.toDateString()) {
			return 'Today';
		}

		// Check if it's yesterday
		const yesterday = new Date(now);
		yesterday.setDate(yesterday.getDate() - 1);
		if (date.toDateString() === yesterday.toDateString()) {
			return 'Yesterday';
		}

		// Check if it's within the last week
		const lastWeek = new Date(now);
		lastWeek.setDate(lastWeek.getDate() - 7);
		if (date > lastWeek) {
			return `Last ${days[date.getDay()]}`;
		}

		// Otherwise, return the full date
		return `${days[date.getDay()]}, ${getOrdinal(date.getDate())} of ${months[date.getMonth()]}`;
	}

	function formatTime(dateString: string): string {
		return new Date(dateString).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
	}

	onMount(refreshWorkouts);
</script>

<style lang="postcss">
	.workout-list {
		@apply grid gap-4;
	}
	.workout-card {
		@apply card p-4 transition-transform hover:scale-[1.01] bg-surface-900/50;
		border: 1px solid #E9A737;
	}
	.workout-header {
		@apply flex justify-between items-center mb-2;
	}
	.workout-date {
		@apply text-2xl font-semibold;
	}
	.workout-time {
		@apply text-base opacity-90;
	}
	.workout-notes {
		@apply text-base opacity-90 mb-4;
	}
	.workout-actions {
		@apply flex justify-end gap-2 mt-2;
	}
	.view-details-btn {
		@apply btn variant-filled-primary;
	}
	:global(.dark) .workout-card {
		@apply bg-surface-900/50;
		border: 1px solid #E9A737;
	}
	:global(.dark) .view-details-btn {
		@apply bg-primary-500/80 hover:bg-primary-500;
	}
</style>

<div class="container mx-auto p-4 space-y-6">
	<header class="text-center">
		<h2 class="h2 mb-4">Workout History</h2>
	</header>

	{#if error}
		<div class="alert variant-filled-error">
			{error}
		</div>
	{/if}

	<div class="workout-list">
		{#if loading}
			<div class="card p-4 text-center">
				<span class="loading">Loading workouts...</span>
			</div>
		{:else if workouts.length === 0}
			<div class="card variant-ghost p-4 text-center">
				<p>No workouts yet. Start your fitness journey today!</p>
			</div>
		{:else}
			{#each workouts as workout}
				<div class="workout-card">
					<div class="workout-header">
						<div>
							<div class="workout-date">{formatDateRelative(workout.date)}</div>
							<div class="workout-time">
								{formatTime(workout.start_time)} - {formatTime(workout.end_time)}
							</div>
						</div>
						<div class="flex items-center gap-2">
							{#if workout.feedback}
								<span class="text-3xl">{workout.feedback}</span>
							{/if}
						</div>
					</div>
					
					{#if workout.notes}
						<div class="workout-notes">{workout.notes}</div>
					{/if}

					<div class="workout-actions">
						<button 
							class="btn variant-filled-error"
							on:click={() => handleDeleteWorkout(workout.id)}
							disabled={loading}
						>
							<span class="text-xl mr-2">🗑️</span>
							Delete
						</button>
						<a href="/workouts/{workout.id}" class="view-details-btn">
							View Details →
						</a>
					</div>
				</div>
			{/each}
		{/if}
	</div>
</div> 