<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		hero: Snippet;
		metrics: Snippet;
		filters: Snippet;
		list: Snippet;
	}
	let { hero, metrics, filters, list }: Props = $props();
</script>

<div class="desk">
	<div class="band hero">{@render hero()}</div>

	<!-- Filters + summary in a sticky rail; the session list fills the wide main column. -->
	<div class="workspace">
		<aside class="col-rail">
			{@render metrics()}
			{@render filters()}
		</aside>
		<div class="col-main">
			{@render list()}
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

	/* Rail is narrow: stack the metric tiles instead of squeezing 3 across. */
	.col-rail :global(.metrics) {
		grid-template-columns: 1fr;
	}
</style>
