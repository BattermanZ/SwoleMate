<script lang="ts">
	import { PageHero, Btn } from '$lib/components/ui';

	interface Props {
		showDemoAction?: boolean;
		offlineMode?: boolean;
		loading?: boolean;
		onStart?: () => void;
		onUseTemplate?: () => void;
		onDemo?: () => void;
	}
	let {
		showDemoAction = false,
		offlineMode = false,
		loading = false,
		onStart,
		onUseTemplate,
		onDemo
	}: Props = $props();
</script>

<PageHero kicker="► No session yet">
	{#snippet title()}Ready when <em>you are.</em>{/snippet}
	{#snippet sub()}Start a fresh session, load a template plan, or run the demo to explore.{/snippet}

	<div class="ctas">
		<Btn variant="primary" onclick={onStart} disabled={loading}>▶ Start session</Btn>
		<Btn variant="soft" onclick={onUseTemplate} disabled={loading || offlineMode}>Use template</Btn>
		{#if showDemoAction}
			<Btn variant="ghost" onclick={onDemo} disabled={loading}>Load demo</Btn>
		{/if}
	</div>
</PageHero>

<style>
	.ctas {
		display: grid;
		grid-template-columns: 1fr;
		gap: 8px;
	}
	.ctas :global(button) {
		justify-content: center;
	}
</style>
