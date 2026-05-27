<script lang="ts">
	import SideNav from '$lib/components/shell/SideNav.svelte';
	import type { NavItem } from '$lib/components/ui';

	interface Props {
		current?: string;
		onLogout?: () => void;
	}
	let { current, onLogout }: Props = $props();
</script>

{#snippet icon()}
	<svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="9" /></svg>
{/snippet}

{#snippet body()}
	{@const items = [
		{ href: '/', label: 'Today', icon },
		{ href: '/templates', label: 'Plans', icon },
		{ href: '/progress', label: 'Progress', icon }
	] satisfies NavItem[]}
	<SideNav {items} {current} {onLogout} />
{/snippet}

{@render body()}
