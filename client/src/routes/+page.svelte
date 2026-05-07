<script lang="ts">
	import { onMount } from 'svelte';
	import { getWorkoutTemplates } from '$lib/api';
	import ExerciseComposer from '$lib/components/today/ExerciseComposer.svelte';
	import { readDemoModePreference } from '$lib/preferences/demoMode';
	import EndSessionModal from '$lib/components/today/EndSessionModal.svelte';
	import RecentSessions from '$lib/components/today/RecentSessions.svelte';
	import SessionExercise from '$lib/components/today/SessionExercise.svelte';
	import { createTodayController } from '$lib/today/controller';
	import type { WorkoutTemplate } from '$lib/types';
	import { formatDateRelative, formatTime } from '$lib/utils/date';

	const controller = createTodayController();

	const {
		addExercise,
		addExerciseSetting,
		addSet,
		cancelSession,
		currentSession,
		elapsedLabel,
		endModalOpen,
		endMood,
		endNotes,
		error,
		exerciseQuery,
		getLastTimeForExercise,
		loading,
		notice,
		offlineMode,
		markExerciseDone,
		openEndModal,
		openExerciseId,
		plannedTemplateExercises,
		pendingSyncCount,
		quickPicks,
		recentSessions,
		removeExercise,
		removeExerciseSetting,
		sessionNotes,
		start,
		startSession,
		startSessionFromTemplate,
		startPlannedTemplateExercise,
		submitEndSession,
		syncPendingSessions,
		suggestions,
		toggleExercise,
		toggleExercisePerSideWeight,
		toggleExerciseSplitWeight,
		totalSets,
		totalDurationSeconds,
		totalVolumeKg,
		updateExerciseNotes,
		updateExerciseSetting,
		updateExerciseTracking
	} = controller;

	let showDemoAction = false;
	let templatePickerOpen = false;
	let templateLoading = false;
	let templateError: string | null = null;
	let templates: WorkoutTemplate[] = [];

	onMount(() => {
		showDemoAction = readDemoModePreference();
		return start();
	});

	async function openTemplatePicker() {
		templatePickerOpen = true;
		templateError = null;
		templateLoading = true;

		try {
			templates = await getWorkoutTemplates();
		} catch (error) {
			templateError = error instanceof Error ? error.message : 'Failed to load templates';
		} finally {
			templateLoading = false;
		}
	}

	async function handleStartFromTemplate(templateId: number) {
		await startSessionFromTemplate(templateId);
		if (!$error) templatePickerOpen = false;
	}

	function formatSetDuration(seconds: number): string {
		const value = Math.max(0, Math.round(seconds));
		const minutes = Math.floor(value / 60);
		const remaining = value % 60;
		if (minutes < 60) return `${minutes}:${String(remaining).padStart(2, '0')}`;
		const hours = Math.floor(minutes / 60);
		const remMinutes = minutes % 60;
		return remMinutes ? `${hours}h ${remMinutes}m` : `${hours}h`;
	}
</script>

