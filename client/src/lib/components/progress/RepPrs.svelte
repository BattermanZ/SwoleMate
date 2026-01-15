<script lang="ts">
	import { summarizeRepPrs, type RepPr } from '$lib/progress/repPrs';

	export let repPrs: RepPr[] = [];

	let showAll = false;
	$: summarized = summarizeRepPrs(repPrs);
	$: if (!summarized.length) showAll = false;
</script>

{#if summarized.length}
	<div
		class="rounded-xl border border-surface-200/50 bg-surface-50/60 p-3 dark:border-surface-700/50 dark:bg-surface-950/30"
	>
		<div class="flex items-center justify-between gap-3">
			<div class="text-sm font-semibold opacity-80">Rep PRs</div>
			{#if summarized.length > 10}
				<button
					type="button"
					class="btn variant-ghost text-xs"
					on:click={() => (showAll = !showAll)}
				>
					{showAll ? 'Show less' : `Show all (${summarized.length})`}
				</button>
			{/if}
		</div>

		<div class="mt-2 flex flex-wrap gap-2">
			{#each showAll ? summarized : summarized.slice(0, 10) as pr (pr.reps)}
				<span
					class="inline-flex overflow-hidden rounded-full border border-surface-200/50 dark:border-surface-700/60"
				>
					<span
						class="px-2 py-1 text-xs font-extrabold tracking-tight bg-secondary-500/25 text-surface-950 dark:text-surface-50"
						>{pr.reps} reps</span
					>
					<span
						class="px-2 py-1 text-xs font-extrabold tracking-tight bg-primary-500/30 text-surface-950 dark:text-surface-50 border-l border-surface-200/50 dark:border-surface-700/60"
						>{pr.weight}kg</span
					>
				</span>
			{/each}
		</div>
	</div>
{/if}
