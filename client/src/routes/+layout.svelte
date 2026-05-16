<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { auth } from '$lib/auth';
	import { logger } from '$lib/logger';
	import { BottomNav, type NavItem } from '$lib/components/ui';
	import AppBar from '$lib/components/shell/AppBar.svelte';
	import type { Snippet } from 'svelte';

	interface Props {
		children?: Snippet;
	}
	let { children }: Props = $props();

	const authState = auth.state;
	let currentPath = $derived($page.url.pathname);
	let isLogin = $derived(currentPath === '/login');

	$effect(() => {
		if ($authState.status === 'unauthenticated' && !isLogin) {
			void goto('/login');
		}
		if ($authState.status === 'authenticated' && isLogin) {
			void goto('/');
		}
	});

	onMount(() => {
		void auth.refresh();

		if ('serviceWorker' in navigator) {
			if (import.meta.env.DEV) {
				navigator.serviceWorker.getRegistrations().then((regs) => {
					for (const reg of regs) reg.unregister();
				});
				caches.keys().then((keys) => Promise.all(keys.map((k) => caches.delete(k))));
				return;
			}
			navigator.serviceWorker
				.register('/service-worker.js', { scope: '/' })
				.then((reg) => logger.debug('pwa', 'sw registered', { scope: reg.scope }))
				.catch((err) => logger.error('pwa', 'sw registration failed', { err }));
		}
	});

	$effect(() => {
		const shouldEnableRemoteLogs =
			!import.meta.env.DEV && $authState.status === 'authenticated' && !$authState.offline;
		logger.setRemoteEnabled(shouldEnableRemoteLogs);
	});

	function logout() {
		void auth.logout();
	}
</script>

<svelte:head>
	<title>SwoleMate</title>
</svelte:head>

{#snippet iconToday()}
	<svg
		width="14"
		height="14"
		viewBox="0 0 24 24"
		fill="none"
		stroke="currentColor"
		stroke-width="2.4"
		aria-hidden="true"
	>
		<path d="M12 21s-7-4.5-7-11a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 6.5-7 11-7 11z" />
	</svg>
{/snippet}
{#snippet iconPlans()}
	<svg
		width="14"
		height="14"
		viewBox="0 0 24 24"
		fill="none"
		stroke="currentColor"
		stroke-width="2"
		aria-hidden="true"
	>
		<rect x="4" y="4" width="16" height="16" rx="3" /><path d="M4 9h16" />
	</svg>
{/snippet}
{#snippet iconHistory()}
	<svg
		width="14"
		height="14"
		viewBox="0 0 24 24"
		fill="none"
		stroke="currentColor"
		stroke-width="2"
		aria-hidden="true"
	>
		<circle cx="12" cy="12" r="9" /><path d="M12 7v5l3 2" />
	</svg>
{/snippet}
{#snippet iconProgress()}
	<svg
		width="14"
		height="14"
		viewBox="0 0 24 24"
		fill="none"
		stroke="currentColor"
		stroke-width="2"
		aria-hidden="true"
	>
		<path d="M4 19h16" /><path d="M6 16l4-5 4 3 5-7" />
	</svg>
{/snippet}
{#snippet iconMore()}
	<span style="font: 800 14px/1 'Onest';" aria-hidden="true">⋯</span>
{/snippet}

{#if isLogin}
	{@render children?.()}
{:else}
	{@const navItems = [
		{ href: '/', label: 'Today', icon: iconToday },
		{ href: '/templates', label: 'Plans', icon: iconPlans },
		{ href: '/workouts', label: 'History', icon: iconHistory },
		{ href: '/progress', label: 'Progress', icon: iconProgress },
		{ href: '/more', label: 'More', icon: iconMore }
	] satisfies NavItem[]}
	<div class="shell">
		<AppBar onLogout={$authState.status === 'authenticated' ? logout : undefined} />

		{#if $authState.offline}
			<div class="offline-wrap">
				<div class="offline">
					<span class="dot"></span>
					Offline mode — showing cached data. Some actions are disabled.
				</div>
			</div>
		{/if}

		<main>
			{@render children?.()}
		</main>

		<BottomNav items={navItems} current={currentPath} />
	</div>
{/if}

<style>
	.shell {
		min-height: 100dvh;
		display: flex;
		flex-direction: column;
	}
	main {
		flex: 1;
		padding: 14px 18px calc(96px + env(safe-area-inset-bottom));
		max-width: 720px;
		width: 100%;
		margin: 0 auto;
	}
	.offline-wrap {
		display: flex;
		justify-content: center;
		padding: 8px 18px 0;
	}
	.offline {
		padding: 8px 14px;
		font:
			600 12px/1.3 'Onest',
			system-ui,
			sans-serif;
		color: var(--warn);
		background: color-mix(in oklab, var(--warn) 14%, var(--card));
		border: 1px solid color-mix(in oklab, var(--warn) 30%, var(--line));
		border-radius: 999px;
		display: inline-flex;
		align-items: center;
		gap: 8px;
	}
	.dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--warn);
	}
</style>
