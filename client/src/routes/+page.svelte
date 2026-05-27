<script lang="ts">
	import { onMount } from 'svelte';
	import { getWorkoutTemplates } from '$lib/api';
	import { readDemoModePreference } from '$lib/preferences/demoMode';
	import { createTodayController } from '$lib/today/controller';
	import type { WorkoutTemplate } from '$lib/types';
	import { formatTime, formatDateRelative } from '$lib/utils/date';
	import { Btn, Card, Notice } from '$lib/components/ui';
	import SessionHero from '$lib/components/today/SessionHero.svelte';
	import NoSessionState from '$lib/components/today/NoSessionState.svelte';
	import SessionExercise from '$lib/components/today/SessionExercise.svelte';
	import ExerciseComposer from '$lib/components/today/ExerciseComposer.svelte';
	import RecentSessions from '$lib/components/today/RecentSessions.svelte';
	import EndSessionModal from '$lib/components/today/EndSessionModal.svelte';
	import TodayDesktop from '$lib/components/today/TodayDesktop.svelte';
	import { isDesktop, isDesktopView } from '$lib/stores/viewport';

	const c = createTodayController();

	let desktop = $derived(isDesktopView($isDesktop));

	function sessionTitle(startedAt: string | undefined): { lead: string; accent: string } {
		const hour = startedAt ? new Date(startedAt).getHours() : new Date().getHours();
		if (hour >= 5 && hour < 12) return { lead: 'Morning', accent: 'grind.' };
		if (hour >= 12 && hour < 17) return { lead: 'Afternoon', accent: 'session.' };
		if (hour >= 17 && hour < 22) return { lead: 'Evening', accent: 'lift.' };
		return { lead: 'Late', accent: 'session.' };
	}

	// Pull stores out of the controller so the auto-subscribe $-prefix works cleanly in the template.
	const error = c.error;
	const notice = c.notice;
	const loading = c.loading;
	const offlineMode = c.offlineMode;
	const pendingSyncCount = c.pendingSyncCount;
	const currentSession = c.currentSession;
	const elapsedLabel = c.elapsedLabel;
	const totalSets = c.totalSets;
	const totalVolumeKg = c.totalVolumeKg;
	const totalDurationSeconds = c.totalDurationSeconds;
	const exerciseQuery = c.exerciseQuery;
	const sessionNotes = c.sessionNotes;
	const endModalOpen = c.endModalOpen;
	const endMood = c.endMood;
	const endNotes = c.endNotes;
	const suggestions = c.suggestions;
	const quickPicks = c.quickPicks;
	const plannedTemplateExercises = c.plannedTemplateExercises;
	const recentSessions = c.recentSessions;
	const openExerciseIds = c.openExerciseIds;

	let showDemoAction = $state(false);
	let templatePickerOpen = $state(false);
	let composerEl = $state<HTMLElement | null>(null);
	let composerPulsing = $state(false);
	let templateLoading = $state(false);
	let templateError = $state<string | null>(null);
	let templates = $state<WorkoutTemplate[]>([]);

	onMount(() => {
		showDemoAction = readDemoModePreference();
		return c.start();
	});

	async function openTemplatePicker() {
		templatePickerOpen = true;
		templateError = null;
		templateLoading = true;
		try {
			templates = await getWorkoutTemplates();
		} catch (e) {
			templateError = e instanceof Error ? e.message : 'Failed to load templates';
		} finally {
			templateLoading = false;
		}
	}

	async function handleStartFromTemplate(templateId: number) {
		await c.startSessionFromTemplate(templateId);
		// `c.error` is the raw store, so peek via $- in the template; here we just close optimistically.
		templatePickerOpen = false;
	}

	function markDoneAndScroll(exerciseId: number) {
		c.markExerciseDone(exerciseId);
		composerEl?.scrollIntoView({ behavior: 'smooth', block: 'start' });
		setTimeout(() => {
			composerPulsing = true;
			setTimeout(() => (composerPulsing = false), 650);
		}, 380);
	}
</script>

