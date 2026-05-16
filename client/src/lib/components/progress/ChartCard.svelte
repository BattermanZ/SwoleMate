<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Card } from '$lib/components/ui';

	interface Props {
		headline: string;
		titleEm?: string;
		legend?: Snippet;
		children?: Snippet;
		footnote?: Snippet | string;
		height?: number;
	}
	let { headline, titleEm, legend, children, footnote, height = 220 }: Props = $props();
</script>

<Card>
	{#snippet title()}{headline}{#if titleEm}
			<em>— {titleEm}</em>{/if}{/snippet}
	{#snippet actions()}
		{#if legend}<div class="legend">{@render legend()}</div>{/if}
	{/snippet}

	<div class="canvas-wrap" style="height: {height}px;">
		{@render children?.()}
	</div>

	{#if footnote}
		<div class="foot">
			{#if typeof footnote === 'string'}{footnote}{:else}{@render footnote()}{/if}
		</div>
	{/if}
</Card>

<style>
	.canvas-wrap {
		position: relative;
		width: 100%;
	}
	.canvas-wrap :global(canvas) {
		width: 100% !important;
		height: 100% !important;
	}
	.foot {
		margin-top: 10px;
		font: italic 400 12px/1.4 'Instrument Serif';
		color: var(--ink-2);
	}
	.foot :global(b) {
		font-weight: 700;
		color: var(--ink);
		font-style: normal;
		font-family: 'Onest', system-ui, sans-serif;
	}
	.legend {
		display: flex;
		gap: 8px;
		align-items: center;
	}
	.legend :global(.dot) {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		font:
			700 9px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.12em;
		text-transform: uppercase;
		color: var(--ink-soft);
	}
	.legend :global(.dot::before) {
		content: '';
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--dot, var(--clay));
	}
</style>
