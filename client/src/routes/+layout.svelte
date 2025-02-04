<script lang="ts">
	import '../app.postcss';
	import { AppShell, AppBar } from '@skeletonlabs/skeleton';

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
</script>

<AppShell>
	<svelte:fragment slot="header">
		<AppBar>
			<svelte:fragment slot="lead">
				<a href="/" class="text-xl font-bold">💪 SwoleMate</a>
			</svelte:fragment>
			<svelte:fragment slot="trail">
				<nav class="list-nav">
					<ul class="flex space-x-4">
						<li><a href="/workouts" class="hover:underline">Workouts</a></li>
						<li><a href="/progress" class="hover:underline">Progress</a></li>
						<li><a href="/settings" class="hover:underline">Settings</a></li>
					</ul>
				</nav>
			</svelte:fragment>
		</AppBar>
	</svelte:fragment>

	<main class="container mx-auto p-4">
		<slot />
	</main>

	<svelte:fragment slot="footer">
		<div class="container mx-auto p-4 text-center">
			<p class="text-sm">© 2024 SwoleMate. All rights reserved.</p>
		</div>
	</svelte:fragment>
</AppShell>
