<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		/** Left pane — the list of items. */
		list: Snippet;
		/** Right pane — the detail of the selected item. Omit to show `empty`. */
		detail?: Snippet;
		/** Shown in the detail pane when `detail` is not provided. */
		empty?: Snippet;
	}

	let { list, detail, empty }: Props = $props();
</script>

<div class="master-detail">
	<aside class="list">{@render list()}</aside>
	<section class="detail">
		{#if detail}
			{@render detail()}
		{:else if empty}
			{@render empty()}
		{/if}
	</section>
</div>

<style>
	.master-detail {
		display: flex;
		gap: 16px;
		min-height: 0;
		height: 100%;
	}
	.list {
		width: 320px;
		flex: none;
		overflow-y: auto;
		min-height: 0;
	}
	.detail {
		flex: 1;
		min-width: 0;
		overflow-y: auto;
		min-height: 0;
	}
</style>
