<script lang="ts">
	import type { Snippet } from 'svelte';

	/**
	 * The always-dark signature surface of the app. Renders a deep ink card
	 * with two drifting orb glows (clay + gold). Three variants based on
	 * children: `timer`, `consistency`, or `plain` — but those are visual
	 * intentions; the component itself is just the dark surface with kicker,
	 * title, and a children slot.
	 */

	interface Props {
		kicker?: string;
		title?: Snippet;
		sub?: Snippet;
		actions?: Snippet;
		children?: Snippet;
	}

	let { kicker, title, sub, actions, children }: Props = $props();
</script>

<section class="hero">
	{#if kicker}<div class="kicker">{kicker}</div>{/if}
	{#if title}<h1>{@render title()}</h1>{/if}
	{#if sub}<div class="sub">{@render sub()}</div>{/if}
	{#if children}<div class="hero-body">{@render children()}</div>{/if}
	{#if actions}<div class="hero-actions">{@render actions()}</div>{/if}
</section>

<style>
	.hero {
		background: var(--surface-deep);
		color: var(--on-deep);
		border-radius: 28px;
		padding: 22px 22px 22px;
		position: relative;
		overflow: hidden;
		box-shadow: 0 24px 48px -16px var(--shadow-strong);
	}
	.hero::before {
		content: '';
		position: absolute;
		top: -120px;
		right: -100px;
		width: 320px;
		height: 320px;
		border-radius: 50%;
		background: radial-gradient(circle, rgba(255, 94, 31, 0.55), transparent 65%);
		animation: hero-drift 12s ease-in-out infinite alternate;
		pointer-events: none;
	}
	.hero::after {
		content: '';
		position: absolute;
		bottom: -140px;
		left: -80px;
		width: 300px;
		height: 300px;
		border-radius: 50%;
		background: radial-gradient(circle, rgba(213, 162, 58, 0.35), transparent 65%);
		animation: hero-drift 14s ease-in-out infinite alternate-reverse;
		pointer-events: none;
	}
	@keyframes hero-drift {
		to {
			transform: translate(20px, -10px) scale(1.05);
		}
	}

	.kicker {
		font:
			700 11px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.22em;
		text-transform: uppercase;
		color: var(--clay-2);
		position: relative;
	}

	h1 {
		margin: 8px 0 0;
		font:
			800 28px/1.02 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: -0.025em;
		position: relative;
	}
	h1 :global(em) {
		font: italic 400 28px/1.02 'Instrument Serif';
		color: var(--clay-2);
	}

	.sub {
		margin-top: 6px;
		font: italic 400 13px/1.3 'Instrument Serif';
		color: var(--on-deep-soft);
		position: relative;
	}

	.hero-body {
		margin-top: 18px;
		position: relative;
	}
	.hero-actions {
		margin-top: 14px;
		display: flex;
		gap: 8px;
		position: relative;
	}
</style>
