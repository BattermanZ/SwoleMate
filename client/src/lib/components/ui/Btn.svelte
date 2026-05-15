<script lang="ts">
	import type { Snippet } from 'svelte';

	type Variant = 'primary' | 'success' | 'ink' | 'soft' | 'ghost' | 'icon';
	type Size = 'sm' | 'md';

	interface Props {
		variant?: Variant;
		size?: Size;
		type?: 'button' | 'submit' | 'reset';
		disabled?: boolean;
		onclick?: (e: MouseEvent) => void;
		'aria-label'?: string;
		children?: Snippet;
	}

	let {
		variant = 'soft',
		size = 'md',
		type = 'button',
		disabled = false,
		onclick,
		'aria-label': ariaLabel,
		children,
		...rest
	}: Props = $props();
</script>

<button
	{type}
	class="btn v-{variant} s-{size}"
	{disabled}
	{onclick}
	aria-label={ariaLabel}
	{...rest}
>
	{@render children?.()}
</button>

<style>
	.btn {
		border: 0;
		cursor: pointer;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		font-family: 'Onest', system-ui, sans-serif;
		transition:
			transform 100ms ease,
			background-color 120ms ease,
			color 120ms ease,
			box-shadow 160ms ease;
	}
	.btn:active:not(:disabled) {
		transform: scale(0.98);
	}
	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.s-md {
		padding: 13px 18px;
		font-size: 14px;
		font-weight: 700;
		border-radius: 999px;
	}
	.s-sm {
		padding: 9px 14px;
		font-size: 12px;
		font-weight: 700;
		border-radius: 999px;
	}

	.v-primary {
		background: linear-gradient(180deg, var(--clay-2), var(--clay));
		color: white;
		font-weight: 800;
		box-shadow:
			0 14px 28px -10px rgba(255, 94, 31, 0.55),
			inset 0 -3px 0 var(--clay-deep);
	}
	.v-primary:active:not(:disabled) {
		box-shadow:
			0 8px 18px -8px rgba(255, 94, 31, 0.55),
			inset 0 -2px 0 var(--clay-deep);
	}

	.v-success {
		background: linear-gradient(180deg, color-mix(in oklab, var(--sage) 85%, white), var(--sage));
		color: white;
		font-weight: 800;
		box-shadow: 0 10px 22px -8px rgba(79, 125, 84, 0.5);
	}

	.v-ink {
		background: var(--ink);
		color: var(--card);
	}

	.v-soft {
		background: var(--bg-2);
		color: var(--ink-2);
	}

	.v-ghost {
		background: transparent;
		color: var(--ink-soft);
		border: 1px solid var(--line);
	}

	.v-icon {
		background: transparent;
		color: var(--ink-2);
		padding: 0;
		width: 38px;
		height: 38px;
		border-radius: 10px;
	}
	.v-icon:hover:not(:disabled) {
		background: var(--bg-2);
	}
</style>
