<script lang="ts">
	import '../app.postcss';
	import { AppShell, AppBar, LightSwitch, Drawer, initializeStores } from '@skeletonlabs/skeleton';
	import { getDrawerStore } from '@skeletonlabs/skeleton';
	import { page } from '$app/stores';

	// Initialize all Skeleton stores
	initializeStores();

	// Get the drawer store
	const drawerStore = getDrawerStore();

	// Highlight JS
	import hljs from 'highlight.js/lib/core';
	import 'highlight.js/styles/github-dark.css';
	import { storeHighlightJs } from '@skeletonlabs/skeleton';
	import xml from 'highlight.js/lib/languages/xml'; // for HTML
	import css from 'highlight.js/lib/languages/css';
	import javascript from 'highlight.js/lib/languages/javascript';
	import typescript from 'highlight.js/lib/languages/typescript';

	hljs.registerLanguage('xml', xml); // for HTML
	hljs.registerLanguage('css', css);
	hljs.registerLanguage('javascript', javascript);
	hljs.registerLanguage('typescript', typescript);
	storeHighlightJs.set(hljs);

	// Floating UI for Popups
	import { computePosition, autoUpdate, flip, shift, offset, arrow } from '@floating-ui/dom';
	import { storePopup } from '@skeletonlabs/skeleton';
	storePopup.set({ computePosition, autoUpdate, flip, shift, offset, arrow });

	// Navigation items
	const navItems = [
		{ href: '/', label: 'Today', icon: '💪' },
		{ href: '/workouts', label: 'History', icon: '📅' },
		{ href: '/progress', label: 'Progress', icon: '📈' },
		{ href: '/settings', label: 'Settings', icon: '⚙️' }
	];

	function toggleDrawer(): void {
		drawerStore.set({ open: true });
	}
</script>

<Drawer position="right">
	<nav class="p-4">
		<ul class="list-nav flex flex-col space-y-4">
			{#each navItems as item}
				<li>
					<a 
						href={item.href} 
						class="btn {$page.url.pathname === item.href ? 'variant-filled-primary' : 'variant-ghost-primary'}"
						on:click={() => drawerStore.set({ open: false })}
					>
						<span class="text-xl">{item.icon}</span>
						<span>{item.label}</span>
					</a>
				</li>
			{/each}
		</ul>
	</nav>
</Drawer>

<AppShell>
	<svelte:fragment slot="header">
		<AppBar background="bg-surface-100-800-token" border="border-b-2">
			<svelte:fragment slot="lead">
				<a href="/" class="flex items-center space-x-2">
					<span class="text-2xl">💪</span>
					<span class="text-xl font-bold">SwoleMate</span>
				</a>
			</svelte:fragment>
			<svelte:fragment slot="trail">
				<nav class="hidden md:block">
					<ul class="list-nav flex space-x-4">
						{#each navItems as item}
							<li>
								<a 
									href={item.href} 
									class="btn btn-sm {$page.url.pathname === item.href ? 'variant-filled-primary' : 'variant-ghost-primary'}"
								>
									<span>{item.icon}</span>
									<span>{item.label}</span>
								</a>
							</li>
						{/each}
					</ul>
				</nav>
				<!-- Mobile menu -->
				<div class="md:hidden">
					<button class="btn btn-sm variant-ghost-primary" on:click={toggleDrawer}>
						<span>☰</span>
					</button>
				</div>
				<LightSwitch />
			</svelte:fragment>
		</AppBar>
	</svelte:fragment>

	<main class="container mx-auto p-4 flex-1 flex flex-col min-h-[calc(100vh-theme(spacing.32))]">
		<slot />
	</main>

	<svelte:fragment slot="footer">
		<div class="container mx-auto p-4">
			<hr class="opacity-50 my-4" />
			<div class="flex justify-between items-center">
				<p class="text-sm opacity-50">© 2024 SwoleMate. All rights reserved.</p>
			</div>
		</div>
	</svelte:fragment>
</AppShell>

<style>
	:global(html), :global(body) {
		@apply h-full overflow-hidden;
	}
	:global(body) {
		@apply bg-surface-50-900-token;
	}
</style>
