<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		hero: Snippet;
		actionsCard: Snippet;
		manualCard: Snippet;
		autoCard: Snippet;
	}
	let { hero, actionsCard, manualCard, autoCard }: Props = $props();
</script>

<div class="desk">
	<div class="band">{@render hero()}</div>

	<!-- Actions in a sticky rail; the manual + automatic backup lists fill the main column. -->
	<div class="workspace">
		<aside class="col-rail">{@render actionsCard()}</aside>
		<div class="col-main">
			{@render manualCard()}
			{@render autoCard()}
		</div>
	</div>
</div>

<style>
	.desk {
		display: grid;
		grid-template-columns: minmax(0, 1180px);
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
		grid-template-columns: minmax(280px, 340px) minmax(0, 1fr);
		align-items: start;
		gap: 18px;
	}
	.col-rail {
		position: sticky;
		top: 24px;
		max-height: calc(100dvh - 48px);
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 14px;
		min-width: 0;
	}
	.col-main {
		display: flex;
		flex-direction: column;
		gap: 14px;
		min-width: 0;
	}
</style>
