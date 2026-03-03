<script lang="ts">
	import '../app.css';
	import { AppBar } from '@skeletonlabs/skeleton-svelte';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { auth } from '$lib/auth';
	import { logger } from '$lib/logger';

	let drawerOpen = false;
	let darkMode = false;
	const THEME_KEY = 'theme';
	const authState = auth.state;

	// Navigation items
	const navItems = [
		{ href: '/', label: 'Today', icon: '💪' },
		{ href: '/workouts', label: 'History', icon: '📅' },
		{ href: '/progress', label: 'Progress', icon: '📈' },
		{ href: '/settings', label: 'Help', icon: '❓' },
		{ href: '/admin', label: 'Admin', icon: '🛡️' },
		{ href: '/backups', label: 'Backups', icon: '💾' }
	];

	$: isLogin = $page.url.pathname === '/login';
	$: isSettings = $page.url.pathname === '/settings';
	$: mustChangePassword =
		$authState.status === 'authenticated' && $authState.user?.must_change_password;
	$: canSeeAdmin =
		$authState.status === 'authenticated' &&
		$authState.user?.role === 'admin' &&
		!$authState.offline;
	$: canSeeBackups =
		$authState.status === 'authenticated' &&
		$authState.user?.role === 'admin' &&
		!$authState.offline;
	$: visibleNavItems = navItems.filter((item) => {
		if (item.href === '/admin') return canSeeAdmin;
		if (item.href === '/backups') return canSeeBackups;
		return true;
	});

	function toggleDrawer(): void {
		drawerOpen = !drawerOpen;
	}

	function applyTheme(next: boolean) {
		darkMode = next;
		if (typeof document === 'undefined') return;
		document.documentElement.classList.toggle('dark', next);
		document.documentElement.style.colorScheme = next ? 'dark' : 'light';

		try {
			localStorage.setItem(THEME_KEY, next ? 'dark' : 'light');
		} catch {
			// ignore
		}

		const meta = document.querySelector('meta[name="theme-color"]');
		if (meta) meta.setAttribute('content', next ? '#020617' : '#1d4ed8');
	}

	// Add service worker registration
	onMount(() => {
		void auth.refresh();

		if (typeof document !== 'undefined') {
			try {
				const stored = localStorage.getItem(THEME_KEY);
				if (stored === 'dark' || stored === 'light') {
					applyTheme(stored === 'dark');
				} else {
					const prefersDark = window.matchMedia?.('(prefers-color-scheme: dark)').matches ?? false;
					applyTheme(prefersDark);
				}
			} catch {
				darkMode = document.documentElement.classList.contains('dark');
			}
		}

		if ('serviceWorker' in navigator) {
			// Service workers can easily serve stale bundles during development.
			// Keep it enabled only for production builds.
			if (import.meta.env.DEV) {
				navigator.serviceWorker.getRegistrations().then((registrations) => {
					for (const registration of registrations) {
						registration.unregister();
					}
				});
				caches.keys().then((keys) => Promise.all(keys.map((key) => caches.delete(key))));
				return;
			}

			navigator.serviceWorker
				.register('/service-worker.js', { scope: '/' })
				.then((registration) => {
					logger.debug('pwa', 'ServiceWorker registration successful', {
						scope: registration.scope
					});
				})
				.catch((err) => {
					logger.error('pwa', 'ServiceWorker registration failed', { err });
				});
		}
	});

	$: if ($authState.status === 'unauthenticated' && !isLogin) {
		void goto('/login');
	}

	$: if ($authState.status === 'authenticated' && isLogin) {
		void goto(mustChangePassword ? '/settings' : '/');
	}

	$: if ($authState.status === 'authenticated' && mustChangePassword && !isLogin && !isSettings) {
		void goto('/settings');
	}

	$: {
		const shouldEnableRemoteLogs =
			!import.meta.env.DEV && $authState.status === 'authenticated' && !$authState.offline;
		logger.setRemoteEnabled(shouldEnableRemoteLogs);
	}
