<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { auth } from '$lib/auth';
	import { logger } from '$lib/logger';
	import { requestPersistentStorage } from '$lib/pwa/persistentStorage';
	import { BottomNav, type NavItem } from '$lib/components/ui';
	import ConfirmHost from '$lib/components/ui/ConfirmHost.svelte';
	import WhatsNewHost from '$lib/components/ui/WhatsNewHost.svelte';
	import { maybeShowWhatsNew } from '$lib/stores/whatsNew';
	import AppBar from '$lib/components/shell/AppBar.svelte';
	import SideNav from '$lib/components/shell/SideNav.svelte';
	import type { Snippet } from 'svelte';

	interface Props {
		children?: Snippet;
	}
	let { children }: Props = $props();

	const authState = auth.state;
	let currentPath = $derived($page.url.pathname);
	let isLogin = $derived(currentPath === '/login');

	// Only render protected page content once we actually know the session is
	// valid. While auth is still 'unknown' (the initial /auth/me check is in
	// flight) we must NOT flash cached data for a session that may already be
	// revoked server-side (F-MED-8). The exception is offline mode: we can't
	// verify with the server, so we trust the cached session — this preserves the
	// offline-first experience.
	let contentReady = $derived(
		$authState.status === 'authenticated' ||
			($authState.status === 'unknown' && $authState.offline)
	);

	$effect(() => {
		if ($authState.status === 'unauthenticated' && !isLogin) {
			void goto('/login');
		}
		if ($authState.status === 'authenticated' && isLogin) {
			// Send users flagged for a forced password change to /settings, not home.
			// This effect races the login page's own post-login goto and, since
			// isLogin is still true until navigation resolves, would otherwise win
			// and drop a must-change-password user on '/' (F-LOW-4).
			void goto($authState.user?.must_change_password ? '/settings' : '/');
		}
	});

	// Once the user is authenticated, show the changelog if the app has updated
	// since they last opened it. Runs a single time per session.
	let whatsNewChecked = false;
	$effect(() => {
		if (!whatsNewChecked && $authState.status === 'authenticated') {
			whatsNewChecked = true;
			maybeShowWhatsNew();
		}
	});

	onMount(() => {
		void auth.refresh();

		if (!import.meta.env.DEV && !document.querySelector('script[data-umami]')) {
			const s = document.createElement('script');
			s.src = 'https://analytics.battercloud.cc/script.js';
			s.defer = true;
			s.dataset.websiteId = '55c688cf-5605-48ae-b3ae-2a6a9f342c51';
			s.dataset.umami = 'true';
			document.head.appendChild(s);
		}

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

			// Move offline data off the best-effort tier so it survives storage
			// pressure and iOS Safari's 7-day eviction window.
			void requestPersistentStorage();
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
		<div class="chrome chrome-appbar">
			<AppBar onLogout={$authState.status === 'authenticated' ? logout : undefined} />
		</div>
		<div class="chrome chrome-sidenav">
			<SideNav
				items={navItems}
				current={currentPath}
				onLogout={$authState.status === 'authenticated' ? logout : undefined}
			/>
		</div>

		<div class="content">
			{#if $authState.offline}
				<div class="offline-wrap">
					<div class="offline">
						<span class="dot"></span>
						Offline mode — showing cached data. Some actions are disabled.
					</div>
				</div>
			{/if}
			<main>
				{#if contentReady}
					{@render children?.()}
				{:else}
					<div class="auth-pending" role="status" aria-live="polite">
						<span class="spinner" aria-hidden="true"></span>
						<span class="sr-only">Checking your session…</span>
					</div>
				{/if}
			</main>
		</div>

		<div class="chrome chrome-bottomnav">
			<BottomNav items={navItems} current={currentPath} />
		</div>
	</div>
{/if}

<ConfirmHost />
<WhatsNewHost />

<style>
	.shell {
		min-height: 100dvh;
		display: flex;
		flex-direction: column;
	}
	.auth-pending {
		display: flex;
		align-items: center;
		justify-content: center;
		min-height: 50dvh;
	}
	.spinner {
		width: 28px;
		height: 28px;
		border-radius: 50%;
		border: 3px solid var(--color-border, rgba(128, 128, 128, 0.3));
		border-top-color: var(--color-accent, #4f8cff);
		animation: auth-spin 0.7s linear infinite;
	}
	@keyframes auth-spin {
		to {
			transform: rotate(360deg);
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.spinner {
			animation-duration: 2s;
		}
	}
	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border: 0;
	}
	/* Chrome wrappers are layout-transparent on mobile so AppBar's sticky and
	   BottomNav's fixed positioning behave as if direct children of .shell. */
	.chrome-appbar,
	.chrome-bottomnav {
		display: contents;
	}
	.chrome-sidenav {
		display: none;
	}
	.content {
		flex: 1;
		display: flex;
		flex-direction: column;
		min-width: 0;
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

	/* Desktop: sidebar rail + content column. Children render ONCE; only the
	   chrome swaps. Breakpoint MUST match DESKTOP_MIN_WIDTH (1024) in
	   lib/stores/viewport.ts. */
	@media (min-width: 1024px) {
		.shell {
			flex-direction: row;
		}
		.chrome-appbar,
		.chrome-bottomnav {
			display: none;
		}
		.chrome-sidenav {
			display: block;
			position: sticky;
			top: 0;
			align-self: flex-start;
			height: 100dvh;
			flex: none;
		}
		main {
			padding: 24px;
			max-width: none;
			margin: 0;
		}
	}
</style>
