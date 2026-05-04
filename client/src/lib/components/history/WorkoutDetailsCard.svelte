<script lang="ts">
	import type { WorkoutWithExercises, Set } from '$lib/types';
	import { formatDateRelative, formatTime } from '$lib/utils/date';
	import SetPillsHybrid from '$lib/components/ui/SetPillsHybrid.svelte';

	export let workout: WorkoutWithExercises | null = null;
	export let loading = false;
	export let error: string | null = null;
	export let title = 'Workout details';
	export let subtitle: string | null = null;

	function durationMinutes(startTime: string, endTime: string): number | null {
		const start = new Date(startTime).getTime();
		const end = new Date(endTime).getTime();
		if (!Number.isFinite(start) || !Number.isFinite(end)) return null;
		if (end <= start) return null;
		return Math.round((end - start) / 60_000);
	}

	function setTotalWeight(set: Set, perSideWeight: boolean, splitWeight: boolean): number {
		if (!perSideWeight) return set.weight;
		if (!splitWeight) return set.weight * 2;
		const left = set.weight_left ?? set.weight;
		const right = set.weight_right ?? set.weight;
		return left + right;
	}

	function totalVolumeKg(w: WorkoutWithExercises): number {
		return w.exercises.reduce((total, { exercise, sets }) => {
			const perSideWeight = exercise.per_side_weight ?? false;
			const splitWeight = exercise.split_weight ?? false;
			return (
				total +
				sets.reduce(
					(sum, set) => sum + set.reps * setTotalWeight(set, perSideWeight, splitWeight),
					0
				)
			);
		}, 0);
	}

	function totalSets(w: WorkoutWithExercises): number {
		return w.exercises.reduce((count, { sets }) => count + sets.length, 0);
	}

	function toUiSets(sets: Set[]) {
		return sets.map((s) => ({
			reps: s.reps,
			weight: s.weight,
			weightLeft: s.weight_left,
			weightRight: s.weight_right,
			durationSeconds: s.duration_seconds
		}));
	}

	function avgExerciseDurationMinutes(w: WorkoutWithExercises): number | null {
		const duration = durationMinutes(w.start_time, w.end_time);
		if (duration === null) return null;
		const count = w.exercises.length;
		if (count <= 0) return null;
		return Math.round(duration / count);
	}
</script>

