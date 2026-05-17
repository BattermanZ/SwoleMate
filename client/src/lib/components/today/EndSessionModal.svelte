<script lang="ts">
	import { Btn } from '$lib/components/ui';
	import type { UiMood } from '$lib/today/types';

	interface Props {
		open: boolean;
		notes: string;
		mood: UiMood | null;
		disabled?: boolean;
		onCancel?: () => void;
		onSubmit?: () => void;
	}
	let {
		open,
		notes = $bindable(''),
		mood = $bindable(null),
		disabled = false,
		onCancel,
		onSubmit
	}: Props = $props();

	const MOODS: UiMood[] = ['😞', '😐', '😊'];
	const MOOD_LABEL: Record<UiMood, string> = {
		'😊': 'Good',
		'😐': 'Neutral',
		'😞': 'Bad'
	};
</script>

{#if open}
	<div
		class="backdrop"
		role="presentation"
		onclick={(e) => e.target === e.currentTarget && onCancel?.()}
		onkeydown={(e) => e.key === 'Escape' && onCancel?.()}
	>
		<div class="modal" role="dialog" aria-modal="true" aria-labelledby="end-session-title">
			<h3 id="end-session-title">End session</h3>
			<p>Pick how it felt and add a quick note.</p>
			<div class="moods" role="radiogroup" aria-label="Session feeling">
				{#each MOODS as m (m)}
					<button
						type="button"
						role="radio"
						class="mood"
						class:selected={mood === m}
						aria-checked={mood === m}
						aria-label={MOOD_LABEL[m]}
						onclick={() => (mood = m)}
						{disabled}>{m}</button
					>
				{/each}
			</div>
			{#if mood}<div class="mood-label">{MOOD_LABEL[mood]}</div>{/if}
			<label>
				<span class="lbl">Session notes</span>
				<textarea bind:value={notes} placeholder="Anything to remember…" {disabled}></textarea>
			</label>
			<div class="actions">
				<Btn variant="soft" onclick={onCancel} {disabled}>Cancel</Btn>
				<Btn variant="primary" onclick={onSubmit} disabled={disabled || !mood}>
					▶ Submit &amp; end
				</Btn>
			</div>
		</div>
	</div>
{/if}

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		backdrop-filter: blur(4px);
		display: grid;
		place-items: end center;
		padding: 16px;
		padding-bottom: calc(16px + env(safe-area-inset-bottom));
		z-index: 80;
	}
	@media (min-width: 640px) {
		.backdrop {
			place-items: center;
		}
	}
	.modal {
		background: var(--card);
		border-radius: 22px;
		padding: 20px;
		border: 1px solid var(--line);
		box-shadow: 0 20px 40px -12px var(--shadow-strong);
		width: 100%;
		max-width: 420px;
	}
	h3 {
		margin: 0;
		text-align: center;
		font:
			800 20px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: -0.015em;
	}
	p {
		text-align: center;
		margin: 4px 0 14px;
		font:
			500 13px/1.4 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}
	.moods {
		display: grid;
		grid-template-columns: 1fr 1fr 1fr;
		gap: 8px;
	}
	.mood {
		padding: 14px 0;
		border-radius: 14px;
		border: 1.5px solid var(--line);
		background: var(--card-3);
		font:
			400 32px/1 'Onest',
			system-ui,
			sans-serif;
		text-align: center;
		cursor: pointer;
		transition: transform 120ms ease;
	}
	.mood:active {
		transform: scale(0.97);
	}
	.mood.selected {
		background: linear-gradient(180deg, var(--clay-2), var(--clay));
		border-color: var(--clay-deep);
		color: white;
		box-shadow: 0 12px 26px -10px rgba(255, 94, 31, 0.55);
	}
	.mood-label {
		margin-top: 8px;
		font:
			700 11px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.16em;
		text-transform: uppercase;
		text-align: center;
		color: var(--ink-soft);
	}
	label {
		display: block;
		margin-top: 14px;
	}
	.lbl {
		font:
			700 10px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--ink-soft);
	}
	textarea {
		margin-top: 6px;
		width: 100%;
		min-height: 72px;
		resize: vertical;
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 12px;
		padding: 11px 12px;
		font:
			500 14px/1.4 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink);
		outline: 0;
	}
	textarea:focus {
		border-color: var(--clay);
		box-shadow: 0 0 0 3px rgba(255, 94, 31, 0.16);
	}
	.actions {
		margin-top: 14px;
		display: grid;
		grid-template-columns: 1fr 2fr;
		gap: 8px;
	}
</style>
