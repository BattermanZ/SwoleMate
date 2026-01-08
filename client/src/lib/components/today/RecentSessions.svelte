<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { formatDateRelative, formatTime } from '$lib/utils/date';
	import type { UiSession } from '$lib/today/types';
	import SetPillsHybrid from '$lib/components/ui/SetPillsHybrid.svelte';

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

	function durationMinutes(session: UiSession): number | null {
		if (!session.endedAt) return null;
		const start = new Date(session.startedAt).getTime();
		const end = new Date(session.endedAt).getTime();
		const diff = Math.max(0, end - start);
		return Math.round(diff / 60_000);
	}
</script>

<section class="space-y-3 min-w-0">
	<header class="flex items-end justify-between gap-2">
		<div>
			<h2 class="text-lg font-semibold tracking-tight">Past 2 Sessions</h2>
			<p class="text-sm opacity-70">Quick recall for exercise order + notes.</p>
		</div>
	</header>

	{#if sessions.length === 0}
		<div class="card variant-ghost p-4 text-center opacity-80">No sessions yet.</div>
	{:else}
		<div class="space-y-3 min-w-0">
			{#each sessions as session (session.id)}
				<article class="card variant-glass-surface p-4 space-y-3 min-w-0">
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
										<div class="mt-2">
											<SetPillsHybrid
												sets={ex.sets}
												perSideWeight={ex.perSideWeight}
												splitWeight={ex.splitWeight}
												size="xs"
											/>
										</div>
										{#if ex.notes}
											<div class="mt-2 text-sm opacity-75">Notes: {ex.notes}</div>
										{/if}
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
