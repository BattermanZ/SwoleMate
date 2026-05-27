<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		hero: Snippet;
		howToCard: Snippet;
		goodToKnowCard: Snippet;
		troubleshootCard: Snippet;
	}
	let { hero, howToCard, goodToKnowCard, troubleshootCard }: Props = $props();
</script>

<div class="desk">
	<div class="band">{@render hero()}</div>

	<!-- Walkthrough on the left; reference + reset stacked on the right. -->
	<div class="workspace">
		<div class="col">
			{@render howToCard()}
		</div>
		<div class="col">
			{@render goodToKnowCard()}
			{@render troubleshootCard()}
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
