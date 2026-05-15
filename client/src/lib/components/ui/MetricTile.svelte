<script lang="ts">
	import type { Snippet } from 'svelte';

	type Rail = 'clay' | 'warn' | 'sage' | 'gold' | 'ink';
	type Delta = 'up' | 'down' | 'neutral';

	interface Props {
		label: string;
		value: Snippet | string;
		unit?: string;
		delta?: Snippet | string;
		deltaTone?: Delta;
		rail?: Rail;
	}

	let { label, value, unit, delta, deltaTone = 'neutral', rail = 'clay' }: Props = $props();
</script>

<div class="tile rail-{rail}">
	<div class="k">{label}</div>
	<div class="v">
		{#if typeof value === 'string'}{value}{:else}{@render value()}{/if}
		{#if unit}<small>{unit}</small>{/if}
	</div>
	{#if delta !== undefined && delta !== ''}
		<div class="d t-{deltaTone}">
			{#if typeof delta === 'string'}{delta}{:else}{@render delta()}{/if}
		</div>
	{/if}
</div>

<style>
	.tile {
		position: relative;
		overflow: hidden;
		background: var(--card);
		border: 1px solid var(--line);
		border-radius: 12px;
		padding: 10px 12px 10px 16px;
		min-width: 0;
	}
	.tile::before {
		content: '';
		position: absolute;
		top: 8px;
		bottom: 8px;
		left: 0;
		width: 3px;
		border-radius: 0 3px 3px 0;
	}
	.rail-clay::before {
		background: var(--clay);
	}
	.rail-warn::before {
		background: var(--warn);
	}
	.rail-sage::before {
		background: var(--sage);
	}
	.rail-gold::before {
		background: var(--gold);
	}
	.rail-ink::before {
		background: var(--ink);
	}

	.k {
		font: 700 9px/1 'Onest', system-ui, sans-serif;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--ink-soft);
	}
	.v {
		margin-top: 5px;
		font: 800 19px/1 'Onest', system-ui, sans-serif;
		letter-spacing: -0.02em;
		font-variant-numeric: tabular-nums;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		display: flex;
		align-items: baseline;
		gap: 4px;
	}
	.v small {
		font: 500 10px/1 'Onest', system-ui, sans-serif;
		color: var(--ink-soft);
		font-weight: 600;
	}
	.d {
		margin-top: 3px;
		font: 600 10px/1 'Onest', system-ui, sans-serif;
		color: var(--ink-soft);
	}
	.t-up {
		color: var(--sage);
	}
	.t-down {
		color: var(--clay-text);
	}
</style>