{#snippet notices()}
	{#if $notice || $pendingSyncCount}
		<Notice
			tone="info"
			action={$pendingSyncCount
				? {
						label: 'Sync now',
						onclick: c.syncPendingSessions,
						disabled: $loading || $offlineMode
					}
				: undefined}
		>
			{#if $notice}{$notice}{/if}
			{#if $pendingSyncCount}
				<span style="opacity: 0.8;">
					{#if $notice}&nbsp;·&nbsp;{/if}
					{$pendingSyncCount} change{$pendingSyncCount === 1 ? '' : 's'} pending sync
				</span>
			{/if}
		</Notice>
	{/if}

	{#if $error}
		<Notice tone="error">{$error}</Notice>
	{/if}
{/snippet}

{#snippet hero()}
	{#if $currentSession}
		<SessionHero
			elapsedLabel={$elapsedLabel}
			exerciseCount={$currentSession.exercises.length}
			exercisesPlanned={$plannedTemplateExercises.length + $currentSession.exercises.length}
			setCount={$totalSets}
			volumeKg={$totalVolumeKg}
			durationSeconds={$totalDurationSeconds}
			startedAtLabel={`${formatTime($currentSession.startedAt)} · ${formatDateRelative($currentSession.startedAt)}`}
			titleLead={sessionTitle($currentSession.startedAt).lead}
			titleAccent={sessionTitle($currentSession.startedAt).accent}
			onCancel={c.cancelSession}
			onEnd={c.openEndModal}
			disabled={$loading}
		/>
	{:else}
		<NoSessionState
			{showDemoAction}
			offlineMode={$offlineMode}
			loading={$loading}
			onStart={() => c.startSession('empty')}
			onUseTemplate={openTemplatePicker}
			onDemo={() => c.startSession('demo')}
		/>
	{/if}
{/snippet}

{#snippet templatePicker()}
	{#if templatePickerOpen && !$currentSession}
		<Card>
			{#snippet title()}Start from template <em>— preloads exercise plan</em>{/snippet}
			{#snippet actions()}
				<Btn variant="soft" size="sm" onclick={() => (templatePickerOpen = false)}>Close</Btn>
			{/snippet}

			{#if templateError}
				<Notice tone="error">{templateError}</Notice>
			{:else if templateLoading}
				<div class="muted">Loading templates…</div>
			{:else if templates.length === 0}
				<div class="muted">No templates yet. Create one from the Plans page first.</div>
			{:else}
				<div class="t-grid">
					{#each templates as t (t.id)}
						<button
							class="t-card"
							type="button"
							onclick={() => handleStartFromTemplate(t.id)}
							disabled={$loading}
						>
							<div class="t-name">{t.name}</div>
							<div class="t-count">
								{t.exercise_count} exercise{t.exercise_count === 1 ? '' : 's'}
							</div>
						</button>
					{/each}
				</div>
			{/if}
		</Card>
	{/if}
{/snippet}

{#snippet primary()}
	{#if $currentSession}
		<Card>
			<label class="notes-label">
				<span class="lbl">Session notes</span>
				<textarea
					bind:value={$sessionNotes}
					rows="2"
					placeholder="How did it feel? Any cues to remember…"
				></textarea>
			</label>
		</Card>

		{#if $currentSession.exercises.length === 0}
			<Card>
				<div class="empty">
					{#if $plannedTemplateExercises.length > 0}
						<div class="t">Start your template plan</div>
						<p>Tap an exercise from your template plan below.</p>
					{:else}
						<div class="t">Add your first exercise</div>
						<p>Use the search below or tap a quick pick from your recent sessions.</p>
					{/if}
				</div>
			</Card>
		{:else}
			<div class="ex-list">
				{#each $currentSession.exercises as ex (ex.id)}
					<SessionExercise
						exercise={ex}
						isOpen={$openExerciseIds.includes(ex.id)}
						disabled={$loading}
						lastTime={c.getLastTimeForExercise(ex.name)}
						onToggle={() => c.toggleExercise(ex.id)}
						onDelete={() => c.removeExercise(ex.id)}
						onMarkDone={() => markDoneAndScroll(ex.id)}
						onAddSet={(p) =>
							c.addSet(ex.id, p.reps, p.weight, p.weightLeft, p.weightRight, p.durationSeconds)}
						onUpdateSet={(setId, p) =>
							c.updateSet(ex.id, setId, {
								reps: p.reps,
								weight: p.weight,
								weightLeft: p.weightLeft,
								weightRight: p.weightRight,
								durationSeconds: p.durationSeconds
							})}
						onRemoveSet={(setId) => c.removeSet(ex.id, setId)}
						onUpdateNotes={(n) => c.updateExerciseNotes(ex.id, n)}
						onAddSetting={(k, v) => c.addExerciseSetting(ex.id, k, v)}
						onRemoveSetting={(id) => c.removeExerciseSetting(ex.id, id)}
						onUpdateSetting={(id, k, v) => c.updateExerciseSetting(ex.id, id, k, v)}
						onTogglePerSideWeight={(e) => c.toggleExercisePerSideWeight(ex.id, e)}
						onToggleSplitWeight={(e) => c.toggleExerciseSplitWeight(ex.id, e)}
						onUpdateTracking={(t) => c.updateExerciseTracking(ex.id, t)}
					/>
				{/each}
			</div>
		{/if}

		<div bind:this={composerEl} class:composer-pulse={composerPulsing}>
			<ExerciseComposer
				bind:query={$exerciseQuery}
				suggestions={$suggestions}
				templatePicks={$plannedTemplateExercises}
				quickPicks={$quickPicks}
				disabled={$loading}
				onAdd={(name) => c.addExercise(name)}
				onAddTemplateExercise={(id) => c.startPlannedTemplateExercise(id)}
			/>
		</div>
	{/if}
{/snippet}

{#snippet recall()}
	<RecentSessions
		sessions={$recentSessions}
		canAdd={Boolean($currentSession) && !$loading}
		disabled={$loading || !$currentSession}
		onAddExercise={(p) =>
			c.addExercise(p.name, {
				perSideWeight: p.perSideWeight,
				splitWeight: p.splitWeight,
				tracksReps: p.tracksReps,
				tracksTime: p.tracksTime,
				tracksWeight: p.tracksWeight,
				settings: p.settings
			})}
	/>
{/snippet}

{#if desktop}
	<TodayDesktop
		hasSession={Boolean($currentSession)}
		{notices}
		{hero}
		{templatePicker}
		{primary}
		{recall}
	/>
{:else}
	<div class="page">
		{@render notices()}
		{@render hero()}
		{@render templatePicker()}
		{@render primary()}
		{@render recall()}
	</div>
{/if}

<EndSessionModal
	open={$endModalOpen}
	bind:notes={$endNotes}
	bind:mood={$endMood}
	disabled={$loading}
	onCancel={() => endModalOpen.set(false)}
	onSubmit={c.submitEndSession}
/>

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.notes-label {
		display: block;
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
		margin-top: 8px;
		width: 100%;
		min-height: 56px;
		resize: vertical;
		background: transparent;
		border: 0;
		outline: 0;
		color: var(--ink);
		font:
			500 14px/1.45 'Onest',
			system-ui,
			sans-serif;
	}

	.muted {
		opacity: 0.7;
		font:
			500 13px/1.4 'Onest',
			system-ui,
			sans-serif;
	}

	.empty {
		text-align: center;
		padding: 14px 0;
	}
	.empty .t {
		font:
			800 18px/1 'Onest',
			system-ui,
			sans-serif;
		letter-spacing: -0.015em;
	}
	.empty p {
		margin: 6px 0 0;
		font:
			500 13px/1.5 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}

	.ex-list {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.t-grid {
		margin-top: 4px;
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 8px;
	}
	.t-card {
		text-align: left;
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 14px;
		padding: 12px;
		cursor: pointer;
		color: inherit;
	}
	.t-card:hover {
		border-color: var(--clay);
	}
	.t-name {
		font:
			800 14px/1.1 'Onest',
			system-ui,
			sans-serif;
	}
	.t-count {
		margin-top: 4px;
		font:
			500 12px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}

	@keyframes composer-glow {
		0% {
			box-shadow: 0 0 0 0 color-mix(in oklab, var(--clay) 0%, transparent);
			border-radius: 18px;
		}
		25% {
			box-shadow: 0 0 0 5px color-mix(in oklab, var(--clay) 38%, transparent);
			border-radius: 18px;
		}
		100% {
			box-shadow: 0 0 0 0 color-mix(in oklab, var(--clay) 0%, transparent);
			border-radius: 18px;
		}
	}
	.composer-pulse {
		animation: composer-glow 650ms ease-out both;
	}
</style>
