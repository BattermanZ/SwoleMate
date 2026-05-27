<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		hasSession: boolean;
		notices: Snippet;
		hero: Snippet;
		templatePicker: Snippet;
		primary: Snippet;
		recall: Snippet;
	}
	let { hasSession, notices, hero, templatePicker, primary, recall }: Props = $props();
</script>

<div class="desk" class:no-session={!hasSession}>
	<div class="band notices">{@render notices()}</div>
	<div class="band hero">{@render hero()}</div>

	{#if hasSession}
		<!-- Live session: active logging flow on the left, quick-recall rail on the right. -->
		<div class="workspace">
			<div class="col-primary">
				{@render primary()}
			</div>
			<aside class="col-rail">
				{@render recall()}
			</aside>
		</div>
	{:else}
		<!-- No session yet: centered single column so the start CTA isn't stranded. -->
		<div class="band stack">
			{@render templatePicker()}
			{@render recall()}
		</div>
	{/if}
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
	.band:empty {
		display: none;
	}

	.workspace {
		display: grid;
		grid-template-columns: minmax(0, 1.6fr) minmax(320px, 1fr);
		align-items: start;
		gap: 18px;
	}

	.col-primary {
		display: flex;
		flex-direction: column;
		gap: 14px;
		min-width: 0;
	}

	.col-rail {
		position: sticky;
		top: 24px;
		display: flex;
		flex-direction: column;
		gap: 14px;
		min-width: 0;
	}

	/* No-session: keep the hero + recall in a comfortable reading column. */
	.no-session {
		grid-template-columns: minmax(0, 680px);
	}
	.no-session .stack {
		gap: 14px;
	}

	/* Below the wide desktop workspace, fall back to a single column. */
	@media (max-width: 1200px) {
		.workspace {
			grid-template-columns: minmax(0, 1.4fr) minmax(280px, 1fr);
		}
	}
</style>
