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
	let timeDistributionChart: Chart | null = null;
	let durationDistributionChart: Chart | null = null;
	let monthlyVolumeChart: Chart | null = null;

	onMount(async () => {
		try {
			exerciseTypes = await getExerciseTypes();
			workoutStats = await getWorkoutStats();
			if (exerciseTypes.length > 0) {
				selectedExercise = exerciseTypes[0];
			}
			createFeedbackChart();
			createTimeDistributionChart();
			createDurationDistributionChart();
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
			createMonthlyVolumeChart();
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
				labels: volumeStats.weekly_volume.map((v) => v.week),
				datasets: [
					{
						label: 'Total Volume (kg)',
						data: volumeStats.weekly_volume.map((v) => v.total_volume),
						borderColor: 'rgb(75, 192, 192)',
						tension: 0.1,
						yAxisID: 'y'
					},
					{
						label: 'Estimated 1RM (kg)',
						data: volumeStats.weekly_volume.map((v) => v.max_estimated_1rm),
						borderColor: 'rgb(255, 99, 132)',
						tension: 0.1,
						yAxisID: 'y1'
					}
				]
			},
			options: {
				responsive: true,
				scales: {
					y: {
						beginAtZero: true,
						position: 'left',
						title: {
							display: true,
							text: 'Volume (kg)'
						}
					},
					y1: {
						beginAtZero: true,
						position: 'right',
						title: {
							display: true,
							text: 'Estimated 1RM (kg)'
						},
						grid: {
							drawOnChartArea: false
						}
					}
				}
			}
		});
	}

	function createMonthlyVolumeChart() {
		if (!volumeStats) return;

		const ctx = document.getElementById('monthlyVolumeChart') as HTMLCanvasElement;
		if (!ctx) return;

		if (monthlyVolumeChart) monthlyVolumeChart.destroy();

		monthlyVolumeChart = new Chart(ctx, {
			type: 'bar',
			data: {
				labels: volumeStats.monthly_volume.map((v) => v.month),
				datasets: [
					{
						label: 'Monthly Volume (kg)',
						data: volumeStats.monthly_volume.map((v) => v.total_volume),
						backgroundColor: 'rgb(75, 192, 192)'
					}
				]
			},
			options: {
				responsive: true,
				scales: {
					y: {
						beginAtZero: true,
						title: {
							display: true,
							text: 'Volume (kg)'
						}
					}
				}
			}
		});
	}

	function createTimeDistributionChart() {
		if (!workoutStats?.popular_hours) return;

		const ctx = document.getElementById('timeDistributionChart') as HTMLCanvasElement;
		if (!ctx) return;

		if (timeDistributionChart) timeDistributionChart.destroy();

		timeDistributionChart = new Chart(ctx, {
			type: 'bar',
			data: {
				labels: workoutStats.popular_hours.map((h) => `${h.hour}:00`),
				datasets: [
					{
						label: 'Workouts',
						data: workoutStats.popular_hours.map((h) => h.count),
						backgroundColor: 'rgb(54, 162, 235)'
					}
				]
			},
			options: {
				responsive: true,
				scales: {
					y: {
						beginAtZero: true,
						title: {
							display: true,
							text: 'Number of Workouts'
						}
					}
				}
			}
		});
	}

	function createDurationDistributionChart() {
		if (!workoutStats?.duration_distribution) return;

		const ctx = document.getElementById('durationDistributionChart') as HTMLCanvasElement;
		if (!ctx) return;

		if (durationDistributionChart) durationDistributionChart.destroy();

		durationDistributionChart = new Chart(ctx, {
			type: 'bar',
			data: {
				labels: workoutStats.duration_distribution.map((d) => `${d.range} min`),
				datasets: [
					{
						label: 'Workouts',
						data: workoutStats.duration_distribution.map((d) => d.count),
						backgroundColor: 'rgb(153, 102, 255)'
					}
				]
			},
			options: {
				responsive: true,
				scales: {
					y: {
						beginAtZero: true,
						title: {
							display: true,
							text: 'Number of Workouts'
						}
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

		const data = exerciseProgress.map((ep) => ({
			x: new Date(ep.exercise.start_time).getTime(),
			y: Math.max(...ep.sets.map((s) => s.weight))
		}));

		progressChart = new Chart(ctx, {
			type: 'scatter',
			data: {
				datasets: [
					{
						label: 'Max Weight per Session',
						data: data,
						backgroundColor: 'rgb(75, 192, 192)',
						borderColor: 'rgb(75, 192, 192)',
						showLine: true
					}
				]
			},
			options: {
				responsive: true,
				scales: {
					x: {
						type: 'time',
						time: {
							unit: 'day',
							displayFormats: {
								day: 'MMM d, yyyy'
							}
						},
						title: {
							display: true,
							text: 'Date'
						}
					},
					y: {
						beginAtZero: true,
						title: {
							display: true,
							text: 'Weight (kg)'
						}
					}
				},
				plugins: {
					tooltip: {
						callbacks: {
							label: (context) => {
								return `Weight: ${context.parsed.y}kg`;
							}
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
				datasets: [
					{
						data: [
							workoutStats.feedback_distribution.good,
							workoutStats.feedback_distribution.neutral,
							workoutStats.feedback_distribution.bad
						],
						backgroundColor: ['rgb(75, 192, 192)', 'rgb(255, 205, 86)', 'rgb(255, 99, 132)']
					}
				]
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

# Progress Page

<div class="container mx-auto p-4">
	<h1 class="h1 mb-4">Progress</h1>

	<!-- Overall Stats -->
	{#if workoutStats}
		<div class="card p-4 mb-8">
			<h2 class="h2 mb-4">Overall Statistics</h2>
			<div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
				<div class="card variant-soft p-4">
					<h3 class="h3">Workout Frequency</h3>
					<p class="text-4xl font-bold">{workoutStats.workout_frequency.average_per_week}</p>
					<p class="text-sm">workouts per week</p>
					{#if workoutStats.workout_frequency.trend !== undefined}
						<div class="mt-2 flex items-center gap-1">
							{#if workoutStats.workout_frequency.trend > 0}
								<span class="text-green-500">↑</span>
								<span class="text-sm text-green-500"
									>+{workoutStats.workout_frequency.trend} last 4 weeks</span
								>
							{:else if workoutStats.workout_frequency.trend < 0}
								<span class="text-red-500">↓</span>
								<span class="text-sm text-red-500"
									>{workoutStats.workout_frequency.trend} last 4 weeks</span
								>
							{:else}
								<span class="text-sm">No change in last 4 weeks</span>
							{/if}
						</div>
					{/if}
				</div>
				<div class="card variant-soft p-4">
					<h3 class="h3">Average Duration</h3>
					<p class="text-4xl font-bold">{Math.round(workoutStats.average_duration_minutes)}</p>
					<p class="text-sm">minutes</p>
					{#if workoutStats.duration_trend !== undefined}
						<div class="mt-2 flex items-center gap-1">
							{#if workoutStats.duration_trend > 0}
								<span class="text-green-500">↑</span>
								<span class="text-sm text-green-500"
									>+{Math.round(workoutStats.duration_trend)} min last 4 weeks</span
								>
							{:else if workoutStats.duration_trend < 0}
								<span class="text-red-500">↓</span>
								<span class="text-sm text-red-500"
									>{Math.round(workoutStats.duration_trend)} min last 4 weeks</span
								>
							{:else}
								<span class="text-sm">No change in last 4 weeks</span>
							{/if}
						</div>
					{/if}
				</div>
				<div class="card variant-soft p-4">
					<h3 class="h3">Total Workouts</h3>
					<p class="text-4xl font-bold">{workoutStats.total_workouts}</p>
				</div>
			</div>

			<div class="grid grid-cols-1 md:grid-cols-2 gap-8">
				<div class="card variant-soft p-4">
					<h3 class="h3 mb-4">Workout Feedback</h3>
					<canvas id="feedbackChart"></canvas>
				</div>
				<div class="card variant-soft p-4">
					<h3 class="h3 mb-4">Popular Workout Times</h3>
					<canvas id="timeDistributionChart"></canvas>
				</div>
				<div class="card variant-soft p-4">
					<h3 class="h3 mb-4">Workout Duration Distribution</h3>
					<canvas id="durationDistributionChart"></canvas>
				</div>
			</div>
		</div>
	{/if}

	<!-- Exercise Progress -->
	<div class="card p-4 mb-8">
		<h2 class="h2 mb-4">Exercise Progress</h2>
		<select class="select mb-4" bind:value={selectedExercise}>
			{#each exerciseTypes as type}
				<option value={type}>{type}</option>
			{/each}
		</select>

		{#if volumeStats}
			<div class="grid grid-cols-1 gap-8">
				<!-- Personal Records -->
				<div class="card variant-soft p-4">
					<h3 class="h3 mb-4">Personal Records</h3>
					<div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
						<div>
							<h4 class="h4">All-Time Max Weight</h4>
							<p class="text-3xl font-bold">{volumeStats.personal_records.all_time_max_weight}kg</p>
						</div>
						<div>
							<h4 class="h4">Estimated 1RM</h4>
							<p class="text-3xl font-bold">{volumeStats.personal_records.estimated_max_1rm}kg</p>
						</div>
						<div>
							<h4 class="h4">Max Volume</h4>
							<p class="text-3xl font-bold">{volumeStats.personal_records.max_volume}kg</p>
						</div>
					</div>

					{#if volumeStats.personal_records.rep_prs}
						<div class="table-container">
							<table class="table table-hover">
								<thead>
									<tr>
										<th>Reps</th>
										<th>Weight (kg)</th>
									</tr>
								</thead>
								<tbody>
									{#each volumeStats.personal_records.rep_prs as pr}
										<tr>
											<td>{pr.reps}</td>
											<td>{pr.weight}</td>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
					{/if}
				</div>

				<div class="grid grid-cols-1 md:grid-cols-2 gap-8">
					<div class="card variant-soft p-4">
						<h3 class="h3 mb-4">Weekly Progress</h3>
						<canvas id="volumeChart"></canvas>
					</div>
					<div class="card variant-soft p-4">
						<h3 class="h3 mb-4">Weight Progress</h3>
						<canvas id="progressChart"></canvas>
					</div>
				</div>

				<div class="card variant-soft p-4">
					<h3 class="h3 mb-4">Monthly Volume</h3>
					<canvas id="monthlyVolumeChart"></canvas>
				</div>

				<!-- Set Schemes -->
				<div class="card variant-soft p-4">
					<h3 class="h3 mb-4">Recent Set Schemes</h3>
					<div class="flex flex-wrap gap-2">
						{#each volumeStats.weekly_volume.slice(-1)[0].set_schemes || [] as scheme}
							<span class="badge variant-filled">{scheme}</span>
						{/each}
					</div>
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
