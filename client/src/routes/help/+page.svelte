<script lang="ts">
	function resetUiPreferences() {
		if (!confirm('Reset local UI preferences? (Theme, progress selection, and legacy settings)')) {
			return;
		}

		const keys = [
			'theme',
			'progress.selectedExercise',
			'unitPreference',
			'restTimer',
			'autoEndTimeout',
			'viewDensity',
			'accentColor'
		];
		for (const key of keys) {
			try {
				localStorage.removeItem(key);
			} catch {
				// ignore local storage failures
			}
		}
		window.location.reload();
	}
</script>

<div class="space-y-6">
	<header
		class="relative overflow-hidden rounded-2xl border border-surface-200/50 dark:border-surface-700/50 bg-gradient-to-br from-primary-500/10 via-transparent to-tertiary-500/10 p-5 sm:p-6"
	>
		<div
			class="pointer-events-none absolute -top-24 -right-24 size-72 rounded-full blur-3xl bg-primary-500/15"
		></div>
		<div
			class="pointer-events-none absolute -bottom-24 -left-24 size-72 rounded-full blur-3xl bg-secondary-500/15"
		></div>

		<div class="relative flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
			<div class="space-y-1">
				<h1 class="text-3xl sm:text-4xl font-black tracking-tight">Help</h1>
				<p class="text-sm sm:text-base opacity-80 max-w-prose">
					Quick guidance on how to use SwoleMate and a safe place to reset local UI state.
				</p>
			</div>
			<div class="flex flex-wrap gap-2">
				<a href="/settings" class="btn variant-soft">Settings →</a>
				<a href="/backups" class="btn variant-soft">Data & backups →</a>
			</div>
		</div>
	</header>

	<div class="grid gap-6 md:grid-cols-12">
		<section class="md:col-span-8 space-y-4 min-w-0">
			<div class="card variant-glass-surface p-4 space-y-3">
				<div>
					<h2 class="text-lg font-semibold tracking-tight">How to use</h2>
					<p class="text-sm opacity-70">The workflow is designed to be fast and repeatable.</p>
				</div>

				<div class="space-y-3 text-sm opacity-85">
					<div
						class="rounded-xl border border-surface-200/50 bg-surface-50/60 p-3 dark:border-surface-700/50 dark:bg-surface-950/30"
					>
						<div class="font-semibold">Today</div>
						<div class="opacity-80">
							Start a session → add exercises → log sets, notes, and settings → mark exercises done
							→ end the session with your mood.
						</div>
					</div>
					<div
						class="rounded-xl border border-surface-200/50 bg-surface-50/60 p-3 dark:border-surface-700/50 dark:bg-surface-950/30"
					>
						<div class="font-semibold">History</div>
						<div class="opacity-80">
							Search and filter sessions, then review exercise settings and set schemes without
							leaving the page.
						</div>
					</div>
					<div
						class="rounded-xl border border-surface-200/50 bg-surface-50/60 p-3 dark:border-surface-700/50 dark:bg-surface-950/30"
					>
						<div class="font-semibold">Progress</div>
						<div class="opacity-80">
							Pick a focus exercise to see PRs and charts; overall cards show frequency and
							time-of-day patterns.
						</div>
					</div>
				</div>
			</div>

			<div class="card variant-glass-surface p-4 space-y-3">
				<div>
					<h2 class="text-lg font-semibold tracking-tight">Troubleshooting</h2>
					<p class="text-sm opacity-70">
						If theme or page state gets weird, a local reset usually fixes it.
					</p>
				</div>

				<button type="button" class="btn variant-soft-error w-full" on:click={resetUiPreferences}>
					Reset local UI preferences
				</button>
			</div>
		</section>

		<aside class="md:col-span-4 space-y-4 min-w-0">
			<div class="card variant-glass-surface p-4 space-y-2">
				<h2 class="text-lg font-semibold tracking-tight">Good to know</h2>
				<ul class="text-sm opacity-80 space-y-1 list-disc pl-5">
					<li>Theme is toggled from the top bar.</li>
					<li>If you use multiple devices, backups are the safest portability path.</li>
					<li>Personal MCP tokens are shown once. Rotate or revoke them if you lose one.</li>
					<li>Marked-done exercises lock editing until you press Edit.</li>
				</ul>
			</div>
		</aside>
	</div>
</div>
