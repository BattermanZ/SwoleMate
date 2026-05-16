<script lang="ts" generics="T extends string">
	interface TabItem {
		id: T;
		label: string;
	}

	interface Props {
		items: TabItem[];
		selected: T;
		'aria-label'?: string;
		onselect?: (id: T) => void;
	}

	let { items, selected = $bindable(), 'aria-label': ariaLabel, onselect }: Props = $props();

	function pick(id: T) {
		selected = id;
		onselect?.(id);
	}
</script>

<div class="tabs" role="tablist" aria-label={ariaLabel}>
	{#each items as item (item.id)}
		<button
			type="button"
			role="tab"
			aria-selected={selected === item.id}
			class:active={selected === item.id}
			onclick={() => pick(item.id)}
		>
			{item.label}
		</button>
	{/each}
</div>

<style>
	.tabs {
		background: var(--card);
		border: 1px solid var(--line);
		border-radius: 999px;
		padding: 4px;
		display: grid;
		grid-auto-flow: column;
		grid-auto-columns: 1fr;
		gap: 2px;
	}
	button {
		border: 0;
		background: transparent;
		padding: 10px 16px;
		font:
			700 12px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.06em;
		color: var(--ink-soft);
		cursor: pointer;
		border-radius: 999px;
		transition:
			background-color 160ms ease,
			color 160ms ease,
			box-shadow 200ms ease;
	}
	button.active {
		background: linear-gradient(180deg, var(--clay-2), var(--clay));
		color: white;
		box-shadow: 0 6px 14px -4px rgba(255, 94, 31, 0.4);
	}
</style>
