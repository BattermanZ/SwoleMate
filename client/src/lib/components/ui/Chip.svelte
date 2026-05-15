<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		size?: 'xs' | 'sm';
		children?: Snippet;
	}

	let { size = 'sm', children }: Props = $props();
</script>

<span class="chip s-{size}">{@render children?.()}</span>

<style>
	.chip {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		background: var(--bg-2);
		color: var(--ink-2);
		font-family: 'Onest', system-ui, sans-serif;
		font-weight: 700;
		letter-spacing: 0.02em;
		border-radius: 8px;
	}
	.s-xs {
		font-size: 10px;
		padding: 4px 7px;
	}
	.s-sm {
		font-size: 11px;
		padding: 5px 8px;
	}
</style>
