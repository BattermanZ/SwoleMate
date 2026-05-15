<script lang="ts">
	import { Btn } from '$lib/components/ui';
	import type { PlannedTemplateExercise } from '$lib/today/types';

	interface Props {
		query: string;
		suggestions?: string[];
		templatePicks?: PlannedTemplateExercise[];
		quickPicks?: string[];
		disabled?: boolean;
		onAdd?: (name: string) => void;
		onAddTemplateExercise?: (id: number) => void;
	}
	let {
		query = $bindable(''),
		suggestions = [],
		templatePicks = [],
		quickPicks = [],
		disabled = false,
		onAdd,
		onAddTemplateExercise
	}: Props = $props();

	function add(name: string) {
		const trimmed = name.trim();
		if (!trimmed) return;
		onAdd?.(trimmed);
	}

	function onKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			add(query);
		}
	}
</script>

<section class="composer">
	<div class="row">
		<input
			class="search"
			placeholder="Search (e.g. Bench Press, Squat)…"
			bind:value={query}
			{disabled}
			autocomplete="off"
			onkeydown={onKeydown}
			aria-label="Add exercise"
		/>
		<Btn variant="ink" onclick={() => add(query)} disabled={disabled || !query.trim()}>Add</Btn>
	</div>

	{#if suggestions.length > 0}
		<div class="suggestions" role="listbox" aria-label="Exercise suggestions">
			{#each suggestions as name (name)}
				<button
					type="button"
					role="option"
					aria-selected={query.trim() === name}
					onclick={() => add(name)}
					{disabled}
				>
					{name}
				</button>
			{/each}
		</div>
	{/if}

	{#if templatePicks.length > 0}
		<div class="template-plan">
			<div class="top">
				<span class="lbl">▸ Template plan</span>
				<span class="count">{templatePicks.length} left</span>
			</div>
			<div class="picks">
				{#each templatePicks as pick (pick.id)}
					<button
						type="button"
						class="pick"
						onclick={() => onAddTemplateExercise?.(pick.id)}
						{disabled}
					>
						{pick.name}
					</button>
				{/each}
			</div>
		</div>
	{/if}

	{#if quickPicks.length > 0}
		<div class="quick-picks">
			<span class="lbl">Quick picks</span>
			{#each quickPicks as pick (pick)}
				<button type="button" class="chip" onclick={() => add(pick)} {disabled}>{pick}</button>
			{/each}
		</div>
	{/if}
</section>

<style>
	.composer {
		background: var(--card);
		border: 1px solid var(--line);
		border-radius: 20px;
		padding: 14px;
	}
	.row {
		display: grid;
		grid-template-columns: 1fr auto;
		gap: 8px;
	}
	.search {
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 12px;
		padding: 12px 14px;
		font: 500 14px/1.2 'Onest', system-ui, sans-serif;
		color: var(--ink);
		outline: 0;
	}
	.search:focus {
		border-color: var(--clay);
		box-shadow: 0 0 0 3px rgba(255, 94, 31, 0.16);
	}
	.suggestions {
		margin-top: 8px;
		border: 1px solid var(--line);
		border-radius: 12px;
		overflow: hidden;
		background: var(--card);
	}
	.suggestions button {
		width: 100%;
		text-align: left;
		padding: 12px 14px;
		background: transparent;
		border: 0;
		border-bottom: 1px solid var(--line);
		font: 600 13px/1 'Onest', system-ui, sans-serif;
		color: var(--ink-2);
		cursor: pointer;
	}
	.suggestions button:last-child {
		border-bottom: 0;
	}
	.suggestions button:hover {
		background: var(--card-3);
		color: var(--clay-text);
	}

	.template-plan {
		margin-top: 12px;
		padding: 12px;
		border-radius: 14px;
		background: color-mix(in oklab, var(--clay) 6%, var(--card));
		border: 1px solid color-mix(in oklab, var(--clay) 22%, var(--line));
	}
	.top {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}
	.template-plan .lbl {
		font: 700 10px/1 'Onest', system-ui, sans-serif;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--clay-text);
	}
	.count {
		font: 700 10px/1 'Onest', system-ui, sans-serif;
		color: var(--clay-text);
		background: color-mix(in oklab, var(--clay) 12%, var(--card));
		padding: 4px 8px;
		border-radius: 999px;
	}
	.picks {
		margin-top: 8px;
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}
	.pick {
		background: var(--card);
		border: 1px solid color-mix(in oklab, var(--clay) 32%, var(--line));
		color: var(--clay-text);
		padding: 7px 11px;
		border-radius: 999px;
		font: 700 12px/1 'Onest', system-ui, sans-serif;
		cursor: pointer;
	}
	.pick::before {
		content: '+ ';
	}

	.quick-picks {
		margin-top: 12px;
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		align-items: center;
	}
	.quick-picks .lbl {
		font: 700 10px/1 'Onest', system-ui, sans-serif;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--ink-soft);
		margin-right: 4px;
	}
	.chip {
		padding: 7px 12px;
		border-radius: 999px;
		background: var(--bg-2);
		color: var(--ink-2);
		font: 700 12px/1 'Onest', system-ui, sans-serif;
		cursor: pointer;
		border: 0;
	}
</style>