<div class="card variant-glass-surface p-4 min-w-0">
	<header class="flex items-start justify-between gap-3">
		<div class="min-w-0">
			<h2 class="text-lg font-semibold tracking-tight">{title}</h2>
			{#if subtitle}
				<p class="text-sm opacity-70">{subtitle}</p>
			{/if}
		</div>
		<div class="flex items-center gap-2">
			<slot name="actions" />
		</div>
	</header>

	{#if error}
		<div class="mt-4 alert variant-filled-error">{error}</div>
	{:else if loading}
		<div class="mt-4 space-y-3 animate-pulse">
			<div class="h-4 w-36 bg-surface-200/60 dark:bg-surface-700/50 rounded"></div>
			<div class="h-24 bg-surface-200/60 dark:bg-surface-700/50 rounded-xl"></div>
			<div class="h-24 bg-surface-200/60 dark:bg-surface-700/50 rounded-xl"></div>
		</div>
	{:else if !workout}
		<div class="mt-4 card variant-ghost p-4 text-center opacity-80">
			Select a session to see exercises and set schemes.
		</div>
	{:else}
		<div class="mt-4 space-y-4 min-w-0">
			<div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3">
				<div class="card variant-glass-surface p-3 border-l-4 border-primary-500/70">
					<div class="text-xs font-semibold opacity-70">When</div>
					<div class="text-sm font-bold truncate">{formatDateRelative(workout.start_time)}</div>
				</div>
				<div class="card variant-glass-surface p-3 border-l-4 border-secondary-500/70">
					<div class="text-xs font-semibold opacity-70">Duration</div>
					<div class="text-sm font-bold">
						{durationMinutes(workout.start_time, workout.end_time) ?? '—'}m
					</div>
				</div>
				<div class="card variant-glass-surface p-3 border-l-4 border-warning-500/70">
					<div class="text-xs font-semibold opacity-70">Avg / exercise</div>
					<div class="text-sm font-bold">{avgExerciseDurationMinutes(workout) ?? '—'}m</div>
				</div>
				<div
					class="card variant-glass-surface p-3 border-l-4 border-surface-400/70 dark:border-surface-600/70"
				>
					<div class="text-xs font-semibold opacity-70">Exercises</div>
					<div class="text-sm font-bold">{workout.exercises.length}</div>
				</div>
				<div class="card variant-glass-surface p-3 border-l-4 border-tertiary-500/70">
					<div class="text-xs font-semibold opacity-70">Sets</div>
					<div class="text-sm font-bold">{totalSets(workout)}</div>
				</div>
				<div class="card variant-glass-surface p-3 border-l-4 border-success-500/70">
					<div class="text-xs font-semibold opacity-70">Volume</div>
					<div class="text-sm font-bold">{Math.round(totalVolumeKg(workout))} kg</div>
				</div>
			</div>

			<div class="flex items-center justify-between gap-3">
				<div class="text-sm opacity-75">
					{formatTime(workout.start_time)} - {formatTime(workout.end_time)}
				</div>
				<div class="flex items-center gap-2">
					{#if workout.auto_closed_at}
						<span class="badge variant-soft-warning text-xs">Auto-closed</span>
					{/if}
					{#if workout.feedback}
						<span class="text-2xl" aria-label="Session mood">{workout.feedback}</span>
					{/if}
				</div>
			</div>

			{#if workout.auto_closed_at}
				<div class="text-sm opacity-70">
					Auto-closed due to inactivity — adjust times if needed.
				</div>
			{/if}

			{#if workout.notes}
				<div
					class="rounded-xl border border-surface-200/50 bg-surface-50/60 p-3 dark:border-surface-700/50 dark:bg-surface-950/30"
				>
					<div class="text-xs font-semibold opacity-70">Notes</div>
					<div class="text-sm mt-1">{workout.notes}</div>
				</div>
			{/if}

			<div class="space-y-3 min-w-0">
				{#if workout.exercises.length === 0}
					<div class="card variant-ghost p-4 text-center opacity-80">No exercises recorded.</div>
				{:else}
					{#each workout.exercises as { exercise, sets } (exercise.id)}
						<article
							class="rounded-2xl border border-surface-200/50 bg-surface-50/60 p-4 dark:border-surface-700/50 dark:bg-surface-950/30 min-w-0"
						>
							<header class="flex items-start justify-between gap-3">
								<div class="min-w-0">
									<h3 class="text-base font-semibold truncate">{exercise.exercise_type}</h3>
								</div>
								<div class="text-sm opacity-70 whitespace-nowrap">{sets.length} sets</div>
							</header>

							{#if exercise.settings?.length}
								<div class="mt-2 flex flex-wrap gap-1">
									{#each exercise.settings as s (s.id)}
										<span class="badge variant-soft text-xs">{s.key}: {s.value}</span>
									{/each}
								</div>
							{/if}

							{#if sets.length}
								<div class="mt-3">
									<SetPillsHybrid
										sets={toUiSets(sets)}
										perSideWeight={exercise.per_side_weight ?? false}
										splitWeight={exercise.split_weight ?? false}
										size="xs"
									/>
								</div>
							{:else}
								<div class="mt-3 text-sm opacity-70">No sets recorded.</div>
							{/if}

							{#if exercise.notes}
								<div class="mt-3 text-sm opacity-75">Notes: {exercise.notes}</div>
							{/if}
						</article>
					{/each}
				{/if}
			</div>
		</div>
	{/if}
</div>
