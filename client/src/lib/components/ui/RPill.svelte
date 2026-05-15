<script lang="ts">
	// Always reps + weight, no count, saturated weight cell.
	// Used in Exercise focus → Rep PRs band.
	interface Props {
		reps: number;
		weight: string;
	}
	let { reps, weight }: Props = $props();
</script>

<span class="rpill">
	<span class="reps">{reps} {reps === 1 ? 'rep' : 'reps'}</span>
	<span class="weight">{weight}</span>
</span>

<style>
	.rpill {
		display: inline-flex;
		align-items: stretch;
		overflow: hidden;
		border-radius: 999px;
		border: 1px solid color-mix(in oklab, var(--ink) 10%, transparent);
		box-shadow: 0 2px 6px -2px var(--shadow-soft);
		font: 800 11px/1 'Onest', system-ui, sans-serif;
		font-variant-numeric: tabular-nums;
		vertical-align: middle;
	}
	.reps,
	.weight {
		padding: 5px 9px;
		display: inline-flex;
		align-items: center;
		white-space: nowrap;
	}
	.reps {
		background: var(--bg-2);
		color: var(--ink-2);
	}
	.weight {
		background: color-mix(in oklab, var(--clay) 75%, var(--bg-2));
		color: var(--ink);
		border-left: 1px solid color-mix(in oklab, var(--ink) 10%, transparent);
	}
</style>
