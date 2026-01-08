<script lang="ts">
	import type { WorkoutWithExercises } from '$lib/types';
	import WorkoutDetailsCard from '$lib/components/history/WorkoutDetailsCard.svelte';

	export let data: { workout: WorkoutWithExercises | null; error: string | null };
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
				<h1 class="text-3xl sm:text-4xl font-black tracking-tight">Workout</h1>
				<p class="text-sm sm:text-base opacity-80 max-w-prose">Full session details.</p>
			</div>
			<div class="flex sm:justify-end">
				<a href="/workouts" class="btn variant-soft">← Back to History</a>
			</div>
		</div>
	</header>

	<WorkoutDetailsCard
		workout={data.workout}
		loading={!data.workout && !data.error}
		error={data.error}
	/>
</div>
