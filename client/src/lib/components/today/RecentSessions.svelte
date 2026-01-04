<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { formatDateRelative, formatTime } from '$lib/utils/date';
	import type { UiSession } from '$lib/mocks/today';

	export let sessions: UiSession[] = [];
	export let canAdd = false;
	export let disabled = false;

	const dispatch = createEventDispatcher<{
		addExercise: {
			name: string;
			notes?: string;
			perSideWeight?: boolean;
			splitWeight?: boolean;
			settings?: Array<{ key: string; value: string }>;
		};
	}>();

	function setLabel(
		exercise: UiSession['exercises'][number],
		reps: number,
		set: { weight: number; weightLeft?: number; weightRight?: number }
	) {
		if (!exercise.perSideWeight) return `${reps}×${set.weight}kg`;
		if (!exercise.splitWeight) return `${reps}×${set.weight}kg/side`;
		const left = set.weightLeft ?? set.weight;
		const right = set.weightRight ?? set.weight;
		return left === right ? `${reps}×${left}kg/side` : `${reps}×${left}/${right}kg`;
	}

	function compressSetLabels(
		exercise: UiSession['exercises'][number]
	): Array<{ count: number; label: string }> {
		const compressed: Array<{ count: number; label: string }> = [];
		for (const set of exercise.sets) {
			const label = setLabel(exercise, set.reps, set);
			const existing = compressed.find((c) => c.label === label);
			if (existing) existing.count += 1;
			else compressed.push({ count: 1, label });
		}
		return compressed;
	}

	function durationMinutes(session: UiSession): number | null {
		if (!session.endedAt) return null;
		const start = new Date(session.startedAt).getTime();
		const end = new Date(session.endedAt).getTime();
		const diff = Math.max(0, end - start);
		return Math.round(diff / 60_000);
	}
</script>

<section class="space-y-3">
	<header class="flex items-end justify-between gap-2">
		<div>
			<h2 class="text-lg font-semibold tracking-tight">Past 2 Sessions</h2>
			<p class="text-sm opacity-70">Quick recall for exercise order + notes.</p>
		</div>
	</header>

	{#if sessions.length === 0}
		<div class="card variant-ghost p-4 text-center opacity-80">No sessions yet.</div>
	{:else}
		<div class="space-y-3">
			{#each sessions as session (session.id)}
				<article class="card variant-glass-surface p-4 space-y-3">
					<header class="flex items-start justify-between gap-3">
						<div>
							<div class="flex items-center gap-2">
								<h3 class="text-base font-semibold">{formatDateRelative(session.startedAt)}</h3>
								{#if session.mood}
									<span class="text-xl" aria-label="Session mood">{session.mood}</span>
								{/if}
							</div>
							<div class="text-sm opacity-75">
								{formatTime(session.startedAt)}
								{#if session.endedAt}
									- {formatTime(session.endedAt)}
								{/if}
								{#if durationMinutes(session) !== null}
									<span class="opacity-60"> • </span>
									{durationMinutes(session)}m
								{/if}
							</div>
						</div>
					</header>

					{#if session.notes}
						<p class="text-sm opacity-80">{session.notes}</p>
					{/if}

					<div class="space-y-2">
						{#each session.exercises as ex (ex.id)}
							<div
								class="rounded-xl border border-surface-200/50 bg-surface-50/60 p-3 dark:border-surface-700/50 dark:bg-surface-950/30"
							>
								<div class="flex items-start justify-between gap-3">
									<div class="min-w-0">
										<div class="font-semibold truncate">{ex.name}</div>
										{#if ex.notes}
											<div class="text-sm opacity-75 truncate">{ex.notes}</div>
										{/if}
										{#if ex.settings.length > 0}
											<div class="mt-1 flex flex-wrap gap-1 text-xs opacity-70">
												{#each ex.settings.slice(0, 2) as s}
													<span class="badge variant-soft text-xs">{s.key}: {s.value}</span>
												{/each}
												{#if ex.settings.length > 2}
													<span class="badge variant-soft text-xs">+{ex.settings.length - 2}</span>
												{/if}
											</div>
										{/if}
										<div class="mt-2 flex flex-wrap gap-1">
											{#each compressSetLabels(ex) as s}
												<span class="badge variant-filled-secondary text-xs"
													>{s.count}×{s.label}</span
												>
											{/each}
										</div>
									</div>

									{#if canAdd}
										<button
											type="button"
											class="btn btn-sm variant-filled-primary whitespace-nowrap"
											on:click={() =>
												dispatch('addExercise', {
													name: ex.name,
													notes: ex.notes,
													perSideWeight: ex.perSideWeight,
													splitWeight: ex.splitWeight,
													settings: ex.settings.map((s) => ({ key: s.key, value: s.value }))
												})}
											{disabled}
										>
											Add →
										</button>
									{/if}
								</div>
							</div>
						{/each}
					</div>
				</article>
			{/each}
		</div>
	{/if}
</section>
