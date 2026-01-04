<script lang="ts">
	import { createEventDispatcher } from 'svelte';

	export let query = '';
	export let suggestions: string[] = [];
	export let quickPicks: string[] = [];
	export let disabled = false;

	const dispatch = createEventDispatcher<{
		add: { name: string };
	}>();

	function add(name: string) {
		const trimmed = name.trim();
		if (!trimmed) return;
		dispatch('add', { name: trimmed });
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key !== 'Enter') return;
		event.preventDefault();
		add(query);
	}
</script>

<div class="card variant-glass-surface p-4 space-y-3">
	<div class="flex flex-col gap-3 sm:flex-row sm:items-end">
		<label class="block flex-1">
			<span class="text-sm font-semibold opacity-80">Add exercise</span>
			<div class="relative mt-1">
				<input
					class="input w-full"
					placeholder="Search (e.g. Bench Press, Squat)…"
					bind:value={query}
					{disabled}
					autocomplete="off"
					on:keydown={handleKeydown}
				/>

				{#if suggestions.length > 0}
					<div
						role="listbox"
						aria-label="Exercise suggestions"
						class="absolute left-0 right-0 z-50 mt-2 max-h-64 overflow-auto rounded-lg border border-surface-200/60 bg-surface-50-900-token shadow-xl dark:border-surface-700/60"
					>
						{#each suggestions as name}
							<button
								type="button"
								role="option"
								aria-selected={query.trim() === name}
								class="btn variant-ghost-primary w-full justify-start rounded-none px-4 py-3"
								on:click={() => add(name)}
								{disabled}
							>
								{name}
							</button>
						{/each}
					</div>
				{/if}
			</div>
		</label>

		<button
			type="button"
			class="btn variant-filled-primary w-full sm:w-auto"
			on:click={() => add(query)}
			disabled={disabled || !query.trim()}
		>
			Add
		</button>
	</div>

	{#if quickPicks.length > 0}
		<div class="flex flex-wrap gap-2 items-center">
			<span class="text-xs font-semibold opacity-70 mr-1">Quick picks</span>
			{#each quickPicks as pick}
				<button
					type="button"
					class="chip variant-filled text-sm"
					on:click={() => add(pick)}
					{disabled}
				>
					{pick}
				</button>
			{/each}
		</div>
	{/if}
</div>
