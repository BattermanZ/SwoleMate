<script lang="ts">
	import type { Snippet } from 'svelte';
	import { isActive } from '$lib/components/shell/nav';

	export type NavItem = {
		href: string;
		label: string;
		icon: Snippet;
	};

	interface Props {
		items: NavItem[];
		current?: string;
		'aria-label'?: string;
	}

	let { items, current, 'aria-label': ariaLabel = 'Primary navigation' }: Props = $props();
</script>

<nav class="tabs" aria-label={ariaLabel}>
	{#each items as item (item.href)}
		{@const active = isActive(item.href, current)}
		<a href={item.href} class:active aria-current={active ? 'page' : undefined}>
			<span class="pill">{@render item.icon()}</span>
			<span class="lbl">{item.label}</span>
		</a>
	{/each}
</nav>

<style>
	.tabs {
		position: fixed;
		left: 16px;
		right: 16px;
		bottom: env(safe-area-inset-bottom);
		height: 68px;
		background: color-mix(in oklab, var(--card) 92%, transparent);
		backdrop-filter: blur(14px);
		border-radius: 28px;
		border: 1px solid var(--line);
		display: grid;
		grid-auto-flow: column;
		grid-auto-columns: 1fr;
		align-items: center;
		z-index: 25;
		box-shadow: 0 16px 32px -10px var(--shadow-card);
	}
	a {
		display: grid;
		place-items: center;
		gap: 4px;
		color: var(--ink-soft);
		text-decoration: none;
		font:
			700 10px/1 'Onest',
			system-ui,
			sans-serif;
		padding: 8px 0;
		transition: color 160ms ease;
	}
	.pill {
		width: 32px;
		height: 32px;
		border-radius: 16px;
		display: grid;
		place-items: center;
		background: var(--bg-2);
		color: currentColor;
		transition:
			background-color 160ms ease,
			box-shadow 200ms ease;
	}
	a.active {
		color: var(--clay);
	}
	a.active .pill {
		background: var(--clay);
		color: white;
		box-shadow: 0 6px 14px -6px var(--clay);
	}
	.lbl {
		max-width: 100%;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
</style>
