<script lang="ts">
	interface Props {
		value: number;
		step?: number;
		min?: number;
		max?: number;
		unit?: string;
		label?: string;
		disabled?: boolean;
		'aria-label'?: string;
		onchange?: (v: number) => void;
		/** Custom formatter for the displayed value (e.g. one decimal place). */
		format?: (v: number) => string;
	}

	let {
		value = $bindable(0),
		step = 1,
		min,
		max,
		unit,
		label,
		disabled = false,
		'aria-label': ariaLabel,
		onchange,
		format
	}: Props = $props();

	function clamp(n: number): number {
		if (min !== undefined && n < min) return min;
		if (max !== undefined && n > max) return max;
		return n;
	}

	function inc() {
		if (disabled) return;
		const next = clamp(value + step);
		if (next === value) return;
		value = next;
		onchange?.(next);
	}

	function dec() {
		if (disabled) return;
		const next = clamp(value - step);
		if (next === value) return;
		value = next;
		onchange?.(next);
	}

	let display = $derived(format ? format(value) : String(value));
</script>

<div class="stepper-pill" class:disabled aria-label={ariaLabel ?? label}>
	<button
		type="button"
		class="step"
		aria-label="Decrease {label ?? 'value'}"
		onclick={dec}
		{disabled}
	>−</button>
	<div class="v-wrap">
		<span class="value">{display}</span>
		{#if unit}<span class="unit">{unit}</span>{/if}
	</div>
	<button
		type="button"
		class="step"
		aria-label="Increase {label ?? 'value'}"
		onclick={inc}
		{disabled}
	>+</button>
</div>

<style>
	.stepper-pill {
		display: grid;
		grid-template-columns: 44px minmax(0, 1fr) 44px;
		align-items: stretch;
		height: 52px;
		overflow: hidden;
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 999px;
		transition:
			border-color 120ms ease,
			box-shadow 120ms ease;
	}
	.stepper-pill:focus-within {
		border-color: var(--clay);
		box-shadow: 0 0 0 3px rgba(255, 94, 31, 0.16);
	}
	.stepper-pill.disabled {
		opacity: 0.55;
	}

	.step {
		background: transparent;
		border: 0;
		padding: 0;
		cursor: pointer;
		font: 800 22px/1 'Onest', system-ui, sans-serif;
		color: var(--ink-soft);
		display: grid;
		place-items: center;
		transition:
			background-color 100ms ease,
			color 100ms ease;
	}
	.step:hover:not(:disabled) {
		background: color-mix(in oklab, var(--ink) 6%, transparent);
		color: var(--ink);
	}
	.step:active:not(:disabled) {
		background: var(--ink);
		color: var(--card);
	}
	.step:disabled {
		cursor: not-allowed;
	}

	.v-wrap {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 4px;
		min-width: 0;
		padding: 0 4px;
	}
	.value {
		font: 800 22px/1 'Onest', system-ui, sans-serif;
		color: var(--ink);
		font-variant-numeric: tabular-nums;
		letter-spacing: -0.03em;
	}
	.unit {
		font: 600 12px/1 'Onest', system-ui, sans-serif;
		color: var(--ink-soft);
		letter-spacing: 0.04em;
		text-transform: lowercase;
		transform: translateY(3px);
		flex: none;
	}
</style>
