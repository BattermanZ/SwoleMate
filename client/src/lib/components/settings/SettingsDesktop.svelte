<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		hero: Snippet;
		accountCard: Snippet;
		toolsCard: Snippet;
		mcpCard: Snippet;
	}
	let { hero, accountCard, toolsCard, mcpCard }: Props = $props();
</script>

<div class="desk">
	<div class="band">{@render hero()}</div>

	<!-- Account + tools on the left; the taller MCP tokens card on the right balances height. -->
	<div class="workspace">
		<div class="col">
			{@render accountCard()}
			{@render toolsCard()}
		</div>
		<div class="col">
			{@render mcpCard()}
		</div>
	</div>
</div>

<style>
	.desk {
		display: grid;
		grid-template-columns: minmax(0, 1100px);
		justify-content: center;
		gap: 18px;
	}
	.band {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.workspace {
		display: grid;
		grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
		align-items: start;
		gap: 18px;
	}
	.col {
		display: flex;
		flex-direction: column;
		gap: 14px;
		min-width: 0;
	}
</style>
