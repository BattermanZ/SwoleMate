<script lang="ts">
	import '../app.css';
	import { AppBar } from '@skeletonlabs/skeleton-svelte';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { auth } from '$lib/auth';
	import { logger } from '$lib/logger';

	let moreMenuOpen = false;
	let darkMode = false;
	const THEME_KEY = 'theme';
	const authState = auth.state;

	// Navigation items
	const navItems = [
		{ href: '/', label: 'Today', icon: '💪' },
		{ href: '/templates', label: 'Templates', icon: '🗂️' },
		{ href: '/workouts', label: 'History', icon: '📅' },
		{ href: '/progress', label: 'Progress', icon: '📈' },
		{ href: '/settings', label: 'Settings', icon: '⚙️' },
		{ href: '/help', label: 'Help', icon: '❓' },
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
	$: primaryMobileNavItems = visibleNavItems.filter((item) =>
		['/', '/templates', '/workouts', '/progress'].includes(item.href)
	);
	$: secondaryMobileNavItems = visibleNavItems.filter(
		(item) => !primaryMobileNavItems.some((primary) => primary.href === item.href)
	);
	$: isMoreActive = secondaryMobileNavItems.some((item) => isNavItemActive(item.href));

	function isNavItemActive(href: string): boolean {
		const pathname = $page.url.pathname;
		if (href === '/') return pathname === '/';
		return pathname === href || pathname.startsWith(`${href}/`);
	}

	function closeMoreMenu(): void {
		moreMenuOpen = false;
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

	$: if (isLogin || $page.url.pathname) {
		moreMenuOpen = false;
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
										class="btn btn-sm {isNavItemActive(item.href)
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

	{#if moreMenuOpen && !isLogin}
		<div class="fixed inset-0 z-40 md:hidden" role="dialog" aria-modal="true">
			<button class="absolute inset-0 bg-black/50" aria-label="Close menu" on:click={closeMoreMenu}
			></button>
			<nav
				id="mobile-more-menu"
				class="absolute bottom-0 left-0 right-0 max-h-[75dvh] overflow-y-auto rounded-t-2xl border-t border-surface-200/70 bg-surface-50-900-token p-4 pb-[calc(env(safe-area-inset-bottom)+5.75rem)] shadow-xl dark:border-surface-700/70"
			>
				<ul class="list-nav flex flex-col gap-2">
					{#each secondaryMobileNavItems as item}
						<li>
							<a
								href={item.href}
								class="btn w-full justify-start {isNavItemActive(item.href)
									? 'variant-filled-primary'
									: 'variant-ghost-primary'}"
								on:click={closeMoreMenu}
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
									closeMoreMenu();
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

	{#if !isLogin}
		<nav
			class="fixed bottom-0 left-0 right-0 z-50 border-t border-surface-200/80 bg-surface-50-900-token px-2 pt-2 pb-[calc(env(safe-area-inset-bottom)+0.5rem)] shadow-[0_-10px_30px_rgba(15,23,42,0.12)] md:hidden dark:border-surface-700/80 dark:shadow-[0_-10px_30px_rgba(2,6,23,0.34)]"
			aria-label="Primary mobile navigation"
		>
			<ul class="list-nav grid grid-cols-5 gap-1">
				{#each primaryMobileNavItems as item}
					<li>
						<a
							href={item.href}
							class="mobile-tab {isNavItemActive(item.href) ? 'mobile-tab-active' : ''}"
							aria-current={isNavItemActive(item.href) ? 'page' : undefined}
						>
							<span class="mobile-tab-icon">{item.icon}</span>
							<span class="mobile-tab-label">{item.label}</span>
						</a>
					</li>
				{/each}
				<li>
					<button
						type="button"
						class="mobile-tab {isMoreActive || moreMenuOpen ? 'mobile-tab-active' : ''}"
						aria-label="Open more navigation"
						aria-expanded={moreMenuOpen}
						aria-controls="mobile-more-menu"
						on:click={() => (moreMenuOpen = !moreMenuOpen)}
					>
						<span class="mobile-tab-icon" aria-hidden="true">•••</span>
						<span class="mobile-tab-label">More</span>
					</button>
				</li>
			</ul>
		</nav>
	{/if}
</div>
