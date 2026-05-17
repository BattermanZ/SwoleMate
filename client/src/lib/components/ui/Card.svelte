<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		children?: Snippet;
		title?: Snippet;
		lede?: Snippet;
		actions?: Snippet;
		as?: 'section' | 'article' | 'div';
		padded?: boolean;
		class?: string;
	}

	let {
		children,
		title,
		lede,
		actions,
		as = 'section',
		padded = true,
		class: cls = ''
	}: Props = $props();
</script>

<svelte:element this={as} class="card {cls}" class:padded>
	{#if title || actions}
		<header class="card-head">
			<div class="card-head-text">
				{#if title}<h2 class="card-title">{@render title()}</h2>{/if}
				{#if lede}<p class="card-lede">{@render lede()}</p>{/if}
			</div>
			{#if actions}<div class="card-actions">{@render actions()}</div>{/if}
		</header>
	{/if}
	{@render children?.()}
</svelte:element>

<style>
	.card {
		background: var(--card);
		border: 1px solid var(--line);
		border-radius: 22px;
		box-shadow: 0 6px 18px -10px var(--shadow-card);
	}
	.card.padded {
		padding: 16px;
	}

	.card-head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 10px;
		margin-bottom: 12px;
	}
	.card-head-text {
		min-width: 0;
	}
	.card-title {
		margin: 0;
		font: 800 17px/1.1 'Onest';
		letter-spacing: -0.015em;
		color: var(--ink);
	}
	.card-title :global(em) {
		font: italic 400 14px/1 'Instrument Serif';
		color: var(--ink-soft);
		font-weight: 400;
		margin-left: 4px;
		letter-spacing: 0;
	}
	.card-lede {
		margin: 4px 0 0;
		font: 500 12px/1.4 'Onest';
		color: var(--ink-soft);
	}
	.card-actions {
		display: flex;
		gap: 6px;
		flex: none;
	}
</style>
