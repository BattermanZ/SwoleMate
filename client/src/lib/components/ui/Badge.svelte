<script lang="ts">
	import type { Snippet } from 'svelte';

	type Tone = 'done' | 'live' | 'soft' | 'warn' | 'pr';

	interface Props {
		tone?: Tone;
		children?: Snippet;
	}

	let { tone = 'soft', children }: Props = $props();
</script>

<span class="badge t-{tone}">{@render children?.()}</span>

<style>
	.badge {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font: 700 9px/1 'Onest', system-ui, sans-serif;
		letter-spacing: 0.16em;
		text-transform: uppercase;
		padding: 5px 8px;
		border-radius: 999px;
	}
	.t-soft {
		background: var(--bg-2);
		color: var(--ink-2);
	}
	.t-done {
		background: color-mix(in oklab, var(--sage) 22%, var(--card));
		color: var(--sage);
	}
	.t-live {
		background: var(--clay);
		color: white;
	}
	.t-live::before {
		content: '';
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: white;
		animation: badge-pulse 1s ease-in-out infinite;
	}
	.t-warn {
		background: color-mix(in oklab, var(--warn) 22%, var(--card));
		color: var(--warn);
	}
	.t-pr {
		background: color-mix(in oklab, var(--gold) 24%, var(--card));
		color: var(--clay-text);
		border: 1px solid color-mix(in oklab, var(--gold) 40%, var(--line));
		padding: 4px 8px;
		letter-spacing: 0.15em;
	}

	@keyframes badge-pulse {
		50% {
			opacity: 0.3;
			transform: scale(0.85);
		}
	}
</style>
