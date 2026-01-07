<script lang="ts">
	import { createEventDispatcher } from 'svelte';

	type SetEntry = { reps: number; weight: number };
	type SetGroup = { reps: number; weight: number; count: number };

	export let selectedOption: 1 | 2 | 3 | 4 | 5 = 1;

	const dispatch = createEventDispatcher<{ select: { option: 1 | 2 | 3 | 4 | 5 } }>();

	const sampleSets: SetEntry[] = [
		{ reps: 12, weight: 62 },
		{ reps: 12, weight: 62 },
		{ reps: 12, weight: 54 },
		{ reps: 12, weight: 54 },
		{ reps: 12, weight: 59 },
		{ reps: 12, weight: 41 },
		{ reps: 12, weight: 41 },
		{ reps: 12, weight: 43 }
	];

	function groupSets(sets: SetEntry[]): SetGroup[] {
		const groups: SetGroup[] = [];
		for (const set of sets) {
			const existing = groups.find((g) => g.reps === set.reps && g.weight === set.weight);
			if (existing) existing.count += 1;
			else groups.push({ ...set, count: 1 });
		}
		return groups;
	}

	const groups = groupSets(sampleSets);
	const weights = groups.map((g) => g.weight);
	const minWeight = Math.min(...weights);
	const maxWeight = Math.max(...weights);

	function weightIntensity(weight: number): number {
		if (minWeight === maxWeight) return 0.6;
		const t = (weight - minWeight) / (maxWeight - minWeight);
		return 0.25 + t * 0.65;
	}

	function pct(n: number): string {
		return `${Math.round(n * 100)}%`;
	}

	function select(option: 1 | 2 | 3 | 4 | 5) {
		dispatch('select', { option });
	}
</script>

