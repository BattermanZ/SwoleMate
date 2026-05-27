<script lang="ts">
	import DesktopTopBar from '$lib/components/shell/DesktopTopBar.svelte';

	interface Props {
		title: string;
		subtitle?: string;
		withActions?: boolean;
	}
	let { title, subtitle, withActions = false }: Props = $props();
</script>

{#if withActions}
	<DesktopTopBar {title} {subtitle}>
		{#snippet actions()}
			<button data-testid="action-btn">New</button>
		{/snippet}
	</DesktopTopBar>
{:else}
	<DesktopTopBar {title} {subtitle} />
{/if}
