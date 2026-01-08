<script lang="ts">
	import { getWorkout, getWorkouts, cancelWorkout } from '$lib/api';
	import type { FeedbackEmoji, Workout, WorkoutWithExercises } from '$lib/types';
	import { logger } from '$lib/logger';
	import { formatDateRelative } from '$lib/utils/date';
	import WorkoutDetailsCard from '$lib/components/history/WorkoutDetailsCard.svelte';

	export let data: { workouts: Workout[] };
	let workouts = data.workouts;
	let loading = false;
	let error: string | null = null;

	type RangeFilter = 'all' | '30d' | '90d' | '365d';
	type SortOrder = 'newest' | 'oldest';

	let query = '';
	let range: RangeFilter = 'all';
	let mood: 'all' | FeedbackEmoji = 'all';
	let sort: SortOrder = 'newest';
	let selectedId: number | null = null;

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

	function withinRange(workout: Workout, filter: RangeFilter): boolean {
		if (filter === 'all') return true;
		const days = filter === '30d' ? 30 : filter === '90d' ? 90 : filter === '365d' ? 365 : 0;
		if (!days) return true;
		const start = new Date(workout.start_time).getTime();
		if (!Number.isFinite(start)) return true;
		const cutoff = Date.now() - days * 24 * 60 * 60 * 1000;
		return start >= cutoff;
	}

	function matchesQuery(workout: Workout, q: string): boolean {
		const term = q.trim().toLowerCase();
		if (!term) return true;
		const haystack =
			`${workout.date} ${workout.notes ?? ''} ${formatDateRelative(workout.start_time)}`.toLowerCase();
		return haystack.includes(term);
	}

	$: sortedWorkouts = [...workouts].sort((a, b) => {
		const at = new Date(a.start_time).getTime();
		const bt = new Date(b.start_time).getTime();
		if (sort === 'oldest') return (Number.isFinite(at) ? at : 0) - (Number.isFinite(bt) ? bt : 0);
		return (Number.isFinite(bt) ? bt : 0) - (Number.isFinite(at) ? at : 0);
	});

	$: filteredWorkouts = sortedWorkouts.filter((w) => {
		if (mood !== 'all' && w.feedback !== mood) return false;
		if (!withinRange(w, range)) return false;
		if (!matchesQuery(w, query)) return false;
		return true;
	});

	$: {
		if (filteredWorkouts.length === 0) {
			selectedId = null;
		} else if (selectedId === null || !filteredWorkouts.some((w) => w.id === selectedId)) {
			selectedId = filteredWorkouts[0].id ?? null;
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

		const last7d = items.filter((w) => {
			const start = new Date(w.start_time).getTime();
			return Number.isFinite(start) && start >= Date.now() - 7 * 24 * 60 * 60 * 1000;
		}).length;

		return { total, avgDuration, last7d };
	}

	$: summary = statsSummary(filteredWorkouts);
	$: selectedState = selectedId ? detailsById[selectedId] : undefined;
	$: selectedWorkout = selectedState?.status === 'loaded' ? selectedState.workout : null;
	$: selectedError = selectedState?.status === 'error' ? selectedState.message : null;
	$: loadingSelected = selectedState?.status === 'loading';

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

	async function handleDeleteWorkout(workoutId: number | null) {
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

		<div class="relative mt-5 grid gap-3 grid-cols-2 sm:grid-cols-4">
			<div class="card variant-glass-surface p-3 border-l-4 border-primary-500/70">
				<div class="text-xs font-semibold opacity-70">Filtered sessions</div>
				<div class="text-lg font-bold">{summary.total}</div>
			</div>
			<div class="card variant-glass-surface p-3 border-l-4 border-secondary-500/70">
				<div class="text-xs font-semibold opacity-70">Last 7 days</div>
				<div class="text-lg font-bold">{summary.last7d}</div>
			</div>
			<div class="card variant-glass-surface p-3 border-l-4 border-tertiary-500/70">
				<div class="text-xs font-semibold opacity-70">Avg duration</div>
				<div class="text-lg font-bold">{summary.avgDuration ?? '—'}m</div>
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
					<div class="text-sm opacity-70">{filteredWorkouts.length} shown</div>
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
						<select class="select w-full mt-1" bind:value={range} disabled={loading}>
							<option value="all">All</option>
							<option value="30d">Last 30 days</option>
							<option value="90d">Last 90 days</option>
							<option value="365d">Last year</option>
						</select>
					</label>
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
					<div class="space-y-2">
						{#each filteredWorkouts as workout (workout.id)}
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

<style>
	.line-clamp-2 {
		display: -webkit-box;
		line-clamp: 2;
		-webkit-line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}
</style>
