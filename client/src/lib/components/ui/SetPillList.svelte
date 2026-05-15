<script lang="ts">
	import Spill from './Spill.svelte';
	import { groupSets, type SetLike } from '$lib/today/setPills';

	interface Props {
		sets: SetLike[];
		perSideWeight?: boolean;
		splitWeight?: boolean;
		size?: 'sm' | 'xs';
		/** Index (within the grouped list) to highlight as a PR — null/undefined hides the highlight. */
		prGroupIndex?: number | null;
	}

	let {
		sets,
		perSideWeight = false,
		splitWeight = false,
		size = 'sm',
		prGroupIndex = null
	}: Props = $props();

	let groups = $derived(groupSets(sets, { perSideWeight, splitWeight }));
</script>

<div class="set-pills">
	{#each groups as g, i (i)}
		<Spill
			count={g.count > 1 ? g.count : undefined}
			reps={g.reps}
			duration={g.durationLabel}
			weight={g.weightLabel}
			intensity={g.intensity}
			bodyweight={g.bodyweight}
			pr={prGroupIndex === i}
			{size}
		/>
	{/each}
</div>

<style>
	.set-pills {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		align-items: center;
		min-width: 0;
	}
</style>
