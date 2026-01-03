<script lang="ts">
	import { getWorkouts, cancelWorkout } from '$lib/api';
	import type { Workout } from '$lib/types';
	import { logger } from '$lib/logger';
	import { formatDateRelative, formatTime } from '$lib/utils/date';

	export let data: { workouts: Workout[] };
	let workouts = data.workouts;
	let loading = false;
	let error: string | null = null;

	async function refreshWorkouts() {
		try {
			loading = true;
			error = null;
			workouts = await getWorkouts();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load workouts';
			logger.error('workout', 'Failed to load workouts', { error: e });
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
</script>

<div class="container mx-auto p-4 space-y-6">
	<header class="text-center">
		<h2 class="h2 mb-4">Workout History</h2>
	</header>

	{#if error}
		<div class="alert variant-filled-error">
			{error}
		</div>
	{/if}

	<div class="grid gap-4">
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
				<div
					class="card p-4 transition-transform hover:scale-[1.01] bg-surface-900/50 border border-[#e9a737]"
				>
					<div class="flex justify-between items-center mb-2">
						<div>
							<div class="text-2xl font-semibold">{formatDateRelative(workout.date)}</div>
							<div class="text-base opacity-90">
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
						<div class="text-base opacity-90 mb-4">{workout.notes}</div>
					{/if}

					<div class="flex justify-end gap-2 mt-2">
						<button
							class="btn variant-filled-error"
							on:click={() => handleDeleteWorkout(workout.id)}
							disabled={loading}
						>
							<span class="text-xl mr-2">🗑️</span>
							Delete
						</button>
						<a href="/workouts/{workout.id}" class="btn variant-filled-primary"> View Details → </a>
					</div>
				</div>
			{/each}
		{/if}
	</div>
</div>