<div class="space-y-6">
	{#if $notice || $pendingSyncCount}
		<div
			class="card variant-glass-surface p-3 flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between"
		>
			<div class="text-sm">
				{#if $notice}{$notice}{/if}
				{#if $pendingSyncCount}
					<span class="opacity-80">
						&nbsp;•&nbsp;{$pendingSyncCount} change{$pendingSyncCount === 1 ? '' : 's'} pending sync
					</span>
				{/if}
			</div>
			{#if $pendingSyncCount}
				<button
					type="button"
					class="btn btn-sm variant-filled-secondary w-full sm:w-auto"
					on:click={syncPendingSessions}
					disabled={$loading || $offlineMode}
				>
					Sync now
				</button>
			{/if}
		</div>
	{/if}

	{#if $error}
		<div class="alert variant-filled-error">{$error}</div>
	{/if}

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
				<h1 class="text-3xl sm:text-4xl font-black tracking-tight">Today</h1>
				<p class="text-sm sm:text-base opacity-80 max-w-prose">
					Log your current session with quick notes per exercise — and keep the last two sessions
					visible for instant recall.
				</p>
			</div>

			<div class="flex flex-col sm:items-end gap-2">
				{#if $currentSession}
					<div class="text-sm opacity-80">
						Started {formatTime($currentSession.startedAt)} • {formatDateRelative(
							$currentSession.startedAt
						)}
					</div>
					<div class="flex gap-2 w-full sm:w-auto">
						<button
							type="button"
							class="btn variant-soft-error flex-1 sm:flex-initial"
							on:click={cancelSession}
							disabled={$loading}
						>
							Cancel
						</button>
						<button
							type="button"
							class="btn variant-filled-primary flex-1 sm:flex-initial"
							on:click={openEndModal}
							disabled={$loading}
						>
							End session
						</button>
					</div>
				{:else}
					<div class="flex flex-col sm:flex-row gap-2 w-full sm:w-auto">
						<button
							type="button"
							class="btn variant-filled-primary w-full sm:w-auto"
							on:click={() => startSession('empty')}
							disabled={$loading}
						>
							Start session
						</button>
						<button
							type="button"
							class="btn variant-soft w-full sm:w-auto"
							on:click={openTemplatePicker}
							disabled={$loading || $offlineMode}
						>
							Use template
						</button>
						{#if showDemoAction}
							<button
								type="button"
								class="btn variant-soft w-full sm:w-auto"
								on:click={() => startSession('demo')}
								disabled={$loading}
							>
								Load demo
							</button>
						{/if}
					</div>
				{/if}
			</div>
		</div>

		{#if $currentSession}
			<div class="relative mt-5 grid gap-3 grid-cols-2 sm:grid-cols-4">
				<div class="card variant-glass-surface p-3">
					<div class="text-xs font-semibold opacity-70">Elapsed</div>
					<div class="text-lg font-bold">{$elapsedLabel}</div>
				</div>
				<div class="card variant-glass-surface p-3">
					<div class="text-xs font-semibold opacity-70">Exercises</div>
					<div class="text-lg font-bold">{$currentSession.exercises.length}</div>
				</div>
				<div class="card variant-glass-surface p-3">
					<div class="text-xs font-semibold opacity-70">Sets</div>
					<div class="text-lg font-bold">{$totalSets}</div>
				</div>
				<div class="card variant-glass-surface p-3">
					<div class="text-xs font-semibold opacity-70">
						{$totalVolumeKg > 0 ? 'Volume' : $totalDurationSeconds > 0 ? 'Time' : 'Volume'}
					</div>
					<div class="text-lg font-bold">
						{$totalVolumeKg > 0
							? `${Math.round($totalVolumeKg)} kg`
							: $totalDurationSeconds > 0
								? formatSetDuration($totalDurationSeconds)
								: '0 kg'}
					</div>
					{#if $totalVolumeKg > 0 && $totalDurationSeconds > 0}
						<div class="text-xs opacity-65">{formatSetDuration($totalDurationSeconds)} timed</div>
					{/if}
				</div>
			</div>
		{/if}
	</header>

	{#if templatePickerOpen && !$currentSession}
		<section class="card variant-glass-surface p-4 space-y-3">
			<div class="flex items-start justify-between gap-3">
				<div>
					<h2 class="text-lg font-semibold">Start from template</h2>
					<p class="text-sm opacity-75">
						Templates preload your exercise plan without carrying over reps or weight.
					</p>
				</div>
				<button
					type="button"
					class="btn btn-sm variant-soft"
					on:click={() => (templatePickerOpen = false)}
					disabled={$loading}
				>
					Close
				</button>
			</div>

			{#if templateError}
				<div class="alert variant-filled-error">{templateError}</div>
			{:else if templateLoading}
				<div class="text-sm opacity-70">Loading templates…</div>
			{:else if templates.length === 0}
				<div class="card variant-ghost p-4 text-sm opacity-80">
					No templates yet. Create one from the Templates page or save a past workout as a template.
				</div>
			{:else}
				<div class="grid gap-3 sm:grid-cols-2">
					{#each templates as template (template.id)}
						<button
							type="button"
							class="text-left rounded-2xl border border-surface-200/50 bg-surface-50/70 p-4 transition hover:border-primary-500/50 hover:bg-surface-100/70 dark:border-surface-700/50 dark:bg-surface-950/30"
							on:click={() => handleStartFromTemplate(template.id)}
							disabled={$loading}
						>
							<div class="font-semibold">{template.name}</div>
							<div class="text-sm opacity-70">
								{template.exercise_count} exercise{template.exercise_count === 1 ? '' : 's'}
							</div>
						</button>
					{/each}
				</div>
			{/if}
		</section>
	{/if}

	<div class="grid gap-6 md:grid-cols-12">
		<section class="md:col-span-7 lg:col-span-8 space-y-4 min-w-0">
			{#if $currentSession}
				<div class="card variant-glass-surface p-4 space-y-2">
					<label class="block">
						<span class="text-sm font-semibold opacity-80">Session notes</span>
						<textarea
							class="textarea mt-1"
							rows="2"
							placeholder="How did it feel? Any cues to remember…"
							bind:value={$sessionNotes}
						></textarea>
					</label>
				</div>

				{#if $currentSession.exercises.length === 0}
					<div class="card variant-ghost p-6 text-center space-y-2">
						<div class="text-lg font-semibold">
							{$plannedTemplateExercises.length > 0
								? 'Start your template plan'
								: 'Add your first exercise'}
						</div>
						<p class="opacity-70 text-sm max-w-prose mx-auto">
							{$plannedTemplateExercises.length > 0
								? 'Start an exercise from your template plan below.'
								: 'Use the search below or tap a quick pick from your recent sessions.'}
						</p>
					</div>
				{:else}
					<div class="space-y-3">
						{#each $currentSession.exercises as ex (ex.id)}
							<SessionExercise
								exercise={ex}
								isOpen={$openExerciseId === ex.id}
								disabled={$loading}
								lastTime={getLastTimeForExercise(ex.name)}
								on:toggle={() => toggleExercise(ex.id)}
								on:delete={() => removeExercise(ex.id)}
								on:markDone={() => markExerciseDone(ex.id)}
								on:addSet={(e) =>
									addSet(
										ex.id,
										e.detail.reps,
										e.detail.weight,
										e.detail.weightLeft,
										e.detail.weightRight,
										e.detail.durationSeconds
									)}
								on:updateNotes={(e) => updateExerciseNotes(ex.id, e.detail.notes)}
								on:addSetting={(e) => addExerciseSetting(ex.id, e.detail.key, e.detail.value)}
								on:removeSetting={(e) => removeExerciseSetting(ex.id, e.detail.id)}
								on:updateSetting={(e) =>
									updateExerciseSetting(ex.id, e.detail.id, e.detail.key, e.detail.value)}
								on:togglePerSideWeight={(e) => toggleExercisePerSideWeight(ex.id, e.detail.enabled)}
								on:toggleSplitWeight={(e) => toggleExerciseSplitWeight(ex.id, e.detail.enabled)}
								on:updateTracking={(e) => updateExerciseTracking(ex.id, e.detail)}
							/>
						{/each}
					</div>
				{/if}

				<ExerciseComposer
					bind:query={$exerciseQuery}
					disabled={$loading || !$currentSession}
					suggestions={$suggestions}
					templatePicks={$plannedTemplateExercises}
					quickPicks={$quickPicks}
					on:add={(e) => addExercise(e.detail.name)}
					on:addTemplateExercise={(e) => startPlannedTemplateExercise(e.detail.id)}
				/>
			{:else}
				<!-- no extra landing card -->
			{/if}
		</section>

		<aside class="md:col-span-5 lg:col-span-4 min-w-0">
			<RecentSessions
				sessions={$recentSessions}
				canAdd={Boolean($currentSession) && !$loading}
				disabled={$loading || !$currentSession}
				on:addExercise={(e) => addExercise(e.detail.name, e.detail)}
			/>
		</aside>
	</div>

	<EndSessionModal
		open={$endModalOpen}
		bind:notes={$endNotes}
		bind:mood={$endMood}
		disabled={$loading}
		on:cancel={() => ($endModalOpen = false)}
		on:submit={submitEndSession}
	/>
</div>
