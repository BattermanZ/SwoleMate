<script lang="ts">
	import type { ExerciseProgress } from '$lib/types';
	import SetPillsHybrid from '$lib/components/ui/SetPillsHybrid.svelte';
	import { formatDateRelative, formatTime } from '$lib/utils/date';

	export let exerciseProgress: ExerciseProgress[] | null = null;

	$: sessions = (exerciseProgress ?? []).slice(-5).reverse();
</script>

<div class="card variant-glass-surface p-4">
	<div class="flex items-start justify-between gap-3">
		<div>
			<h3 class="text-base font-semibold">Last 5 sessions</h3>
			<p class="text-sm opacity-70">Set patterns for this exercise.</p>
		</div>
	</div>

	{#if sessions.length}
		<div class="mt-3 space-y-3">
			{#each sessions as session (session.exercise.id)}
				<div
					class="rounded-xl border border-surface-200/50 bg-surface-50/60 p-3 dark:border-surface-700/50 dark:bg-surface-950/30"
				>
					<div class="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1">
						<div class="font-semibold">{formatDateRelative(session.exercise.start_time)}</div>
						<div class="text-xs opacity-70">{formatTime(session.exercise.start_time)}</div>
					</div>

					{#if session.exercise.notes}
						<div class="mt-1 text-sm opacity-80">Notes: {session.exercise.notes}</div>
					{/if}

					<div class="mt-2">
						<SetPillsHybrid
							sets={session.sets.map((s) => ({
								reps: s.reps,
								weight: s.weight,
								weightLeft: s.weight_left,
								weightRight: s.weight_right,
								durationSeconds: s.duration_seconds
							}))}
							perSideWeight={session.exercise.per_side_weight ?? false}
							splitWeight={session.exercise.split_weight ?? false}
							size="xs"
						/>
					</div>
				</div>
			{/each}
		</div>
	{:else}
		<div class="mt-3 text-sm opacity-70">No sessions yet for this exercise.</div>
	{/if}
</div>