</script>

<div class="app-shell">
	<AppBar class="app-shell-header bg-surface-100-800-token border-b-2 relative z-50">
		<AppBar.Toolbar class="grid grid-cols-[1fr_auto] items-center gap-3">
			<AppBar.Lead>
				<a href="/" class="flex items-center gap-2">
					<span class="text-2xl">💪</span>
					<span class="text-xl font-bold">SwoleMate</span>
				</a>
			</AppBar.Lead>
			<AppBar.Trail class="flex items-center gap-2">
				{#if !isLogin}
					<nav class="hidden md:block">
						<ul class="list-nav flex space-x-4">
							{#each visibleNavItems as item}
								<li>
									<a
										href={item.href}
										class="btn btn-sm {$page.url.pathname === item.href
											? 'variant-filled-primary'
											: 'variant-ghost-primary'}"
									>
										<span>{item.icon}</span>
										<span>{item.label}</span>
									</a>
								</li>
							{/each}
						</ul>
					</nav>
				{/if}

				{#if !isLogin}
					<div class="md:hidden">
						<button class="btn btn-sm variant-ghost-primary" on:click={toggleDrawer}>
							<span>☰</span>
						</button>
					</div>
				{/if}

				<button
					type="button"
					class="btn btn-sm variant-ghost-primary"
					aria-label="Toggle dark mode"
					on:click={() => applyTheme(!darkMode)}
				>
					<span aria-hidden="true">{darkMode ? '🌙' : '☀️'}</span>
				</button>

				{#if !isLogin && $authState.offline}
					<span class="badge variant-soft-warning hidden sm:inline-flex">Offline</span>
				{/if}

				{#if !isLogin && $authState.status === 'authenticated'}
					<button
						type="button"
						class="btn btn-sm variant-ghost-primary"
						aria-label="Log out"
						on:click={() => auth.logout()}
					>
						<span aria-hidden="true">⎋</span>
					</button>
				{/if}
			</AppBar.Trail>
		</AppBar.Toolbar>
	</AppBar>

	{#if drawerOpen && !isLogin}
		<div class="fixed inset-0 z-50 md:hidden" role="dialog" aria-modal="true">
			<button
				class="absolute inset-0 bg-black/50"
				aria-label="Close menu"
				on:click={() => (drawerOpen = false)}
			></button>
			<nav
				class="absolute right-0 top-0 h-full w-72 bg-surface-50-900-token shadow-lg overflow-y-auto drawer-content p-4 pt-[env(safe-area-inset-top)] pb-[env(safe-area-inset-bottom)] pr-[env(safe-area-inset-right)]"
			>
				<ul class="list-nav flex flex-col space-y-4">
					{#each visibleNavItems as item}
						<li>
							<a
								href={item.href}
								class="btn w-full justify-start {$page.url.pathname === item.href
									? 'variant-filled-primary'
									: 'variant-ghost-primary'}"
								on:click={() => (drawerOpen = false)}
							>
								<span class="text-xl">{item.icon}</span>
								<span>{item.label}</span>
							</a>
						</li>
					{/each}
					{#if $authState.status === 'authenticated'}
						<li class="pt-2 border-t border-surface-200/50 dark:border-surface-700/50">
							<button
								type="button"
								class="btn w-full justify-start variant-soft-error"
								on:click={() => {
									drawerOpen = false;
									void auth.logout();
								}}
							>
								<span class="text-xl">⎋</span>
								<span>Log out</span>
							</button>
						</li>
					{/if}
				</ul>
			</nav>
		</div>
	{/if}

	<main class="container mx-auto p-4 flex-1 flex flex-col app-content h-full">
		{#if $authState.offline}
			<div class="offline-banner">
				<span>Offline mode: showing cached data. Some actions are disabled.</span>
			</div>
		{/if}
		<slot />
	</main>
</div>
