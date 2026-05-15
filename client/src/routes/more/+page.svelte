<script lang="ts">
	import { auth } from '$lib/auth';
	import { Card, PageHero } from '$lib/components/ui';

	const authState = auth.state;
	let isAdmin = $derived($authState.user?.role === 'admin');
</script>

<div class="page">
	<PageHero kicker="► More">
		{#snippet title()}Everything <em>else.</em>{/snippet}
		{#snippet sub()}Settings, help, and admin tools for when you need them.{/snippet}
	</PageHero>

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
</div>

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
	a {
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
	a:hover {
		border-color: color-mix(in oklab, var(--clay) 40%, var(--line));
	}
	.icn {
		width: 42px;
		height: 42px;
		border-radius: 11px;
		background: linear-gradient(135deg, var(--bg-2), var(--card-3));
		display: grid;
		place-items: center;
		font: 800 18px/1 'Onest', system-ui, sans-serif;
	}
	.t {
		font: 800 14px/1 'Onest', system-ui, sans-serif;
	}
	.m {
		margin-top: 4px;
		font: 500 12px/1.4 'Onest', system-ui, sans-serif;
		color: var(--ink-soft);
	}
	.arr {
		font: 800 18px/1 'Onest', system-ui, sans-serif;
		color: var(--ink-soft);
	}
</style>
