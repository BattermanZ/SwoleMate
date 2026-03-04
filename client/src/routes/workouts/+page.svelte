<script lang="ts">
	import { getWorkout, getWorkouts, cancelWorkout, updateWorkoutTimes } from '$lib/api';
	import type { FeedbackEmoji, Workout, WorkoutWithExercises } from '$lib/types';
	import { auth } from '$lib/auth';
	import { logger } from '$lib/logger';
	import { formatDateRelative } from '$lib/utils/date';
	import EditWorkoutTimesModal from '$lib/components/history/EditWorkoutTimesModal.svelte';
	import WorkoutDetailsCard from '$lib/components/history/WorkoutDetailsCard.svelte';
	import { isWithinRange, resolveDateRange, type DateRangePreset } from '$lib/history/dateRange';

	export let data: { workouts: Workout[] };
	let workouts = data.workouts;
	let loading = false;
	let error: string | null = null;
	const authState = auth.state;

	type SortOrder = 'newest' | 'oldest' | 'longest' | 'shortest';

	let query = '';
	let rangePreset: DateRangePreset = 'all';
	let customFrom = '';
	let customTo = '';
	let mood: 'all' | FeedbackEmoji = 'all';
	let sort: SortOrder = 'newest';
	let selectedId: number | null = null;
	let pageIndex = 0;
	const PAGE_SIZE = 20;
	let lastFiltersKey = '';

	type DetailsState =
		| { status: 'idle' }
		| { status: 'loading' }
		| { status: 'loaded'; workout: WorkoutWithExercises }
		| { status: 'error'; message: string };

	let detailsById: Record<number, DetailsState> = {};

	function workoutDurationMinutes(workout: Workout): number | null {
		const start = new Date(workout.start_time).getTime();
		const end = new Date(workout.end_time).getTime();
		if (!Number.isFinite(start) || !Number.isFinite(end)) return null;
		if (end <= start) return null;
		return Math.round((end - start) / 60_000);
	}

	$: dateRange = resolveDateRange(rangePreset, customFrom, customTo);

	function matchesQuery(workout: Workout, q: string): boolean {
		const term = q.trim().toLowerCase();
		if (!term) return true;
		const haystack =
			`${workout.date} ${workout.notes ?? ''} ${formatDateRelative(workout.start_time)}`.toLowerCase();
		return haystack.includes(term);
	}

	$: sortedWorkouts = [...workouts].sort((a, b) => {
		if (sort === 'longest' || sort === 'shortest') {
			const ad = workoutDurationMinutes(a);
			const bd = workoutDurationMinutes(b);
			const aVal = ad ?? Number.POSITIVE_INFINITY;
			const bVal = bd ?? Number.POSITIVE_INFINITY;
			return sort === 'shortest' ? aVal - bVal : bVal - aVal;
		}

		const at = new Date(a.start_time).getTime();
		const bt = new Date(b.start_time).getTime();
		if (sort === 'oldest') return (Number.isFinite(at) ? at : 0) - (Number.isFinite(bt) ? bt : 0);
		return (Number.isFinite(bt) ? bt : 0) - (Number.isFinite(at) ? at : 0);
	});

	$: filteredWorkouts = sortedWorkouts.filter((w) => {
		if (mood !== 'all' && w.feedback !== mood) return false;
		if (!isWithinRange(w.start_time, dateRange)) return false;
		if (!matchesQuery(w, query)) return false;
		return true;
	});

	$: pageCount = Math.max(1, Math.ceil(filteredWorkouts.length / PAGE_SIZE));
	$: pageIndex = Math.min(pageIndex, pageCount - 1);
	$: pagedWorkouts = filteredWorkouts.slice(
		pageIndex * PAGE_SIZE,
		pageIndex * PAGE_SIZE + PAGE_SIZE
	);

	$: {
		const filtersKey = [query, rangePreset, customFrom, customTo, mood, sort].join('|');
		if (filtersKey !== lastFiltersKey) {
			pageIndex = 0;
			lastFiltersKey = filtersKey;
		}
	}

	$: {
		if (pagedWorkouts.length === 0) {
			selectedId = null;
		} else if (selectedId === null || !pagedWorkouts.some((w) => w.id === selectedId)) {
			selectedId = pagedWorkouts[0]?.id ?? null;
		}
	}

	async function ensureWorkoutDetails(id: number) {
		const state = detailsById[id];
		if (state?.status === 'loading' || state?.status === 'loaded') return;

		detailsById = { ...detailsById, [id]: { status: 'loading' } };

		try {
			const data = await getWorkout(id);
			const assembled: WorkoutWithExercises = { ...data.workout, exercises: data.exercises };
			detailsById = { ...detailsById, [id]: { status: 'loaded', workout: assembled } };
		} catch (e) {
			const message = e instanceof Error ? e.message : 'Failed to load workout details';
			detailsById = { ...detailsById, [id]: { status: 'error', message } };
		}
	}

	$: {
		if (selectedId !== null) void ensureWorkoutDetails(selectedId);
	}

	function selectWorkout(id: number | undefined) {
		if (!id) return;
		selectedId = id;
	}

	function statsSummary(items: Workout[]) {
		const total = items.length;
		const durations = items.map(workoutDurationMinutes).filter((d): d is number => d !== null);
		const avgDuration = durations.length
			? Math.round(durations.reduce((a, b) => a + b, 0) / durations.length)
			: null;

		const last30d = items.filter((w) => {
			const start = new Date(w.start_time).getTime();
			return Number.isFinite(start) && start >= Date.now() - 30 * 24 * 60 * 60 * 1000;
		}).length;

		let totalDurationForAvgExercise = 0;
		let totalExercisesForAvgExercise = 0;
		for (const w of items) {
			const duration = workoutDurationMinutes(w);
			const count = w.exercise_count ?? 0;
			if (duration === null) continue;
			if (!Number.isFinite(count) || count <= 0) continue;
			totalDurationForAvgExercise += duration;
			totalExercisesForAvgExercise += count;
		}

		const avgExerciseDuration = totalExercisesForAvgExercise
			? Math.round(totalDurationForAvgExercise / totalExercisesForAvgExercise)
			: null;

		return { total, avgDuration, last30d, avgExerciseDuration };
	}

	$: summary = statsSummary(filteredWorkouts);
	$: selectedState = selectedId ? detailsById[selectedId] : undefined;
	$: selectedWorkout = selectedState?.status === 'loaded' ? selectedState.workout : null;
	$: selectedError = selectedState?.status === 'error' ? selectedState.message : null;
	$: loadingSelected = selectedState?.status === 'loading';

	$: pageStart = filteredWorkouts.length ? pageIndex * PAGE_SIZE + 1 : 0;
	$: pageEnd = Math.min(filteredWorkouts.length, (pageIndex + 1) * PAGE_SIZE);
	$: canPrevPage = pageIndex > 0;
	$: canNextPage = pageIndex < pageCount - 1;

	let editTimesOpen = false;
	let editTimesError: string | null = null;
	let editTimesSaving = false;

	async function refreshWorkouts() {
		try {
			loading = true;
			error = null;
			workouts = await getWorkouts();
			detailsById = {};
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load workouts';
			logger.error('workout', 'Failed to load workouts', { error: e });
		} finally {
			loading = false;
		}
	}

	function openEditTimes() {
		editTimesError = null;
		editTimesOpen = true;
	}

	async function handleSaveTimes(
		e: CustomEvent<{
			start_time: string;
			end_time: string;
			notes: string | null;
			feedback: '😊' | '😐' | '😞' | null;
		}>
	) {
		if (!selectedId) return;
		editTimesSaving = true;
		editTimesError = null;
		try {
			await updateWorkoutTimes(selectedId, {
				start_time: e.detail.start_time,
				end_time: e.detail.end_time,
				notes: e.detail.notes,
				feedback: e.detail.feedback
			});
			editTimesOpen = false;
			await refreshWorkouts();
		} catch (err) {
			editTimesError = err instanceof Error ? err.message : 'Failed to update times';
		} finally {
			editTimesSaving = false;
		}
	}

	async function handleDeleteWorkout(workoutId: number | null) {
		if (!workoutId) {
			error = 'Invalid workout ID';
			return;
		}
		if ($authState.offline) {
			error = 'Offline mode: delete workouts when you are back online.';
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
				<h1 class="text-3xl sm:text-4xl font-black tracking-tight">History</h1>
				<p class="text-sm sm:text-base opacity-80 max-w-prose">
					Review past sessions, notes, and set schemes — tuned for quick scanning.
				</p>
			</div>

			<div class="flex flex-col sm:items-end gap-2">
				<button
					type="button"
					class="btn variant-soft"
					on:click={refreshWorkouts}
					disabled={loading}
				>
					Refresh
				</button>
				{#if error}
					<div class="text-sm text-error-500">{error}</div>
				{/if}
			</div>
		</div>

		<div class="relative mt-5 grid gap-3 grid-cols-2 sm:grid-cols-3 lg:grid-cols-5">
			<div class="card variant-glass-surface p-3 border-l-4 border-primary-500/70">
				<div class="text-xs font-semibold opacity-70">Filtered sessions</div>
				<div class="text-lg font-bold">{summary.total}</div>
			</div>
			<div class="card variant-glass-surface p-3 border-l-4 border-secondary-500/70">
				<div class="text-xs font-semibold opacity-70">Last 30 days</div>
				<div class="text-lg font-bold">{summary.last30d}</div>
			</div>
			<div class="card variant-glass-surface p-3 border-l-4 border-tertiary-500/70">
				<div class="text-xs font-semibold opacity-70">Avg duration</div>
				<div class="text-lg font-bold">{summary.avgDuration ?? '—'}m</div>
			</div>
			<div class="card variant-glass-surface p-3 border-l-4 border-warning-500/70">
				<div class="text-xs font-semibold opacity-70">Avg / exercise</div>
				<div class="text-lg font-bold">{summary.avgExerciseDuration ?? '—'}m</div>
			</div>
			<div class="card variant-glass-surface p-3 border-l-4 border-success-500/70">
				<div class="text-xs font-semibold opacity-70">Selected</div>
				<div class="text-lg font-bold truncate">
					{selectedWorkout ? formatDateRelative(selectedWorkout.start_time) : '—'}
				</div>
			</div>
		</div>
	</header>

	<div class="grid gap-6 md:grid-cols-12">
		<section class="md:col-span-5 space-y-4 min-w-0">
			<div class="card variant-glass-surface p-4 space-y-3 min-w-0">
				<div class="flex items-end justify-between gap-3">
					<div>
						<h2 class="text-lg font-semibold tracking-tight">Workouts</h2>
						<p class="text-sm opacity-70">Search and filter sessions.</p>
					</div>
					<div class="text-sm opacity-70">{filteredWorkouts.length} total</div>
				</div>

				<div class="grid gap-2 sm:grid-cols-2">
					<label class="block min-w-0">
						<span class="text-xs font-semibold opacity-70">Search</span>
						<input
							class="input w-full mt-1"
							placeholder="Notes, date…"
							bind:value={query}
							disabled={loading}
						/>
					</label>
					<label class="block min-w-0">
						<span class="text-xs font-semibold opacity-70">Date range</span>
						<select class="select w-full mt-1" bind:value={rangePreset} disabled={loading}>
							<option value="all">All</option>
							<option value="30d">Last 30 days</option>
							<option value="90d">Last 90 days</option>
							<option value="365d">Last year</option>
							<option value="custom">Custom…</option>
						</select>
					</label>
					{#if rangePreset === 'custom'}
						<div class="sm:col-span-2 grid gap-2 sm:grid-cols-2">
							<label class="block min-w-0">
								<span class="text-xs font-semibold opacity-70">From</span>
								<input
									type="date"
									class="input w-full mt-1"
									bind:value={customFrom}
									disabled={loading}
								/>
							</label>
							<label class="block min-w-0">
								<span class="text-xs font-semibold opacity-70">To</span>
								<input
									type="date"
									class="input w-full mt-1"
									bind:value={customTo}
									disabled={loading}
								/>
							</label>
							<div class="sm:col-span-2 flex justify-end">
								<button
									type="button"
									class="btn btn-sm variant-soft"
									on:click={() => {
										customFrom = '';
										customTo = '';
									}}
									disabled={loading}
								>
									Clear
								</button>
							</div>
						</div>
					{/if}
					<label class="block min-w-0">
						<span class="text-xs font-semibold opacity-70">Mood</span>
						<select class="select w-full mt-1" bind:value={mood} disabled={loading}>
							<option value="all">All</option>
							<option value="😊">😊 Good</option>
							<option value="😐">😐 Neutral</option>
							<option value="😞">😞 Bad</option>
						</select>
					</label>
					<label class="block min-w-0">
						<span class="text-xs font-semibold opacity-70">Sort</span>
						<select class="select w-full mt-1" bind:value={sort} disabled={loading}>
							<option value="newest">Newest first</option>
							<option value="oldest">Oldest first</option>
							<option value="longest">Longest first</option>
							<option value="shortest">Shortest first</option>
						</select>
					</label>
				</div>

				{#if loading}
					<div class="card variant-ghost p-4 text-center opacity-80">Loading workouts…</div>
				{:else if filteredWorkouts.length === 0}
					<div class="card variant-ghost p-4 text-center opacity-80">
						No workouts match filters.
					</div>
				{:else}
					<div class="flex items-center justify-between gap-3">
						<div class="text-sm opacity-70">
							Showing {pageStart}-{pageEnd} of {filteredWorkouts.length}
						</div>
						<div class="flex items-center gap-2">
							<button
								type="button"
								class="btn btn-sm variant-soft"
								on:click={() => (pageIndex = Math.max(0, pageIndex - 1))}
								disabled={!canPrevPage}
								aria-label="Previous page"
							>
								←
							</button>
							<div class="text-sm font-semibold tabular-nums">
								{pageIndex + 1}/{pageCount}
							</div>
							<button
								type="button"
								class="btn btn-sm variant-soft"
								on:click={() => (pageIndex = Math.min(pageCount - 1, pageIndex + 1))}
								disabled={!canNextPage}
								aria-label="Next page"
							>
								→
							</button>
						</div>
					</div>
					<div class="space-y-2">
						{#each pagedWorkouts as workout (workout.id)}
							<button
								type="button"
								class="w-full text-left rounded-2xl border p-3 transition-colors min-w-0 {workout.id ===
								selectedId
									? 'border-primary-500/60 bg-primary-500/10'
									: 'border-surface-200/50 bg-surface-50/60 dark:border-surface-700/50 dark:bg-surface-950/30 hover:bg-surface-50/80 dark:hover:bg-surface-950/45'}"
								on:click={() => selectWorkout(workout.id)}
								aria-current={workout.id === selectedId ? 'true' : undefined}
							>
								<div class="flex items-start justify-between gap-3">
									<div class="min-w-0">
										<div class="flex items-center gap-2">
											<div class="font-semibold">{formatDateRelative(workout.start_time)}</div>
											{#if workout.auto_closed_at}
												<span class="badge variant-soft-warning text-xs">Auto-closed</span>
											{/if}
											{#if workout.feedback}
												<span class="text-lg" aria-label="Session mood">{workout.feedback}</span>
											{/if}
										</div>
										<div class="text-sm opacity-70">{workoutDurationMinutes(workout) ?? '—'}m</div>
										{#if workout.notes}
											<div class="text-sm opacity-70 mt-1 line-clamp-2">{workout.notes}</div>
										{/if}
									</div>
									<a
										href="/workouts/{workout.id}"
										class="btn btn-sm variant-ghost-primary whitespace-nowrap"
										on:click|stopPropagation
									>
										View →
									</a>
								</div>
							</button>
						{/each}
					</div>
				{/if}
			</div>
		</section>

		<aside class="md:col-span-7 min-w-0 space-y-4">
			<WorkoutDetailsCard workout={selectedWorkout} loading={loadingSelected} error={selectedError}>
				<svelte:fragment slot="actions">
					{#if selectedId !== null}
						<button
							type="button"
							class="btn btn-sm variant-soft"
							on:click={openEditTimes}
							disabled={loading || loadingSelected}
						>
							Edit times
						</button>
						<button
							type="button"
							class="btn btn-sm variant-soft-error"
							on:click={() => handleDeleteWorkout(selectedId)}
							disabled={loading}
						>
							Delete
						</button>
						<a href="/workouts/{selectedId}" class="btn btn-sm variant-filled-primary">Open →</a>
					{/if}
				</svelte:fragment>
			</WorkoutDetailsCard>
		</aside>
	</div>
</div>

<EditWorkoutTimesModal
	open={editTimesOpen}
	workout={selectedWorkout}
	disabled={editTimesSaving}
	error={editTimesError}
	on:cancel={() => (editTimesOpen = false)}
	on:submit={handleSaveTimes}
/>

<style>
	.line-clamp-2 {
		display: -webkit-box;
		line-clamp: 2;
		-webkit-line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}
</style>
