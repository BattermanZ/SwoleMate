<!-- YOU CAN DELETE EVERYTHING IN THIS PAGE -->

<script lang="ts">
	import { createWorkout, createExercise, createSet, endWorkout, endExercise, getExerciseTypes, getWorkouts, getWorkout, getLastExerciseData, cancelExercise, cancelWorkout, syncOfflineMutations } from '$lib/api';
	import type { Workout, Exercise, UpdateExerciseRequest, Set as WorkoutSet } from '$lib/types';
	import { ProgressRadial, TabGroup, Tab, SlideToggle, RadioGroup, RadioItem, Autocomplete } from '@skeletonlabs/skeleton';
	import { logger } from '$lib/logger';
	import { onMount } from 'svelte';
	import { online } from '$lib/stores/network';
	import { loadWorkoutState, saveWorkoutState, clearWorkoutState } from '$lib/workoutState';
	import type { StoredWorkoutState } from '$lib/workoutState';

	// Workout session persistence
	function saveCurrentWorkout(workoutId: number | null) {
		if (workoutId) {
			localStorage.setItem('currentWorkoutId', workoutId.toString());
		} else {
			localStorage.removeItem('currentWorkoutId');
		}
	}

	function persistWorkoutState() {
		if (!currentWorkout) {
			clearWorkoutState();
			return;
		}

		const storedExercises = exercises.map((ex) => {
			const exerciseRecord: Exercise = {
				id: ex.id,
				workout_id: currentWorkout?.id ?? currentExercise?.workout_id ?? -1,
				exercise_type: ex.name,
				start_time: ex.start_time || new Date().toISOString(),
				end_time: ex.end_time || ex.start_time || new Date().toISOString(),
				notes: ex.notes
			};

			const setRecords: WorkoutSet[] = ex.sets.map((set) => ({
				id: set.id,
				exercise_id: ex.id ?? currentExercise?.id ?? -1,
				reps: set.reps,
				weight: set.weight,
				notes: undefined
			}));

			return {
				exercise: exerciseRecord,
				sets: setRecords
			};
		});

		saveWorkoutState({
			workout: currentWorkout,
			exercises: storedExercises,
			activeExerciseId: currentExercise?.id ?? null,
			sessionNotes,
			sessionFeedback
		});
	}

	function applyStoredState(state: StoredWorkoutState) {
		if (!state.workout) {
			return;
		}

		currentWorkout = state.workout;
		exercises = state.exercises.map(({ exercise, sets }) => {
			const isFinished = exercise.end_time !== exercise.start_time;

			if (state.activeExerciseId && exercise.id === state.activeExerciseId) {
				currentExercise = exercise;
			}

			return {
				id: exercise.id ?? undefined,
				name: exercise.exercise_type,
				sets: sets.map((set) => ({
					id: set.id,
					reps: set.reps,
					weight: set.weight,
					isEditing: false,
					isConfirmed: true
				})),
				showSetForm: !isFinished,
				notes: exercise.notes,
				isEditingNotes: false,
				end_time: exercise.end_time,
				start_time: exercise.start_time,
				lastExerciseData: undefined,
				isFinished
			};
		});

		if (!currentExercise) {
			const unfinished = state.exercises.find(({ exercise }) => exercise.end_time === exercise.start_time);
			if (unfinished) {
				currentExercise = unfinished.exercise;
			}
		}

		sessionNotes = state.sessionNotes;
		sessionFeedback = state.sessionFeedback;

		if (currentExercise && currentWorkout) {
			currentExercise = {
				...currentExercise,
				workout_id: currentWorkout.id ?? currentExercise.workout_id
			};
		}
	}

	async function loadCurrentWorkout() {
		const savedWorkoutId = localStorage.getItem('currentWorkoutId');
		const storedState = loadWorkoutState();
		if (savedWorkoutId) {
			try {
				const workoutId = parseInt(savedWorkoutId);
				if (workoutId < 0 && storedState) {
					applyStoredState(storedState);
					return;
				}
				const details = await getWorkout(workoutId);
				currentWorkout = details.workout;
				
				// Map the exercises to our client format
				exercises = details.exercises.map(e => ({
					id: e.exercise.id,
					name: e.exercise.exercise_type,
					sets: e.sets.map(s => ({
						id: s.id,
						reps: s.reps,
						weight: s.weight,
						isEditing: false,
						isConfirmed: true
					})),
					showSetForm: false,
					notes: e.exercise.notes,
					isEditingNotes: e.exercise.end_time === e.exercise.start_time,
					end_time: e.exercise.end_time,
					start_time: e.exercise.start_time,
					isFinished: e.exercise.end_time !== e.exercise.start_time
				}));

				// Set the current exercise to the last unfinished one
				const lastUnfinished = exercises.find(e => !e.isFinished);
				if (lastUnfinished) {
					currentExercise = {
						id: lastUnfinished.id,
						workout_id: workoutId,
						exercise_type: lastUnfinished.name,
						start_time: details.exercises.find(e => e.exercise.id === lastUnfinished.id)?.exercise.start_time || new Date().toISOString(),
						end_time: lastUnfinished.end_time || new Date().toISOString(),
						notes: lastUnfinished.notes
					};
				}
				persistWorkoutState();
			} catch (e) {
				logger.error('workout', 'Failed to load saved workout', { error: e });
				// If we fail to load the workout, clear the saved ID
				saveCurrentWorkout(null);
				if (storedState) {
					applyStoredState(storedState);
				}
			}
		} else if (storedState) {
			applyStoredState(storedState);
		}
	}

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
		start_time?: string;
		lastExerciseData?: {
			date: string;
			notes?: string;
			sets: Array<{
				reps: number;
				weight: number;
			}>;
		};
		isFinished: boolean;
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
	let recentWorkoutsWithExercises: Array<{
		workout: Workout;
		exercises: Array<{
			exercise: Exercise;
			sets: Array<WorkoutSet>;
		}>;
	}> = [];

	const FEEDBACK_OPTIONS = ['😊', '😐', '😞'] as const;
	type FeedbackEmoji = typeof FEEDBACK_OPTIONS[number];

	onMount(() => {
		let previousOnline = typeof navigator !== 'undefined' ? navigator.onLine : true;

		const initialize = async () => {
			try {
				await syncOfflineMutations();
				exerciseTypes = await getExerciseTypes();
				await loadCurrentWorkout();

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
				logger.error('workout', 'Failed to fetch initial data', { error: e });
			}
		};

		initialize();

		const unsubscribe = online.subscribe(async (value) => {
			if (value && !previousOnline) {
				await syncOfflineMutations();
				await loadCurrentWorkout();
				await refreshRecentWorkouts();
			}
			previousOnline = value;
		});

		return () => {
			unsubscribe();
		};
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
				end_time: now,
				notes: "Today's workout"
			};
			exercises = [];
			currentExercise = null;
			saveCurrentWorkout(result.id);
			persistWorkoutState();
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
				const currentExerciseIndex = exercises.findIndex(e => e.id === currentExercise?.id);
				if (currentExerciseIndex !== -1) {
					const exerciseNotes = exercises[currentExerciseIndex].notes;
					await endExercise(currentExercise.id, {
						end_time: now,
						notes: exerciseNotes
					});
					logger.info('workout', 'Previous exercise ended', {
						exerciseId: currentExercise.id,
						endTime: now,
						notes: exerciseNotes
					});
				}
			}

			// Get last exercise data
			let lastData = null;
			try {
				lastData = await getLastExerciseData(newExerciseName);
			} catch (err) {
				logger.warn('workout', 'Unable to load last exercise data', { error: err });
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
				notes: undefined
			});

			const newExercise = {
				id: result.id,
				workout_id: currentWorkout.id,
				exercise_type: newExerciseName,
				start_time: now,
				end_time: now,
				notes: undefined
			};

			currentExercise = newExercise;
			exercises = [...exercises, {
				id: result.id,
				name: newExerciseName,
				sets: [],
				showSetForm: true,
				isEditingNotes: true,
				notes: undefined,
				start_time: now,
				lastExerciseData: lastData ? {
					date: lastData.exercise.start_time,
					notes: lastData.exercise.notes,
					sets: lastData.sets.map(s => ({ reps: s.reps, weight: s.weight }))
				} : undefined,
				isFinished: false
			}];
			newExerciseName = '';
			showExerciseForm = false;
			persistWorkoutState();
			logger.info('workout', 'Exercise added successfully', { exerciseId: result.id });
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to add exercise';
			logger.error('workout', 'Failed to add exercise', { error });
		} finally {
			loading = false;
		}
	}

	async function refreshRecentWorkouts() {
		try {
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
			logger.error('workout', 'Failed to refresh recent workouts', { error: e });
		}
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
			persistWorkoutState();

			logger.info('workout', 'Set confirmed successfully', { setId: result.id });

			// Refresh recent workouts
			await refreshRecentWorkouts();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to confirm set';
			logger.error('workout', 'Failed to confirm set', { error });
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
				notes: exercise.notes
			};
			await endExercise(exercise.id, updateRequest);
			exercises[exerciseIndex].isEditingNotes = false;
			exercises = [...exercises];
			persistWorkoutState();
			logger.info('workout', 'Exercise notes updated', { exerciseId: exercise.id, notes: exercise.notes });

			// Refresh recent workouts to show updated notes
			await refreshRecentWorkouts();
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
		persistWorkoutState();
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
			const workoutId = currentWorkout.id; // Store the ID before clearing
			
			await endWorkout(currentWorkout.id, {
				end_time: now,
				notes: sessionNotes || undefined,
				feedback: sessionFeedback
			});

			// Clear the saved workout
			saveCurrentWorkout(null);
			clearWorkoutState();
			
			currentWorkout = null;
			currentExercise = null;
			exercises = [];
			showFeedbackModal = false;
			sessionNotes = '';
			sessionFeedback = null;

			logger.info('workout', 'Workout completed', { 
				workoutId, // Use the stored ID
				feedback: sessionFeedback
			});

			// Refresh recent workouts to show the completed one
			await refreshRecentWorkouts();
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

	function compressSets(sets: Array<{ reps: number; weight: number }>) {
		const compressed: Array<{ count: number; reps: number; weight: number }> = [];
		
		sets.forEach(set => {
			const existing = compressed.find(c => c.reps === set.reps && c.weight === set.weight);
			if (existing) {
				existing.count++;
			} else {
				compressed.push({ count: 1, reps: set.reps, weight: set.weight });
			}
		});
		
		return compressed;
	}

	async function cancelExerciseAndRefresh(exerciseIndex: number) {
		const exercise = exercises[exerciseIndex];
		if (!exercise.id) return;

		try {
			loading = true;
			error = null;
			await cancelExercise(exercise.id);
			exercises = exercises.filter((_, i) => i !== exerciseIndex);
			if (currentExercise?.id === exercise.id) {
				currentExercise = null;
			}
			persistWorkoutState();
			logger.info('workout', 'Exercise canceled', { exerciseId: exercise.id });
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to cancel exercise';
			logger.error('workout', 'Failed to cancel exercise', { error });
		} finally {
			loading = false;
		}
	}

	async function cancelWorkoutSession() {
		if (!currentWorkout?.id) return;

		try {
			loading = true;
			error = null;
			const workoutId = currentWorkout.id;
			await cancelWorkout(workoutId);
			
			// Clear the saved workout
			saveCurrentWorkout(null);
			clearWorkoutState();
			
			currentWorkout = null;
			currentExercise = null;
			exercises = [];
			logger.info('workout', 'Workout canceled', { workoutId });
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to cancel workout';
			logger.error('workout', 'Failed to cancel workout', { error });
		} finally {
			loading = false;
		}
	}

	$: {
		if (!currentWorkout) return;
		void [exercises, currentExercise, sessionNotes, sessionFeedback];
		persistWorkoutState();
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
		@apply transition-transform;
	}
	.exercise-card:hover {
		@apply scale-[1.01];
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
		@apply px-2 py-1 rounded-md bg-surface-900/50 text-sm whitespace-nowrap;
	}
	:global(.dark) .exercise-row {
		@apply bg-surface-900/50;
	}
	:global(.dark) .set-chip {
		@apply bg-surface-700/50;
	}
	.last-exercise-info {
		@apply text-sm p-3 rounded-lg bg-surface-900/30 flex flex-wrap items-center gap-3;
	}
	.last-sets {
		@apply flex gap-2 flex-wrap items-center;
	}
	:global(.dark) .last-exercise-info {
		@apply bg-surface-700/30;
	}
	.badge {
		@apply px-2 py-1 rounded-full text-sm font-medium whitespace-nowrap;
	}
	.badge .badge {
		@apply ml-1;
	}

	/* Prevent zoom on input fields */
	input[type="number"],
	input[type="text"] {
		@apply text-base;
		touch-action: manipulation;
		font-size: 16px !important;
	}

	/* Make input containers taller to accommodate the larger font size */
	.input-container {
		@apply flex items-center gap-2;
		min-height: 2.5rem;
	}