<section class="card variant-glass-surface p-4 space-y-4">
	<header class="space-y-1">
		<h3 class="text-lg font-semibold tracking-tight">Set chips — style options</h3>
		<p class="text-sm opacity-75">
			Same data, five visual systems. The goal is to keep repeated sets grouped while making reps vs
			weight scannable.
		</p>
	</header>

	<div class="grid gap-4 lg:grid-cols-2">
		<button
			type="button"
			class="option"
			data-selected={selectedOption === 1}
			on:click={() => select(1)}
		>
			<div class="text-sm font-semibold opacity-80">1) Segmented pill (count + reps + weight)</div>
			<div class="mt-2 flex flex-wrap gap-2">
				{#each groups as g}
					<span class="set-pill">
						{#if g.count > 1}
							<span class="set-pill__count">{g.count}×</span>
						{/if}
						<span class="set-pill__reps">{g.reps}</span>
						<span class="set-pill__weight">{g.weight}kg</span>
					</span>
				{/each}
			</div>
		</button>

		<button
			type="button"
			class="option"
			data-selected={selectedOption === 5}
			on:click={() => select(5)}
		>
			<div class="text-sm font-semibold opacity-80">5) Segmented + weight intensity (hybrid)</div>
			<div class="mt-2 flex flex-wrap gap-2">
				{#each groups as g}
					<span class="set-pill" style={`--w:${pct(weightIntensity(g.weight))}`}>
						{#if g.count > 1}
							<span class="set-pill__count">{g.count}×</span>
						{/if}
						<span class="set-pill__reps">{g.reps}</span>
						<span class="set-pill__weight set-pill__weight--scale">{g.weight}kg</span>
					</span>
				{/each}
			</div>
		</button>

		<button
			type="button"
			class="option"
			data-selected={selectedOption === 2}
			on:click={() => select(2)}
		>
			<div class="text-sm font-semibold opacity-80">2) Chip + count badge</div>
			<div class="mt-2 flex flex-wrap gap-2">
				{#each groups as g}
					<span class="relative inline-flex">
						<span class="chip variant-filled text-sm chip-split">
							<span class="chip-split__reps">{g.reps}</span>
							<span class="chip-split__weight">{g.weight}kg</span>
						</span>
						{#if g.count > 1}
							<span class="badge variant-filled-secondary text-xs count-badge">{g.count}×</span>
						{/if}
					</span>
				{/each}
			</div>
		</button>

		<button
			type="button"
			class="option"
			data-selected={selectedOption === 3}
			on:click={() => select(3)}
		>
			<div class="text-sm font-semibold opacity-80">
				3) Weight-intensity scale (heavier = stronger)
			</div>
			<div class="mt-2 flex flex-wrap gap-2">
				{#each groups as g}
					<span class="chip text-sm weight-scale" style={`--w:${pct(weightIntensity(g.weight))}`}>
						{#if g.count > 1}
							<span class="opacity-80 font-semibold mr-1">{g.count}×</span>
						{/if}
						<span class="opacity-80">{g.reps}×</span>
						<span class="font-semibold">{g.weight}kg</span>
					</span>
				{/each}
			</div>
		</button>

		<button
			type="button"
			class="option"
			data-selected={selectedOption === 4}
			on:click={() => select(4)}
		>
			<div class="text-sm font-semibold opacity-80">4) Two-row (reps row + weight row)</div>
			<div class="mt-2 space-y-2">
				<div class="flex flex-wrap gap-2">
					{#each groups as g}
						<span class="badge variant-soft text-xs">
							{#if g.count > 1}{g.count}×
							{/if}{g.reps} reps
						</span>
					{/each}
				</div>
				<div class="flex flex-wrap gap-2">
					{#each groups as g}
						<span class="badge variant-filled-primary text-xs">
							{#if g.count > 1}{g.count}×
							{/if}{g.weight}kg
						</span>
					{/each}
				</div>
			</div>
		</button>
	</div>
</section>

<style>
	.option {
		text-align: left;
		padding: 0.75rem;
		border-radius: 1rem;
		background: color-mix(in oklab, var(--color-surface-50-950) 88%, transparent);
		border: 1px solid color-mix(in oklab, var(--color-surface-950-50) 14%, transparent);
		box-shadow: 0 14px 36px color-mix(in oklab, var(--color-surface-950) 10%, transparent);
	}

	.option[data-selected='true'] {
		border-color: color-mix(in oklab, var(--color-primary-500) 55%, transparent);
		box-shadow: 0 18px 52px color-mix(in oklab, var(--color-primary-500) 14%, transparent);
	}

	.option:focus-visible {
		outline: 2px solid color-mix(in oklab, var(--color-primary-500) 80%, transparent);
		outline-offset: 2px;
	}

	.set-pill {
		display: inline-flex;
		align-items: stretch;
		overflow: hidden;
		border-radius: 9999px;
		border: 1px solid color-mix(in oklab, var(--color-surface-950-50) 16%, transparent);
		box-shadow: 0 10px 24px color-mix(in oklab, var(--color-surface-950) 12%, transparent);
	}

	.set-pill__count {
		padding: 0.25rem 0.5rem;
		background-color: var(--color-secondary-500);
		color: var(--color-secondary-contrast-500);
		font-weight: 700;
		font-size: 0.75rem;
		line-height: 1;
	}

	.set-pill__reps {
		padding: 0.25rem 0.6rem;
		background-color: color-mix(
			in oklab,
			var(--color-secondary-500) 25%,
			var(--color-surface-50-950)
		);
		color: var(--color-surface-950-50);
		font-weight: 700;
		font-size: 0.75rem;
		line-height: 1;
	}

	.set-pill__weight {
		padding: 0.25rem 0.6rem;
		background-color: color-mix(
			in oklab,
			var(--color-primary-500) 35%,
			var(--color-surface-50-950)
		);
		color: var(--color-surface-950-50);
		font-weight: 700;
		font-size: 0.75rem;
		line-height: 1;
	}

	.set-pill__weight--scale {
		background-color: color-mix(
			in oklab,
			var(--color-primary-500) var(--w),
			var(--color-surface-50-950)
		);
		color: var(--color-surface-950-50);
	}

	.chip-split {
		padding: 0;
		overflow: hidden;
		border-radius: 0.75rem;
	}

	.chip-split__reps {
		padding: 0.25rem 0.6rem;
		background-color: color-mix(
			in oklab,
			var(--color-secondary-500) 22%,
			var(--color-surface-50-950)
		);
	}

	.chip-split__weight {
		padding: 0.25rem 0.6rem;
		background-color: color-mix(
			in oklab,
			var(--color-primary-500) 28%,
			var(--color-surface-50-950)
		);
	}

	.count-badge {
		position: absolute;
		top: -0.5rem;
		left: -0.5rem;
		border-radius: 9999px;
	}

	.weight-scale {
		border: 1px solid color-mix(in oklab, var(--color-surface-950-50) 16%, transparent);
		background-color: color-mix(
			in oklab,
			var(--color-primary-500) var(--w),
			var(--color-surface-50-950)
		);
		color: var(--color-surface-950-50);
	}
</style>
