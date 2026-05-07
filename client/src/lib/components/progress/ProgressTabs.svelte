<script lang="ts" context="module">
	export type ProgressTab = 'overview' | 'exercise' | 'trends';
</script>

<script lang="ts">
	import { createEventDispatcher } from 'svelte';

	export let selectedTab: ProgressTab = 'overview';

	const dispatch = createEventDispatcher<{ select: ProgressTab }>();

	const tabs: Array<{ id: ProgressTab; label: string }> = [
		{ id: 'overview', label: 'Overview' },
		{ id: 'exercise', label: 'Exercise' },
		{ id: 'trends', label: 'Trends' }
	];

	function select(tab: ProgressTab) {
		dispatch('select', tab);
	}
</script>

<div class="flex w-full sm:w-auto" role="tablist" aria-label="Progress sections">
	<div
		class="grid w-full grid-cols-3 rounded-lg border border-surface-200/70 bg-surface-50/70 p-1 dark:border-surface-700/70 dark:bg-surface-950/40 sm:w-auto"
	>
		{#each tabs as tab}
			<button
				type="button"
				role="tab"
				aria-selected={selectedTab === tab.id}
				class="rounded-md px-3 py-2 text-sm font-semibold transition-colors {selectedTab === tab.id
					? 'bg-primary-500 text-white shadow-sm'
					: 'text-surface-700 hover:bg-surface-100 dark:text-surface-200 dark:hover:bg-surface-800/70'}"
				on:click={() => select(tab.id)}
			>
				{tab.label}
			</button>
		{/each}
	</div>
</div>
