<script lang="ts">
    import { onMount } from 'svelte';
    import { page } from '$app/stores';
    import { getWorkout } from '$lib/api';
    import type { WorkoutWithExercises } from '$lib/types';

    let workout: WorkoutWithExercises | null = null;
    let loading = true;
    let error: string | null = null;

    onMount(async () => {
        try {
            const workoutId = Number($page.params.id);
            const data = await getWorkout(workoutId);
            workout = {
                ...data.workout,
                exercises: data.exercises
            };
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to load workout';
        } finally {
            loading = false;
        }
    });

    function formatDate(dateString: string): string {
        return new Date(dateString).toLocaleDateString();
    }
</script>

<div class="space-y-8">
    <header class="flex justify-between items-center">
        <h2 class="h2">Workout Details</h2>
        <a href="/workouts" class="btn variant-soft">Back to Workouts</a>
    </header>

    {#if error}
        <div class="alert variant-filled-error">
            {error}
        </div>
    {:else if loading}
        <div class="card p-4">
            <p class="text-center">Loading workout details...</p>
        </div>
    {:else if workout}
        <div class="card p-4 space-y-6">
            <div>
                <h3 class="h3">Workout Information</h3>
                <p class="text-lg">Date: {formatDate(workout.date)}</p>
                {#if workout.notes}
                    <p class="text-lg">Notes: {workout.notes}</p>
                {/if}
            </div>

            <div>
                <h3 class="h3 mb-4">Exercises</h3>
                {#if workout.exercises.length === 0}
                    <p>No exercises recorded for this workout.</p>
                {:else}
                    {#each workout.exercises as { exercise, sets }}
                        <div class="card variant-soft p-4 mb-4">
                            <h4 class="h4">{exercise.exercise_type}</h4>
                            {#if exercise.notes}
                                <p class="text-sm mb-2">Notes: {exercise.notes}</p>
                            {/if}
                            
                            {#if sets.length > 0}
                                <table class="table table-compact">
                                    <thead>
                                        <tr>
                                            <th>Set</th>
                                            <th>Reps</th>
                                            <th>Weight</th>
                                            <th>Notes</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {#each sets as set, i}
                                            <tr>
                                                <td>{i + 1}</td>
                                                <td>{set.reps}</td>
                                                <td>{set.weight} kg</td>
                                                <td>{set.notes || '-'}</td>
                                            </tr>
                                        {/each}
                                    </tbody>
                                </table>
                            {:else}
                                <p class="text-sm">No sets recorded for this exercise.</p>
                            {/if}
                        </div>
                    {/each}
                {/if}
            </div>
        </div>
    {/if}
</div> 