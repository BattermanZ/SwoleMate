<script lang="ts">
	interface Props {
		label: string;
		checked?: boolean;
		disabled?: boolean;
		onchange?: (v: boolean) => void;
	}

	let { label, checked = $bindable(false), disabled = false, onchange }: Props = $props();

	function toggle() {
		if (disabled) return;
		checked = !checked;
		onchange?.(checked);
	}
</script>

<button
	type="button"
	class="chk"
	class:on={checked}
	{disabled}
	role="switch"
	aria-checked={checked}
	onclick={toggle}
>
	<span class="box" aria-hidden="true"></span>
	{label}
</button>

<style>
	.chk {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 7px 10px;
		background: var(--bg-2);
		border: 1px solid transparent;
		border-radius: 999px;
		font:
			700 11px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-2);
		cursor: pointer;
		transition:
			background-color 120ms ease,
			color 120ms ease;
	}
	.chk:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
	.chk.on {
		background: var(--ink);
		color: var(--card);
	}
	.box {
		width: 14px;
		height: 14px;
		border: 1.5px solid var(--ink-dim);
		border-radius: 4px;
		display: grid;
		place-items: center;
		flex: none;
	}
	.chk.on .box {
		background: var(--clay);
		border-color: var(--clay);
	}
	.chk.on .box::after {
		content: '';
		width: 5px;
		height: 9px;
		border-right: 2px solid white;
		border-bottom: 2px solid white;
		transform: rotate(45deg) translate(-1px, -1px);
		display: block;
	}
</style>
