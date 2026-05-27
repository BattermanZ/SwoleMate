<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		title: string;
		subtitle?: string;
		/** Right-aligned per-screen actions (buttons, etc.). */
		actions?: Snippet;
	}
	let { title, subtitle, actions }: Props = $props();
</script>

<header class="desktop-topbar">
	<h1 class="title">
		{title}{#if subtitle} — <em>{subtitle}</em>{/if}
	</h1>
	{#if actions}
		<div class="actions">{@render actions()}</div>
	{/if}
</header>

<style>
	.desktop-topbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 16px 24px;
		border-bottom: 1px solid var(--line);
		background: color-mix(in oklab, var(--bg) 86%, transparent);
		backdrop-filter: blur(12px);
	}
	.title {
		margin: 0;
		font: 800 20px/1.05 'Onest', system-ui, sans-serif;
		letter-spacing: -0.02em;
		color: var(--ink);
	}
	.title em {
		font: italic 400 16px/1 'Instrument Serif', serif;
		color: var(--ink-soft);
		margin-left: 6px;
	}
	.actions {
		display: flex;
		align-items: center;
		gap: 8px;
		flex: none;
	}
</style>
