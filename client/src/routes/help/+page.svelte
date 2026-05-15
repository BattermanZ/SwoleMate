<script lang="ts">
	import { Btn, Card, PageHero } from '$lib/components/ui';

	function resetUiPreferences() {
		if (!confirm('Reset local UI preferences? (Theme, progress selection, legacy settings)')) {
			return;
		}
		const keys = [
			'theme',
			'progress.selectedExercise',
			'progress.selectedTab',
			'unitPreference',
			'restTimer',
			'autoEndTimeout',
			'viewDensity',
			'accentColor'
		];
		for (const k of keys) {
			try {
				localStorage.removeItem(k);
			} catch {
				/* ignore */
			}
		}
		window.location.reload();
	}
</script>

<div class="page">
	<PageHero kicker="► Help">
		{#snippet title()}Quick guidance, <em>safe resets.</em>{/snippet}
		{#snippet sub()}How to use SwoleMate, plus a place to clear local UI state if something feels stuck.{/snippet}
	</PageHero>

	<Card>
		{#snippet title()}How to use{/snippet}
		{#snippet lede()}The workflow is built to be fast and repeatable.{/snippet}
		<div class="walk">
			<div class="step">
				<div class="t">Today</div>
				<div>
					Start a session → add exercises → log sets, notes, and settings → mark exercises done →
					end the session with your mood.
				</div>
			</div>
			<div class="step">
				<div class="t">History</div>
				<div>
					Search and filter past sessions, then review set schemes and notes without leaving the
					page.
				</div>
			</div>
			<div class="step">
				<div class="t">Progress</div>
				<div>
					Pick a focus exercise to see PRs and charts; overall cards show frequency and
					time-of-day patterns.
				</div>
			</div>
		</div>
	</Card>

	<Card>
		{#snippet title()}Good to know{/snippet}
		<ul>
			<li>Theme toggles from the top bar — persists across reloads.</li>
			<li>If you use multiple devices, backups are the safest portability path.</li>
			<li>Personal MCP tokens are shown once. Rotate or revoke if you lose one.</li>
			<li>Marked-done exercises lock editing until you tap Edit.</li>
		</ul>
	</Card>

	<Card>
		{#snippet title()}Troubleshooting{/snippet}
		{#snippet lede()}If the theme or page state gets weird, a local reset usually fixes it.{/snippet}
		<Btn variant="ink" onclick={resetUiPreferences}>↻ Reset local UI preferences</Btn>
	</Card>
</div>

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.walk {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	.step {
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 12px;
		padding: 12px;
	}
	.step .t {
		font: 800 14px/1 'Onest', system-ui, sans-serif;
		margin-bottom: 4px;
		letter-spacing: -0.01em;
	}
	.step div:last-child {
		font: 500 13px/1.5 'Onest', system-ui, sans-serif;
		color: var(--ink-2);
	}
	ul {
		margin: 0;
		padding-left: 18px;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	li {
		font: 500 14px/1.5 'Onest', system-ui, sans-serif;
		color: var(--ink-2);
	}
</style>
