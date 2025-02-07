# Progress Page

<script lang="ts">
    import { onMount } from 'svelte';
    import { Chart, registerables } from 'chart.js';
    import 'chartjs-adapter-date-fns';
    import { getWorkoutStats, getExerciseTypes, getVolumeStats, getExerciseProgress } from '$lib/api';
    import type { WorkoutStats, VolumeStats, ExerciseProgress } from '$lib/types';

    Chart.register(...registerables);

    let selectedExercise: string = '';
    let exerciseTypes: string[] = [];
    let workoutStats: WorkoutStats | null = null;
    let volumeStats: VolumeStats | null = null;
    let exerciseProgress: ExerciseProgress[] | null = null;
    let volumeChart: Chart | null = null;
    let progressChart: Chart | null = null;
    let feedbackChart: Chart | null = null;

    onMount(async () => {
        try {
            exerciseTypes = await getExerciseTypes();
            workoutStats = await getWorkoutStats();
            if (exerciseTypes.length > 0) {
                selectedExercise = exerciseTypes[0];
                await loadExerciseData();
            }
            createFeedbackChart();
        } catch (error) {
            console.error('Error loading initial data:', error);
        }
    });

    async function loadExerciseData() {
        try {
            volumeStats = await getVolumeStats(selectedExercise);
            exerciseProgress = await getExerciseProgress(selectedExercise);
            createVolumeChart();
            createProgressChart();
        } catch (error) {
            console.error('Error loading exercise data:', error);
        }
    }

    function createVolumeChart() {
        if (!volumeStats) return;

        const ctx = document.getElementById('volumeChart') as HTMLCanvasElement;
        if (!ctx) return;

        if (volumeChart) volumeChart.destroy();

        volumeChart = new Chart(ctx, {
            type: 'line',
            data: {
                labels: volumeStats.weekly_volume.map(v => v.week),
                datasets: [
                    {
                        label: 'Total Volume (kg)',
                        data: volumeStats.weekly_volume.map(v => v.total_volume),
                        borderColor: 'rgb(75, 192, 192)',
                        tension: 0.1
                    },
                    {
                        label: 'Max Weight (kg)',
                        data: volumeStats.weekly_volume.map(v => v.max_weight),
                        borderColor: 'rgb(255, 99, 132)',
                        tension: 0.1
                    }
                ]
            },
            options: {
                responsive: true,
                scales: {
                    y: {
                        beginAtZero: true
                    }
                }
            }
        });
    }

    function createProgressChart() {
        if (!exerciseProgress) return;

        const ctx = document.getElementById('progressChart') as HTMLCanvasElement;
        if (!ctx) return;

        if (progressChart) progressChart.destroy();

        const data = exerciseProgress.map(ep => ({
            x: new Date(ep.exercise.start_time),
            y: Math.max(...ep.sets.map(s => s.weight))
        }));

        progressChart = new Chart(ctx, {
            type: 'scatter',
            data: {
                datasets: [{
                    label: 'Max Weight per Session',
                    data: data,
                    backgroundColor: 'rgb(75, 192, 192)'
                }]
            },
            options: {
                responsive: true,
                scales: {
                    x: {
                        type: 'time',
                        time: {
                            unit: 'day'
                        }
                    },
                    y: {
                        beginAtZero: true,
                        title: {
                            display: true,
                            text: 'Weight (kg)'
                        }
                    }
                }
            }
        });
    }

    function createFeedbackChart() {
        if (!workoutStats) return;

        const ctx = document.getElementById('feedbackChart') as HTMLCanvasElement;
        if (!ctx) return;

        if (feedbackChart) feedbackChart.destroy();

        feedbackChart = new Chart(ctx, {
            type: 'doughnut',
            data: {
                labels: ['Good', 'Neutral', 'Bad'],
                datasets: [{
                    data: [
                        workoutStats.feedback_distribution.good,
                        workoutStats.feedback_distribution.neutral,
                        workoutStats.feedback_distribution.bad
                    ],
                    backgroundColor: [
                        'rgb(75, 192, 192)',
                        'rgb(255, 205, 86)',
                        'rgb(255, 99, 132)'
                    ]
                }]
            },
            options: {
                responsive: true
            }
        });
    }

    $: if (selectedExercise) {
        loadExerciseData();
    }
</script>

<div class="container mx-auto p-4">
    <h1 class="h1 mb-4">Progress</h1>

    <!-- Overall Stats -->
    {#if workoutStats}
        <div class="card p-4 mb-8">
            <h2 class="h2 mb-4">Overall Statistics</h2>
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div class="card variant-soft p-4">
                    <h3 class="h3">Total Workouts</h3>
                    <p class="text-4xl font-bold">{workoutStats.total_workouts}</p>
                </div>
                <div class="card variant-soft p-4">
                    <h3 class="h3">Average Duration</h3>
                    <p class="text-4xl font-bold">{Math.round(workoutStats.average_duration_minutes)} min</p>
                </div>
                <div class="card variant-soft p-4">
                    <h3 class="h3">Workout Feedback</h3>
                    <canvas id="feedbackChart"></canvas>
                </div>
            </div>
        </div>
    {/if}

    <!-- Exercise Progress -->
    <div class="card p-4 mb-8">
        <h2 class="h2 mb-4">Exercise Progress</h2>
        <select
            class="select mb-4"
            bind:value={selectedExercise}
        >
            {#each exerciseTypes as type}
                <option value={type}>{type}</option>
            {/each}
        </select>

        {#if volumeStats}
            <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
                <div class="card variant-soft p-4">
                    <h3 class="h3 mb-4">Volume Progress</h3>
                    <canvas id="volumeChart"></canvas>
                </div>
                <div class="card variant-soft p-4">
                    <h3 class="h3 mb-4">Weight Progress</h3>
                    <canvas id="progressChart"></canvas>
                </div>
            </div>

            <!-- Personal Records -->
            <div class="mt-8">
                <h3 class="h3 mb-4">Personal Records</h3>
                <div class="table-container">
                    <table class="table table-hover">
                        <thead>
                            <tr>
                                <th>Reps</th>
                                <th>Weight (kg)</th>
                                <th>Date Achieved</th>
                            </tr>
                        </thead>
                        <tbody>
                            {#each volumeStats.personal_records as pr}
                                <tr>
                                    <td>{pr.reps}</td>
                                    <td>{pr.weight}</td>
                                    <td>{new Date(pr.achieved_at).toLocaleDateString()}</td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                </div>
            </div>
        {/if}
    </div>
</div>

<style>
    .table-container {
        overflow-x: auto;
    }
</style> 