</style>

<div class="content-container">
	<header class="text-center space-y-2 sm:space-y-4 mb-4 sm:mb-6">
		<div class="card variant-filled-tertiary p-3 sm:p-4">
			<h1 class="h1 text-xl sm:text-2xl mb-2 sm:mb-4">Today's Workout</h1>
			{#if !currentWorkout}
				<div class="p-1 sm:p-2">
					<button 
						class="btn variant-filled-primary w-full md:w-auto {loading ? 'opacity-50' : ''}" 
						on:click={startWorkout} 
						disabled={loading}
					>
						{#if loading}
							<ProgressRadial width="w-6" stroke={150} meter="stroke-primary-500" track="stroke-primary-500/30"/>
						{:else}
							<span class="text-xl sm:text-2xl mr-2">💪</span> Start New Session
						{/if}
					</button>
				</div>
			{/if}
		</div>
	</header>

	{#if error}
		<div class="alert variant-filled-error mb-4">
			<span class="text-xl sm:text-2xl">⚠️</span>
			<span>{error}</span>
		</div>
	{/if}

	<div class="grid gap-4 sm:gap-6">
		{#if currentWorkout}
			<div class="workout-container">
				<div class="card variant-filled-surface p-2 sm:p-4 space-y-3 sm:space-y-5">
					<header class="flex flex-col sm:flex-row justify-between items-center gap-2 mb-2 sm:mb-4">
						<h2 class="h2 text-lg sm:text-2xl">Current Session</h2>
						<div class="flex gap-2 w-full sm:w-auto">
							<button class="btn btn-sm variant-soft-error flex-1 sm:flex-initial" on:click={cancelWorkoutSession}>
								<span class="text-sm sm:text-lg mr-1 sm:mr-2">❌</span>
								<span class="text-sm sm:text-base">Cancel</span>
							</button>
							<button class="btn btn-sm variant-filled-error flex-1 sm:flex-initial" on:click={endWorkoutSession}>
								<span class="text-sm sm:text-lg mr-1 sm:mr-2">🏁</span>
								<span class="text-sm sm:text-base">End</span>
							</button>
						</div>
					</header>
					<div class="space-y-5">
						{#each exercises as exercise, exerciseIndex}
							<div class="card variant-soft p-3 sm:p-5 exercise-card">
								<div class="flex flex-col gap-3 sm:gap-5">
									<div class="flex justify-between items-center">
										<span class="text-lg sm:text-xl font-bold">{exercise.name}</span>
										<button 
											class="btn btn-sm variant-soft-error"
											on:click={() => cancelExerciseAndRefresh(exerciseIndex)}
										>
											<span class="text-sm">❌</span>
										</button>
									</div>
									{#if exercise.lastExerciseData}
										<div class="last-exercise-info">
											<span class="opacity-75 whitespace-nowrap text-sm">Last time ({formatDateRelative(exercise.lastExerciseData.date)}):</span>
											<div class="last-sets">
												{#each compressSets(exercise.lastExerciseData.sets) as set}
													{#if set.count > 1}
														<span class="badge variant-filled-secondary text-xs">
															{set.count}×<span class="badge variant-filled-primary">{set.reps}×{set.weight}kg</span>
														</span>
													{:else}
														<span class="badge variant-filled-primary text-xs">{set.reps}×{set.weight}kg</span>
													{/if}
												{/each}
											</div>
											{#if exercise.lastExerciseData.notes}
												<div class="w-full mt-1 text-sm">
													<span class="opacity-75">Notes:</span> {exercise.lastExerciseData.notes}
												</div>
											{/if}
										</div>
									{/if}
									<div class="flex flex-col gap-3 sm:gap-4">
										<div class="flex flex-wrap gap-2 items-center">
											{#each exercise.sets as set, setIndex}
												{#if set.isEditing}
													<div class="card variant-ghost p-2 flex gap-2 items-center w-full sm:w-auto">
														<span class="text-xs opacity-75">{setIndex + 1}</span>
														<div class="input-container">
														<input
															type="number"
															inputmode="numeric"
															pattern="[0-9]*"
															class="input w-16 text-center"
															bind:value={set.reps}
															on:input={persistWorkoutState}
																min="0"
															/>
															<span class="text-base">×</span>
														<input
															type="number"
															inputmode="numeric"
															pattern="[0-9]*"
															class="input w-16 text-center"
															bind:value={set.weight}
															on:input={persistWorkoutState}
																min="0"
																step="0.5"
															/>
															<span class="text-base">kg</span>
														</div>
														<button 
															class="btn variant-filled-success btn-sm round-btn ml-auto"
															on:click={() => confirmSet(exerciseIndex, setIndex)}
															disabled={loading}
														>
															✓
														</button>
													</div>
												{:else}
													<div class="chip variant-filled text-sm">
														{set.reps}×{set.weight}kg
													</div>
												{/if}
											{/each}
											{#if !exercise.sets.some(s => s.isEditing)}
												<button 
													class="btn variant-filled-secondary round-btn btn-sm" 
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
												<input
													type="text"
													class="input flex-grow"
													placeholder="Exercise notes..."
													bind:value={exercise.notes}
													on:input={persistWorkoutState}
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
												<span class="flex-grow text-sm">{exercise.notes}</span>
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
							<form on:submit|preventDefault={addExercise} class="flex flex-col sm:flex-row gap-3">
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
													class="block w-full text-left px-4 py-3 hover:variant-soft-primary transition-colors"
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
								<div class="flex gap-2">
									<button type="submit" class="btn variant-filled-primary flex-1 sm:flex-none">
										<span class="text-lg mr-2">✨</span> Add
									</button>
									<button type="button" class="btn variant-soft flex-1 sm:flex-none" on:click={() => {
										showExerciseForm = false;
										filteredExerciseTypes = [];
									}}>
										Cancel
									</button>
								</div>
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
									<h3 class="h3">{formatDateRelative(workout.date)}</h3>
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
												<p class="text-sm opacity-90 mt-0.5">{exercise.notes}</p>
											{/if}
										</div>
										<div class="sets-list">
											{#each compressSets(sets) as set}
												{#if set.count > 1}
													<span class="badge variant-filled-secondary">
														{set.count}×<span class="badge variant-filled-primary">{set.reps}×{set.weight}kg</span>
													</span>
												{:else}
													<span class="badge variant-filled-primary">{set.reps}×{set.weight}kg</span>
												{/if}
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
							on:click={() => {
								sessionFeedback = emoji;
								persistWorkoutState();
							}}
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
					on:input={persistWorkoutState}
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
