<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		ready: boolean;
		hero: Snippet;
		gate: Snippet;
		createPanel: Snippet;
		usersList: Snippet;
		resetCard: Snippet;
	}
	let { ready, hero, gate, createPanel, usersList, resetCard }: Props = $props();
</script>

<div class="desk" class:blocked={!ready}>
	<div class="band">{@render hero()}</div>

	{#if ready}
		<!-- Create + reset forms in a rail; the user list fills the main column. -->
		<div class="workspace">
			<aside class="col-rail">
				{@render createPanel()}
				{@render resetCard()}
			</aside>
			<div class="col-main">{@render usersList()}</div>
		</div>
	{:else}
		<div class="band">{@render gate()}</div>
	{/if}
</div>

<style>
	.desk {
		display: grid;
		grid-template-columns: minmax(0, 1180px);
		justify-content: center;
		gap: 18px;
	}
	.blocked {
		grid-template-columns: minmax(0, 680px);
	}
	.band {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.workspace {
		display: grid;
		grid-template-columns: minmax(300px, 380px) minmax(0, 1fr);
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
