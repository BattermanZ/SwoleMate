<!-- YOU CAN DELETE EVERYTHING IN THIS PAGE -->

<script lang="ts">
	import { createWorkout, createExercise, createSet, endWorkout, endExercise, getExerciseTypes, getWorkouts, getWorkout } from '$lib/api';
	import type { Workout, Exercise, UpdateExerciseRequest, Set as WorkoutSet } from '$lib/types';
	import { ProgressRadial, TabGroup, Tab, SlideToggle, RadioGroup, RadioItem, Autocomplete } from '@skeletonlabs/skeleton';
	import { logger } from '$lib/logger';
	import { onMount } from 'svelte';

	let currentWorkout: Workout | null = null;
	let currentExercise: Exercise | null = null;
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
		notes?: string;
		isEditingNotes: boolean;
		end_time?: string;
	}> = [];
	let loading = false;
	let error: string | null = null;
	let showExerciseForm = false;
	let newExerciseName = '';
	let showFeedbackModal = false;
	let sessionNotes = '';
	let sessionFeedback: '😊' | '😐' | '😞' | null = null;
	let exerciseTypes: string[] = [];
	let filteredExerciseTypes: string[] = [];
	let inputValue = '';
	let recentWorkouts: Workout[] = [];
	let recentWorkoutsWithExercises: Array<{
		workout: Workout;
		exercises: Array<{
			exercise: Exercise;
			sets: Array<WorkoutSet>;
		}>;
	}> = [];

	const FEEDBACK_OPTIONS = ['😊', '😐', '😞'] as const;
	type FeedbackEmoji = typeof FEEDBACK_OPTIONS[number];

	onMount(async () => {
		try {
			exerciseTypes = await getExerciseTypes();
			const workouts = (await getWorkouts()).slice(0, 3);
			
			// Fetch complete data for each workout
			recentWorkoutsWithExercises = await Promise.all(
				workouts.map(async (workout) => {
					const details = await getWorkout(workout.id!);
					return {
						workout: details.workout,
						exercises: details.exercises
					};
				})
			);
		} catch (e) {
			logger.error('workout', 'Failed to fetch initial data', { error: e });
		}
	});

	function filterExerciseTypes(input: string) {
		const searchTerm = input.toLowerCase().trim();
		
		// Show all types if input is empty
		if (!searchTerm) {
			filteredExerciseTypes = exerciseTypes.slice(0, 10); // Show top 10 by default
			return;
		}

		// Filter and sort by relevance
		filteredExerciseTypes = exerciseTypes
			.filter(type => type.toLowerCase().includes(searchTerm))
			.sort((a, b) => {
				// Exact matches first
				const aStartsWith = a.toLowerCase().startsWith(searchTerm);
				const bStartsWith = b.toLowerCase().startsWith(searchTerm);
				if (aStartsWith && !bStartsWith) return -1;
				if (!aStartsWith && bStartsWith) return 1;
				return a.localeCompare(b);
			})
			.slice(0, 10); // Limit to top 10 matches
	}

	function handleSelect(event: CustomEvent<string>) {
		newExerciseName = event.detail;
	}

	async function startWorkout() {
		try {
			loading = true;
			error = null;
			logger.info('workout', 'Starting new workout session');
			const now = new Date().toISOString();
			const result = await createWorkout({
				date: now,
				start_time: now,
				notes: "Today's workout"
			});
			currentWorkout = {
				id: result.id,
				date: now,
				start_time: now,
				end_time: now, // Will be updated when workout ends
				notes: "Today's workout"
			};
			logger.info('workout', 'Workout session started', { workoutId: result.id });
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to start workout';
			logger.error('workout', 'Failed to start workout', { error });
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

			// If there's a current exercise, end it
			if (currentExercise?.id) {
				const now = new Date().toISOString();
				await endExercise(currentExercise.id, { end_time: now });
				logger.info('workout', 'Previous exercise ended', { 
					exerciseId: currentExercise.id,
					endTime: now
				});
			}

			// Start new exercise
			const now = new Date().toISOString();
			logger.info('workout', 'Adding new exercise', { 
				workoutId: currentWorkout.id,
				exerciseName: newExerciseName
			});

			const result = await createExercise(currentWorkout.id, {
				exercise_type: newExerciseName,
				start_time: now,
				notes: ''
			});

			const newExercise = {
				id: result.id,
				workout_id: currentWorkout.id,
				exercise_type: newExerciseName,
				start_time: now,
				end_time: now, // Will be updated when next exercise starts or workout ends
				notes: ''
			};

			currentExercise = newExercise;
			exercises = [...exercises, {
				id: result.id,
				name: newExerciseName,
				sets: [],
				showSetForm: true,
				notes: '',
				isEditingNotes: true
			}];
			newExerciseName = '';
			showExerciseForm = false;
			logger.info('workout', 'Exercise added successfully', { exerciseId: result.id });
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to add exercise';
			logger.error('workout', 'Failed to add exercise', { error });
		} finally {
			loading = false;
		}
	}

	async function updateExerciseNotes(exerciseIndex: number) {
		const exercise = exercises[exerciseIndex];
		if (!exercise.id) return;

		try {
			const updateRequest: UpdateExerciseRequest = {
				end_time: exercise.end_time || new Date().toISOString(),
				notes: exercise.notes || undefined
			};
			await endExercise(exercise.id, updateRequest);
			exercises[exerciseIndex].isEditingNotes = false;
			exercises = [...exercises];
			logger.info('workout', 'Exercise notes updated', { exerciseId: exercise.id });

			// Refresh recent workouts to show the updated notes
			const workouts = (await getWorkouts()).slice(0, 3);
			recentWorkoutsWithExercises = await Promise.all(
				workouts.map(async (workout) => {
					const details = await getWorkout(workout.id!);
					return {
						workout: details.workout,
						exercises: details.exercises
					};
				})
			);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to update exercise notes';
			logger.error('workout', 'Failed to update exercise notes', { error });
		}
	}

	async function addSet(exerciseIndex: number) {
		const exercise = exercises[exerciseIndex];
		if (!exercise.id) return;

		// Get the last set's values if they exist
		const lastSet = exercise.sets[exercise.sets.length - 1];
		const defaultReps = lastSet ? lastSet.reps : 12;
		const defaultWeight = lastSet ? lastSet.weight : 0;

		exercises[exerciseIndex].sets = [...exercise.sets, {
			reps: defaultReps,
			weight: defaultWeight,
			isEditing: true,
			isConfirmed: false
		}];
		exercises = [...exercises];
		logger.debug('workout', 'New set form added', { exerciseId: exercise.id });
	}

	async function confirmSet(exerciseIndex: number, setIndex: number) {
		const exercise = exercises[exerciseIndex];
		const set = exercise.sets[setIndex];
		if (!exercise.id || !set) return;

		try {
			loading = true;
			error = null;
			logger.info('workout', 'Confirming set', { 
				exerciseId: exercise.id,
				reps: set.reps,
				weight: set.weight
			});
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

			// Update exercise notes to ensure they're preserved
			if (exercise.notes) {
				const updateRequest: UpdateExerciseRequest = {
					end_time: exercise.end_time || new Date().toISOString(),
					notes: exercise.notes
				};
				await endExercise(exercise.id, updateRequest);
			}

			logger.info('workout', 'Set confirmed successfully', { setId: result.id });

			// Refresh recent workouts to show the updated data
			const workouts = (await getWorkouts()).slice(0, 3);
			recentWorkoutsWithExercises = await Promise.all(
				workouts.map(async (workout) => {
					const details = await getWorkout(workout.id!);
					return {
						workout: details.workout,
						exercises: details.exercises
					};
				})
			);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to confirm set';
			logger.error('workout', 'Failed to confirm set', { error });
		} finally {
			loading = false;
		}
	}

	async function endWorkoutSession() {
		if (!currentWorkout?.id) return;
		showFeedbackModal = true;
	}

	async function submitWorkoutFeedback() {
		if (!currentWorkout?.id || !sessionFeedback) return;

		try {
			loading = true;
			error = null;
			const now = new Date().toISOString();

			// End current exercise if exists
			if (currentExercise?.id) {
				await endExercise(currentExercise.id, { end_time: now });
				logger.info('workout', 'Final exercise ended', { 
					exerciseId: currentExercise.id,
					endTime: now
				});
			}

			// End workout with feedback
			await endWorkout(currentWorkout.id, { 
				end_time: now,
				notes: sessionNotes,
				feedback: sessionFeedback
			});
			logger.info('workout', 'Ending workout session', { 
				workoutId: currentWorkout.id,
				endTime: now,
				feedback: sessionFeedback
			});

			// Reset state
			currentWorkout = null;
			currentExercise = null;
			exercises = [];
			showExerciseForm = false;
			showFeedbackModal = false;
			sessionNotes = '';
			sessionFeedback = null;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to end workout';
			logger.error('workout', 'Failed to end workout', { error });
		} finally {
			loading = false;
		}
	}

	function formatDate(dateString: string): string {
		return new Date(dateString).toLocaleDateString();
	}

	function formatTime(dateString: string): string {
		return new Date(dateString).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
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
	.exercise-card {
		transition: transform 0.2s;
	}
	.exercise-card:hover {
		transform: scale(1.01);
	}
	.round-btn {
		@apply aspect-square rounded-full flex items-center justify-center;
		padding: 0;
		width: 2rem;
		height: 2rem;
	}
	.content-container {
		@apply flex flex-col min-h-full;
	}
	.workout-container {
		@apply flex-1 flex flex-col;
	}
	:global(.dark) .variant-ghost {
		background-color: rgba(0, 0, 0, 0.2) !important;
	}
	:global(.dark) .variant-soft {
		background-color: rgba(0, 0, 0, 0.3) !important;
	}
	:global(.dark) .variant-soft-primary {
		background-color: rgba(var(--color-primary-500), 0.2) !important;
	}
	.exercise-list {
		@apply grid gap-1;
	}
	.exercise-row {
		@apply flex justify-between items-center p-2 rounded-lg bg-surface-700/50;
	}
	.exercise-name {
		@apply font-semibold;
	}
	.sets-list {
		@apply flex flex-wrap gap-1 items-center;
	}
	.set-chip {
		@apply px-2 py-1 rounded-md bg-surface-900/50 text-sm;
	}
	:global(.dark) .exercise-row {
		@apply bg-surface-900/50;
	}
	:global(.dark) .set-chip {
		@apply bg-surface-700/50;
	}
</style>

<div class="content-container">
	<header class="text-center space-y-4 mb-8">
		<div class="card variant-filled-tertiary p-4">
			<h1 class="h1">Today's Workout</h1>
			{#if !currentWorkout}
				<div class="p-4">
					<button 
						class="btn variant-filled-primary w-full md:w-auto {loading ? 'opacity-50' : ''}" 
						on:click={startWorkout} 
						disabled={loading}
					>
						{#if loading}
							<ProgressRadial width="w-6" stroke={150} meter="stroke-primary-500" track="stroke-primary-500/30"/>
						{:else}
							<span class="text-2xl mr-2">💪</span> Start New Session
						{/if}
					</button>
				</div>
			{/if}
		</div>
	</header>

	{#if error}
		<div class="alert variant-filled-error">
			<span class="text-2xl">⚠️</span>
			<span>{error}</span>
		</div>
	{/if}

	<div class="grid gap-8">
		{#if currentWorkout}
			<div class="workout-container">
				<div class="card variant-filled-surface p-4 space-y-4">
					<header class="flex justify-between items-center">
						<h2 class="h2">Current Session</h2>
						<button class="btn variant-soft-error" on:click={endWorkoutSession}>
							<span class="text-lg mr-2">🏁</span> End Session
						</button>
					</header>

					<div class="space-y-4">
						{#each exercises as exercise, exerciseIndex}
							<div class="card variant-soft p-4 exercise-card">
								<div class="flex flex-col gap-4">
									<div class="flex items-center gap-4">
										<span class="text-lg font-bold">{exercise.name}</span>
										<div class="flex-grow flex flex-wrap gap-2 items-center">
											{#each exercise.sets as set, setIndex}
												{#if set.isEditing}
													<div class="card variant-ghost p-2 flex gap-2 items-center">
														<span class="text-sm">{setIndex + 1}</span>
														<input
															type="number"
															inputmode="numeric"
															pattern="[0-9]*"
															class="input w-16 text-center"
															bind:value={set.reps}
															min="0"
														/>
														<span>×</span>
														<input
															type="number"
															inputmode="numeric"
															pattern="[0-9]*"
															class="input w-16 text-center"
															bind:value={set.weight}
															min="0"
															step="0.5"
														/>
														<span>kg</span>
														<button 
															class="btn variant-filled-success btn-sm round-btn"
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
													class="btn variant-filled-secondary round-btn" 
													on:click={() => addSet(exerciseIndex)}
												>
													+
												</button>
											{/if}
										</div>
									</div>
									
									{#if exercise.isEditingNotes}
										<div class="card variant-ghost p-2">
											<div class="flex gap-2 items-center">
												<span class="text-lg">📝</span>
												<input
													type="text"
													class="input flex-grow"
													placeholder="Exercise notes..."
													bind:value={exercise.notes}
												/>
												<button 
													class="btn variant-filled-success btn-sm round-btn"
													on:click={() => updateExerciseNotes(exerciseIndex)}
												>
													✓
												</button>
											</div>
										</div>
									{:else if exercise.notes}
										<div class="card variant-ghost p-2">
											<div class="flex gap-2 items-center">
												<span class="text-lg">📝</span>
												<span class="flex-grow">{exercise.notes}</span>
												<button 
													class="btn variant-filled btn-sm round-btn"
													on:click={() => exercises[exerciseIndex].isEditingNotes = true}
												>
													✎
												</button>
											</div>
										</div>
									{/if}
								</div>
							</div>
						{/each}
					</div>

					{#if showExerciseForm}
						<div class="card variant-ghost p-4">
							<form on:submit|preventDefault={addExercise} class="flex gap-2">
								<div class="relative flex-grow">
									<input
										type="text"
										class="input w-full"
										placeholder="Exercise name"
										bind:value={newExerciseName}
										on:input={(e) => filterExerciseTypes(e.currentTarget.value)}
										required
										autocomplete="off"
									/>
									{#if filteredExerciseTypes.length > 0}
										<div class="absolute w-full mt-1 max-h-48 overflow-y-auto z-50 card variant-filled-surface shadow-xl">
											{#each filteredExerciseTypes as type}
												<button
													class="block w-full text-left px-4 py-2 hover:variant-soft-primary transition-colors"
													on:click|preventDefault={() => {
														newExerciseName = type;
														filteredExerciseTypes = [];
													}}
												>
													{type}
												</button>
											{/each}
										</div>
									{/if}
								</div>
								<button type="submit" class="btn variant-filled-primary">
									<span class="text-lg mr-2">✨</span> Add
								</button>
								<button type="button" class="btn variant-soft" on:click={() => {
									showExerciseForm = false;
									filteredExerciseTypes = [];
								}}>
									Cancel
								</button>
							</form>
						</div>
					{:else}
						<button 
							class="btn variant-filled w-full" 
							on:click={() => {
								showExerciseForm = true;
								filterExerciseTypes('');
							}}
						>
							<span class="text-lg mr-2">{currentExercise ? '➡️' : '+'}</span>
							{currentExercise ? 'Next Exercise' : 'Add Exercise'}
						</button>
					{/if}
				</div>
			</div>
		{/if}

		{#if recentWorkoutsWithExercises.length > 0}
			<div class="card variant-ghost p-4 space-y-4">
				<h2 class="h2 text-center">Recent Workouts</h2>
				<div class="grid gap-4">
					{#each recentWorkoutsWithExercises as {workout, exercises}}
						<div class="card variant-soft p-4 space-y-2">
							<div class="flex justify-between items-center">
								<div>
									<h3 class="h3">{formatDate(workout.date)}</h3>
									<p class="opacity-80">
										{formatTime(workout.start_time)} - {formatTime(workout.end_time)}
									</p>
								</div>
								<div class="flex items-center gap-2">
									{#if workout.feedback}
										<span class="text-2xl">{workout.feedback}</span>
									{/if}
									<a href="/workouts/{workout.id}" class="btn btn-sm variant-soft">View Details →</a>
								</div>
							</div>
							
							{#if workout.notes}
								<p class="opacity-70">{workout.notes}</p>
							{/if}

							<div class="exercise-list">
								{#each exercises as {exercise, sets}}
									<div class="exercise-row">
										<div class="flex-1">
											<span class="exercise-name">{exercise.exercise_type}</span>
											{#if exercise.notes}
												<p class="text-sm opacity-90 mt-0.5">📝 {exercise.notes}</p>
											{/if}
										</div>
										<div class="sets-list">
											{#each sets as set}
												<span class="set-chip">
													{set.reps}×{set.weight}kg
												</span>
											{/each}
										</div>
									</div>
								{/each}
							</div>
						</div>
					{/each}
				</div>
			</div>
		{/if}
	</div>

	<!-- Feedback Modal -->
	{#if showFeedbackModal}
		<div class="modal-backdrop fixed inset-0 bg-black/50 flex items-center justify-center">
			<div class="modal card variant-filled-surface p-4 w-full max-w-lg mx-4 space-y-4">
				<header class="text-center">
					<h3 class="h3">How was your workout?</h3>
				</header>

				<div class="flex justify-center gap-4">
					{#each FEEDBACK_OPTIONS as emoji}
						<button
							class="card {sessionFeedback === emoji ? 'variant-filled-primary' : 'variant-soft'} p-4 text-4xl hover:scale-110 transition-transform"
							on:click={() => sessionFeedback = emoji}
						>
							{emoji}
						</button>
					{/each}
				</div>

				<textarea
					class="textarea"
					rows="3"
					placeholder="Add notes about your session..."
					bind:value={sessionNotes}
				></textarea>

				<footer class="flex justify-end gap-2">
					<button 
						class="btn variant-soft"
						on:click={() => showFeedbackModal = false}
					>
						Cancel
					</button>
					<button
						class="btn variant-filled-primary"
						on:click={submitWorkoutFeedback}
						disabled={!sessionFeedback || loading}
					>
						{#if loading}
							<ProgressRadial width="w-6" stroke={150} meter="stroke-primary-500" track="stroke-primary-500/30"/>
						{:else}
							<span class="text-lg mr-2">✨</span> Submit
						{/if}
					</button>
				</footer>
			</div>
		</div>
	{/if}
</div>
