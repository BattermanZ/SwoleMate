<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		hero: Snippet;
		notices: Snippet;
		list: Snippet;
		editor: Snippet;
	}
	let { hero, notices, list, editor }: Props = $props();
</script>

<div class="desk">
	<div class="band hero">{@render hero()}</div>
	<div class="band notices">{@render notices()}</div>

	<!-- Master-detail: template picker rail on the left, editor pane on the right. -->
	<div class="workspace">
		<aside class="col-rail">
			{@render list()}
		</aside>
		<div class="col-editor">
			{@render editor()}
		</div>
	</div>
</div>

<style>
	.desk {
		display: grid;
		grid-template-columns: minmax(0, 1280px);
		justify-content: center;
		gap: 18px;
	}

	.band {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.band:empty {
		display: none;
	}

	.workspace {
		display: grid;
		grid-template-columns: minmax(260px, 320px) minmax(0, 1fr);
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

	.col-editor {
		display: flex;
		flex-direction: column;
		gap: 14px;
		min-width: 0;
	}
</style>
