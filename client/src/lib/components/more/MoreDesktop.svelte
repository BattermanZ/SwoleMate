<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		hero: Snippet;
		menu: Snippet;
	}
	let { hero, menu }: Props = $props();
</script>

<div class="desk">
	<div class="band">{@render hero()}</div>
	<div class="band">{@render menu()}</div>
</div>

<style>
	.desk {
		display: grid;
		grid-template-columns: minmax(0, 960px);
		justify-content: center;
		gap: 18px;
	}
	.band {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
</style>
