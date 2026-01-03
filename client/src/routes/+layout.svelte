<script lang="ts">
	import '../app.css';
	import { AppBar } from '@skeletonlabs/skeleton-svelte';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';

	let drawerOpen = false;
	let darkMode = true;

	// Navigation items
	const navItems = [
		{ href: '/', label: 'Today', icon: '💪' },
		{ href: '/workouts', label: 'History', icon: '📅' },
		{ href: '/progress', label: 'Progress', icon: '📈' },
		{ href: '/settings', label: 'Settings', icon: '⚙️' },
		{ href: '/backups', label: 'Backups', icon: '💾' }
	];

	function toggleDrawer(): void {
		drawerOpen = !drawerOpen;
	}

	function setDarkMode(next: boolean) {
		darkMode = next;
		if (typeof document !== 'undefined') {
			document.documentElement.classList.toggle('dark', next);
		}
	}

	// Add service worker registration
	onMount(() => {
		if (typeof document !== 'undefined') {
			darkMode = document.documentElement.classList.contains('dark');
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
					console.log('ServiceWorker registration successful:', registration.scope);
				})
				.catch((err) => {
					console.error('ServiceWorker registration failed:', err);
				});
		}
	});
</script>

<div class="app-shell">
	<AppBar class="app-shell-header bg-surface-100-800-token border-b-2">
		<AppBar.Toolbar class="grid grid-cols-[1fr_auto] items-center gap-3">
			<AppBar.Lead>
				<a href="/" class="flex items-center gap-2">
					<span class="text-2xl">💪</span>
					<span class="text-xl font-bold">SwoleMate</span>
				</a>
			</AppBar.Lead>
			<AppBar.Trail class="flex items-center gap-2">
				<nav class="hidden md:block">
					<ul class="list-nav flex space-x-4">
						{#each navItems as item}
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

				<div class="md:hidden">
					<button class="btn btn-sm variant-ghost-primary" on:click={toggleDrawer}>
						<span>☰</span>
					</button>
				</div>

				<button
					type="button"
					class="btn btn-sm variant-ghost-primary"
					aria-label="Toggle dark mode"
					on:click={() => setDarkMode(!darkMode)}
				>
					<span aria-hidden="true">{darkMode ? '🌙' : '☀️'}</span>
				</button>
			</AppBar.Trail>
		</AppBar.Toolbar>
	</AppBar>

	{#if drawerOpen}
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
					{#each navItems as item}
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
				</ul>
			</nav>
		</div>
	{/if}

	<main class="container mx-auto p-4 flex-1 flex flex-col app-content h-full">
		<slot />
	</main>
</div>
