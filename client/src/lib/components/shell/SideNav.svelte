<script lang="ts">
	import { Logo, type NavItem } from '$lib/components/ui';
	import { isActive } from '$lib/components/shell/nav';
	import { toggleTheme } from '$lib/components/shell/theme';

	interface Props {
		items: NavItem[];
		current?: string;
		onLogout?: () => void;
	}
	let { items, current, onLogout }: Props = $props();
</script>

<nav class="sidenav" aria-label="Primary navigation">
	<a class="brand" href="/" aria-label="SwoleMate home">
		<Logo size={30} />
		<span class="name">SwoleMate</span>
	</a>

	<div class="items">
		{#each items as item (item.href)}
			{@const active = isActive(item.href, current)}
			<a href={item.href} class:active aria-current={active ? 'page' : undefined}>
				<span class="ico">{@render item.icon()}</span>
				<span class="lbl">{item.label}</span>
			</a>
		{/each}
	</div>

	<div class="foot">
		<button type="button" class="foot-btn theme-toggle" aria-label="Toggle dark mode" onclick={toggleTheme}>
			<svg class="ico-moon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
			</svg>
			<svg class="ico-sun" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<circle cx="12" cy="12" r="4" />
				<path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41" />
			</svg>
			<span class="lbl">Theme</span>
		</button>
		{#if onLogout}
			<button type="button" class="foot-btn" aria-label="Log out" onclick={onLogout}>
				<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
					<polyline points="16 17 21 12 16 7" />
					<line x1="21" y1="12" x2="9" y2="12" />
				</svg>
				<span class="lbl">Log out</span>
			</button>
		{/if}
	</div>
</nav>

<style>
	.sidenav {
		display: flex;
		flex-direction: column;
		gap: 4px;
		width: 220px;
		height: 100dvh;
		padding: 18px 14px;
		background: var(--surface-deep);
		color: var(--on-deep);
		border-right: 1px solid var(--on-deep-line);
	}
	.brand {
		display: flex;
		align-items: center;
		gap: 9px;
		margin-bottom: 18px;
		text-decoration: none;
		color: inherit;
	}
	.name {
		font: 800 16px/1 'Onest', system-ui, sans-serif;
		letter-spacing: -0.01em;
	}
	.items {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.items a {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 9px 11px;
		border-radius: 10px;
		font: 700 13px/1 'Onest', system-ui, sans-serif;
		color: var(--on-deep-soft);
		text-decoration: none;
		transition:
			background-color 160ms ease,
			color 160ms ease;
	}
	.items a:hover {
		color: var(--on-deep);
	}
	.items a.active {
		background: var(--clay);
		color: #fff;
		box-shadow: 0 6px 16px -8px var(--clay);
	}
	.ico {
		width: 18px;
		height: 18px;
		display: grid;
		place-items: center;
		flex: none;
	}
	.ico :global(svg) {
		width: 18px;
		height: 18px;
	}
	.foot {
		margin-top: auto;
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding-top: 10px;
		border-top: 1px solid var(--on-deep-line);
	}
	.foot-btn {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 9px 11px;
		border: 0;
		border-radius: 10px;
		background: transparent;
		color: var(--on-deep-soft);
		font: 700 12px/1 'Onest', system-ui, sans-serif;
		cursor: pointer;
		text-align: left;
		transition: color 160ms ease;
	}
	.foot-btn:hover {
		color: var(--on-deep);
	}
	.theme-toggle .ico-sun {
		display: none;
	}
	.theme-toggle .ico-moon {
		display: inline-block;
	}
	:global([data-theme='dark']) .theme-toggle .ico-sun {
		display: inline-block;
	}
	:global([data-theme='dark']) .theme-toggle .ico-moon {
		display: none;
	}
</style>
