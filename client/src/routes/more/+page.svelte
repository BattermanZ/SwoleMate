<script lang="ts">
	import { auth } from '$lib/auth';
	import { Card, PageHero } from '$lib/components/ui';
	import MoreDesktop from '$lib/components/more/MoreDesktop.svelte';
	import { isDesktop, isDesktopView } from '$lib/stores/viewport';
	import { openWhatsNew } from '$lib/stores/whatsNew';
	import { APP_VERSION } from '$lib/version';

	const authState = auth.state;
	let isAdmin = $derived($authState.user?.role === 'admin');

	let desktop = $derived(isDesktopView($isDesktop));
</script>

{#snippet hero()}
	<PageHero kicker="► More">
		{#snippet title()}Everything <em>else.</em>{/snippet}
		{#snippet sub()}Settings, help, and admin tools for when you need them.{/snippet}
	</PageHero>
{/snippet}

{#snippet menu()}
	<Card>
		<nav class="links">
			<a href="/settings">
				<div class="icn">⚙</div>
				<div class="text">
					<div class="t">Settings</div>
					<div class="m">Account, password, MCP tokens, demo mode.</div>
				</div>
				<span class="arr" aria-hidden="true">→</span>
			</a>
			<a href="/backups">
				<div class="icn">💾</div>
				<div class="text">
					<div class="t">Data &amp; backups</div>
					<div class="m">Manual + automatic backups of your training data.</div>
				</div>
				<span class="arr" aria-hidden="true">→</span>
			</a>
			<a href="/help">
				<div class="icn">?</div>
				<div class="text">
					<div class="t">Help</div>
					<div class="m">Quick guidance + reset local UI state.</div>
				</div>
				<span class="arr" aria-hidden="true">→</span>
			</a>
			<button type="button" class="row" onclick={() => openWhatsNew()}>
				<div class="icn">✨</div>
				<div class="text">
					<div class="t">What's New</div>
					<div class="m">Release notes — you're on v{APP_VERSION}.</div>
				</div>
				<span class="arr" aria-hidden="true">→</span>
			</button>
			{#if isAdmin}
				<a href="/admin">
					<div class="icn">🛡</div>
					<div class="text">
						<div class="t">Admin</div>
						<div class="m">Manage users — admins only.</div>
					</div>
					<span class="arr" aria-hidden="true">→</span>
				</a>
			{/if}
		</nav>
	</Card>
{/snippet}

{#if desktop}
	<MoreDesktop {hero} {menu} />
{:else}
	<div class="page">
		{@render hero()}
		{@render menu()}
	</div>
{/if}

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.links {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	/* Desktop: flow the nav links into a 2-up grid instead of stretched rows. */
	@media (min-width: 1024px) {
		.links {
			display: grid;
			grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
			align-items: start;
		}
	}
	a,
	.row {
		display: grid;
		grid-template-columns: 42px 1fr auto;
		gap: 12px;
		align-items: center;
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 14px;
		padding: 12px 14px;
		text-decoration: none;
		color: inherit;
	}
	/* The What's New entry opens a modal rather than navigating, so it's a button
	   styled to match the link rows. */
	.row {
		width: 100%;
		text-align: left;
		font: inherit;
		cursor: pointer;
	}
	a:hover,
	.row:hover {
		border-color: color-mix(in oklab, var(--clay) 40%, var(--line));
	}
	.icn {
		width: 42px;
		height: 42px;
		border-radius: 11px;
		background: linear-gradient(135deg, var(--bg-2), var(--card-3));
		display: grid;
		place-items: center;
		font:
			800 18px/1 'Onest',
			system-ui,
			sans-serif;
	}
	.t {
		font:
			800 14px/1 'Onest',
			system-ui,
			sans-serif;
	}
	.m {
		margin-top: 4px;
		font:
			500 12px/1.4 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}
	.arr {
		font:
			800 18px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}
</style>
