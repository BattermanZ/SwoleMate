<script lang="ts">
	import { Btn } from '$lib/components/ui';
	import { whatsNewEntries, closeWhatsNew } from '$lib/stores/whatsNew';

	function fmtDate(iso: string): string {
		const d = new Date(iso);
		return Number.isNaN(d.getTime())
			? iso
			: d.toLocaleDateString(undefined, { day: 'numeric', month: 'short', year: 'numeric' });
	}
</script>

{#if $whatsNewEntries}
	<div
		class="backdrop"
		role="presentation"
		onclick={(e) => e.target === e.currentTarget && closeWhatsNew()}
		onkeydown={(e) => e.key === 'Escape' && closeWhatsNew()}
	>
		<div class="modal" role="dialog" aria-modal="true" aria-labelledby="whatsnew-title">
			<h3 id="whatsnew-title">What's New</h3>

			<div class="entries">
				{#each $whatsNewEntries as entry (entry.version)}
					<section class="entry">
						<div class="ver">v{entry.version} · {fmtDate(entry.date)}</div>
						<h4>{entry.title}</h4>
						<ul>
							{#each entry.items as item (item)}
								<li>{item}</li>
							{/each}
						</ul>
					</section>
				{/each}
			</div>

			<div class="actions">
				<Btn variant="primary" onclick={closeWhatsNew}>Got it</Btn>
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
		max-height: min(80dvh, 640px);
		display: flex;
		flex-direction: column;
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
	.entries {
		margin-top: 16px;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 18px;
	}
	.entry {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.ver {
		font:
			700 10px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: 0.14em;
		text-transform: uppercase;
		color: var(--clay-text, var(--clay));
	}
	h4 {
		margin: 0;
		font:
			800 15px/1.2 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: -0.01em;
		color: var(--ink);
	}
	ul {
		margin: 0;
		padding: 0;
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	li {
		position: relative;
		padding-left: 18px;
		font:
			500 13px/1.5 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}
	li::before {
		content: '';
		position: absolute;
		left: 2px;
		top: 7px;
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--clay);
	}
	.actions {
		margin-top: 18px;
	}
	.actions :global(button) {
		width: 100%;
	}
</style>
