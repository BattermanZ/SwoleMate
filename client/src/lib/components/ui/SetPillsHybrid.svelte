<script lang="ts">
	type SetLike = { reps: number; weight: number; weightLeft?: number; weightRight?: number };
	type SetGroup = {
		reps: number;
		weightLabel: string;
		totalWeight: number;
		count: number;
	};

	export let sets: SetLike[] = [];
	export let perSideWeight = false;
	export let splitWeight = false;
	export let size: 'xs' | 'sm' | 'md' = 'sm';

	function setTotalWeight(set: SetLike): number {
		if (!perSideWeight) return set.weight;
		if (!splitWeight) return set.weight * 2;
		const left = set.weightLeft ?? set.weight;
		const right = set.weightRight ?? set.weight;
		return left + right;
	}

	function formatWeight(set: SetLike): string {
		if (!perSideWeight) return `${set.weight}kg`;
		if (!splitWeight) return `${set.weight}kg/side`;
		const left = set.weightLeft ?? set.weight;
		const right = set.weightRight ?? set.weight;
		return left === right ? `${left}kg/side` : `${left}/${right}kg`;
	}

	function groupSets(items: SetLike[]): SetGroup[] {
		const groups: SetGroup[] = [];
		for (const set of items) {
			const weightLabel = formatWeight(set);
			const key = `${set.reps}×${weightLabel}`;
			const existing = groups.find((g) => `${g.reps}×${g.weightLabel}` === key);
			if (existing) {
				existing.count += 1;
				continue;
			}
			groups.push({
				reps: set.reps,
				weightLabel,
				totalWeight: setTotalWeight(set),
				count: 1
			});
		}
		return groups;
	}

	$: groups = groupSets(sets);
	$: totals = groups.map((g) => g.totalWeight);
	$: minTotal = totals.length ? Math.min(...totals) : 0;
	$: maxTotal = totals.length ? Math.max(...totals) : 0;

	function weightIntensity(total: number): number {
		if (!totals.length) return 0.6;
		if (minTotal === maxTotal) return 0.65;
		const t = (total - minTotal) / (maxTotal - minTotal);
		return 0.38 + t * 0.47;
	}

	function pct(n: number): string {
		return `${Math.round(n * 100)}%`;
	}
</script>

<div class="set-pills" data-size={size}>
	{#each groups as g}
		<span class="set-pill" style={`--w:${pct(weightIntensity(g.totalWeight))}`}>
			{#if g.count > 1}
				<span class="set-pill__count">{g.count}×</span>
			{/if}
			<span class="set-pill__reps">{g.reps}×</span>
			<span class="set-pill__weight set-pill__weight--scale">{g.weightLabel}</span>
		</span>
	{/each}
</div>

<style>
	.set-pills {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
		min-width: 0;
	}

	.set-pill {
		display: inline-flex;
		align-items: stretch;
		overflow: hidden;
		border-radius: 9999px;
		border: 1px solid color-mix(in oklab, var(--color-surface-950-50) 16%, transparent);
		box-shadow: 0 10px 24px color-mix(in oklab, var(--color-surface-950) 12%, transparent);
	}

	.set-pill__count,
	.set-pill__reps,
	.set-pill__weight {
		font-weight: 800;
		letter-spacing: -0.01em;
		line-height: 1;
		white-space: nowrap;
	}

	.set-pill__count {
		background-color: var(--color-tertiary-500);
		color: var(--color-tertiary-contrast-500);
	}

	.set-pill__reps {
		background-color: color-mix(
			in oklab,
			var(--color-secondary-500) 26%,
			var(--color-surface-50-950)
		);
		color: var(--color-surface-950-50);
		border-left: 1px solid color-mix(in oklab, var(--color-surface-950-50) 14%, transparent);
	}

	.set-pill__weight {
		color: var(--color-surface-950-50);
		border-left: 1px solid color-mix(in oklab, var(--color-surface-950-50) 14%, transparent);
	}

	.set-pill__weight--scale {
		background-color: color-mix(
			in oklab,
			var(--color-primary-500) var(--w),
			var(--color-surface-50-950)
		);
	}

	.set-pills[data-size='xs'] .set-pill__count,
	.set-pills[data-size='xs'] .set-pill__reps,
	.set-pills[data-size='xs'] .set-pill__weight {
		padding: 0.22rem 0.5rem;
		font-size: 0.72rem;
	}

	.set-pills[data-size='xs'] {
		gap: 0.35rem;
	}

	.set-pills[data-size='sm'] .set-pill__count,
	.set-pills[data-size='sm'] .set-pill__reps,
	.set-pills[data-size='sm'] .set-pill__weight {
		padding: 0.26rem 0.6rem;
		font-size: 0.78rem;
	}

	.set-pills[data-size='sm'] {
		gap: 0.45rem;
	}

	.set-pills[data-size='md'] .set-pill__count,
	.set-pills[data-size='md'] .set-pill__reps,
	.set-pills[data-size='md'] .set-pill__weight {
		padding: 0.32rem 0.7rem;
		font-size: 0.86rem;
	}

	.set-pill__count {
		padding-left: 0.55rem;
		padding-right: 0.55rem;
	}
</style>
