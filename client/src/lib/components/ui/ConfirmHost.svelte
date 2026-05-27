<script lang="ts">
	import { Btn } from '$lib/components/ui';
	import { confirmRequest } from '$lib/stores/confirm';

	let inputValue = $state('');

	// Seed the input each time a new request opens.
	let lastReq: unknown = null;
	$effect(() => {
		const req = $confirmRequest;
		if (req !== lastReq) {
			lastReq = req;
			inputValue = req?.input?.value ?? '';
		}
	});

	function confirm() {
		const req = $confirmRequest;
		if (!req) return;
		confirmRequest.set(null);
		req.resolve(req.input ? inputValue.trim() : true);
	}

	function cancel() {
		const req = $confirmRequest;
		if (!req) return;
		confirmRequest.set(null);
		req.resolve(req.input ? null : false);
	}
</script>

{#if $confirmRequest}
	{@const req = $confirmRequest}
	<div
		class="backdrop"
		role="presentation"
		onclick={(e) => e.target === e.currentTarget && cancel()}
		onkeydown={(e) => e.key === 'Escape' && cancel()}
	>
		<div class="modal" role="dialog" aria-modal="true" aria-labelledby="confirm-title">
			<h3 id="confirm-title">{req.title}</h3>
			{#if req.message}<p>{req.message}</p>{/if}

			{#if req.input}
				<label>
					{#if req.input.label}<span class="lbl">{req.input.label}</span>{/if}
					<!-- svelte-ignore a11y_autofocus -->
					<input
						type="text"
						bind:value={inputValue}
						placeholder={req.input.placeholder ?? ''}
						autofocus
						onkeydown={(e) => e.key === 'Enter' && confirm()}
					/>
				</label>
			{/if}

			<div class="actions">
				<Btn variant="soft" onclick={cancel}>{req.cancelLabel}</Btn>
				<Btn
					variant={req.danger ? 'ink' : 'primary'}
					onclick={confirm}
					disabled={!!req.input && inputValue.trim().length === 0}
				>
					{req.confirmLabel}
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
		z-index: 90;
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
		margin: 8px 0 0;
		font:
			500 13px/1.45 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}
	label {
		display: block;
		margin-top: 14px;
	}
	.lbl {
		display: block;
		font:
			700 10px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--ink-soft);
		margin-bottom: 6px;
	}
	input {
		width: 100%;
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 12px;
		padding: 11px 12px;
		font:
			500 14px/1.2 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink);
		outline: 0;
	}
	input:focus {
		border-color: var(--clay);
		box-shadow: 0 0 0 3px rgba(255, 94, 31, 0.16);
	}
	.actions {
		margin-top: 18px;
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 8px;
	}
</style>
