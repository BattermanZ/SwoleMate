<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { FEEDBACK_OPTIONS } from '$lib/mocks/today';
	import type { UiMood } from '$lib/today/types';

	export let open = false;
	export let notes = '';
	export let mood: UiMood | null = null;
	export let disabled = false;

	const dispatch = createEventDispatcher<{
		cancel: undefined;
		submit: undefined;
	}>();
</script>

{#if open}
	<div class="modal-backdrop fixed inset-0 z-50 bg-black/50 flex items-center justify-center">
		<div class="card variant-filled-surface p-4 w-full max-w-lg mx-4 space-y-4">
			<header class="text-center space-y-1">
				<h3 class="h3">End session</h3>
				<p class="text-sm opacity-70">Pick how it felt and add a quick note.</p>
			</header>

			<div class="flex justify-center gap-3">
				{#each FEEDBACK_OPTIONS as emoji}
					<button
						type="button"
						class="card {mood === emoji
							? 'variant-filled-primary'
							: 'variant-soft'} p-3 text-3xl hover:scale-105 transition-transform"
						on:click={() => (mood = emoji)}
						{disabled}
						aria-pressed={mood === emoji}
					>
						{emoji}
					</button>
				{/each}
			</div>

			<label class="block">
				<span class="text-sm font-semibold opacity-80">Session notes</span>
				<textarea
					class="textarea mt-1"
					rows="3"
					placeholder="Anything to remember for next time…"
					bind:value={notes}
					{disabled}
				></textarea>
			</label>

			<footer class="flex justify-end gap-2">
				<button
					type="button"
					class="btn variant-soft"
					on:click={() => dispatch('cancel')}
					{disabled}
				>
					Cancel
				</button>
				<button
					type="button"
					class="btn variant-filled-primary"
					on:click={() => dispatch('submit')}
					disabled={disabled || !mood}
				>
					Submit
				</button>
			</footer>
		</div>
	</div>
{/if}
