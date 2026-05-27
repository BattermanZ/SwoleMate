<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		ready: boolean;
		hero: Snippet;
		summary: Snippet;
		exercises: Snippet;
	}
	let { ready, hero, summary, exercises }: Props = $props();
</script>

<div class="desk" class:loading={!ready}>
	<div class="band">{@render hero()}</div>

	{#if ready}
		<!-- Session summary + actions in a sticky rail; logged exercises fill the main column. -->
		<div class="workspace">
			<aside class="col-rail">{@render summary()}</aside>
			<div class="col-main">{@render exercises()}</div>
		</div>
	{:else}
		<div class="band">{@render summary()}</div>
	{/if}
</div>

<style>
	.desk {
		display: grid;
		grid-template-columns: minmax(0, 1280px);
		justify-content: center;
		gap: 18px;
	}
	.loading {
		grid-template-columns: minmax(0, 680px);
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
