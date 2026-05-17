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
	let editing = $state(false);
	let draft = $state('');
	let inputEl: HTMLInputElement | undefined = $state();

	function startEdit() {
		if (disabled) return;
		draft = String(value);
		editing = true;
		queueMicrotask(() => {
			inputEl?.focus();
			inputEl?.select();
		});
	}

	function commit() {
		if (!editing) return;
		editing = false;
		const parsed = Number(draft.replace(',', '.'));
		if (!Number.isFinite(parsed)) return;
		const next = clamp(parsed);
		if (next === value) return;
		value = next;
		onchange?.(next);
	}

	function cancel() {
		editing = false;
	}

	function onKey(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			inputEl?.blur();
		} else if (e.key === 'Escape') {
			e.preventDefault();
			draft = String(value);
			cancel();
		}
	}
</script>

<div class="stepper-pill" class:disabled aria-label={ariaLabel ?? label}>
	<button
		type="button"
		class="step"
		aria-label="Decrease {label ?? 'value'}"
		onclick={dec}
		{disabled}>−</button
	>
	<div class="v-wrap">
		{#if editing}
			<input
				bind:this={inputEl}
				class="value value-input"
				type="text"
				inputmode="decimal"
				bind:value={draft}
				onblur={commit}
				onkeydown={onKey}
				aria-label={ariaLabel ?? label}
			/>
		{:else}
			<button
				type="button"
				class="value value-button"
				onclick={startEdit}
				{disabled}
				aria-label={`Edit ${label ?? 'value'}`}>{display}</button
			>
		{/if}
		{#if unit}<span class="unit">{unit}</span>{/if}
	</div>
	<button
		type="button"
		class="step"
		aria-label="Increase {label ?? 'value'}"
		onclick={inc}
		{disabled}>+</button
	>
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
		font:
			800 22px/1 'Onest',
			system-ui,
			sans-serif;
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
		font:
			800 22px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink);
		font-variant-numeric: tabular-nums;
		letter-spacing: -0.03em;
	}
	.value-button {
		background: transparent;
		border: 0;
		padding: 4px 6px;
		margin: 0;
		cursor: text;
		border-radius: 8px;
		min-width: 0;
		text-align: center;
	}
	.value-button:hover:not(:disabled) {
		background: color-mix(in oklab, var(--ink) 6%, transparent);
	}
	.value-button:disabled {
		cursor: not-allowed;
	}
	.value-input {
		background: transparent;
		border: 0;
		outline: 0;
		padding: 4px 6px;
		width: 100%;
		min-width: 0;
		text-align: center;
		-moz-appearance: textfield;
		appearance: textfield;
	}
	.value-input::-webkit-outer-spin-button,
	.value-input::-webkit-inner-spin-button {
		-webkit-appearance: none;
		margin: 0;
	}
	.unit {
		font:
			600 12px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
		letter-spacing: 0.04em;
		text-transform: lowercase;
		transform: translateY(3px);
		flex: none;
	}
</style>
