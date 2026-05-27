<script lang="ts">
	import { goto } from '$app/navigation';
	import { cancelWorkout, createWorkoutTemplateFromWorkout } from '$lib/api';
	import { auth } from '$lib/auth';
	import { formatDateRelative, formatTime } from '$lib/utils/date';
	import { Btn, Card, Badge, Chip, PageHero, SetPillList } from '$lib/components/ui';
	import SessionDetailDesktop from '$lib/components/history/SessionDetailDesktop.svelte';
	import { isDesktop, isDesktopView } from '$lib/stores/viewport';
	import type { WorkoutWithExercises } from '$lib/types';

	interface Props {
		data: { workout: WorkoutWithExercises | null; error: string | null };
	}
	let { data }: Props = $props();

	let desktop = $derived(isDesktopView($isDesktop));

	const authState = auth.state;

	let workout = $state<WorkoutWithExercises | null>(null);
	let loadError = $state<string | null>(null);
	$effect(() => {
		workout = data.workout;
		loadError = data.error;
	});

	let deleting = $state(false);
	let savingTemplate = $state(false);
	let error = $state<string | null>(null);

	function durationMinutes(w: WorkoutWithExercises | null): number | null {
		if (!w) return null;
		const s = new Date(w.start_time).getTime();
		const e = new Date(w.end_time).getTime();
		if (!Number.isFinite(s) || !Number.isFinite(e) || e <= s) return null;
		return Math.round((e - s) / 60_000);
	}

	async function handleDelete() {
		if (!workout || typeof workout.id !== 'number') {
			error = 'Invalid workout ID';
			return;
		}
		if ($authState.offline) {
			error = 'Offline mode: delete workouts when you are back online.';
			return;
		}
		if (!confirm('Delete this workout? This cannot be undone.')) return;

		deleting = true;
		error = null;
		try {
			await cancelWorkout(workout.id);
			await goto('/workouts');
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to delete workout';
		} finally {
			deleting = false;
		}
	}

	async function handleSaveAsTemplate() {
		if (!workout || typeof workout.id !== 'number') {
			error = 'Invalid workout ID';
			return;
		}
		if ($authState.offline) {
			error = 'Offline mode: templates are available when you are back online.';
			return;
		}
		const defaultName =
			workout.exercises.length > 0
				? `${workout.exercises[0].exercise.exercise_type} template`
				: 'Workout template';
		const name = prompt('Template name:', defaultName)?.trim();
		if (!name) return;

		savingTemplate = true;
		error = null;
		try {
			const created = await createWorkoutTemplateFromWorkout(workout.id, { name });
			await goto(`/templates?template=${created.template.id}`);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to save template';
		} finally {
			savingTemplate = false;
		}
	}
</script>

{#snippet hero()}
	<PageHero kicker="► Workout">
		{#snippet title()}
			{#if workout}{formatDateRelative(workout.start_time)}{:else}Workout{/if}<em> — details.</em>
		{/snippet}
		{#snippet sub()}
			{#if workout}
				{formatTime(workout.start_time)} – {formatTime(workout.end_time)} ·
				{durationMinutes(workout) ?? '—'}m
			{/if}
		{/snippet}
	</PageHero>
{/snippet}

{#snippet summary()}
	{#if loadError}
		<Card><div class="err">{loadError}</div></Card>
	{:else if !workout}
		<Card><div class="muted">Loading…</div></Card>
	{:else}
		<Card>
			{#snippet title()}Session{/snippet}
			{#snippet actions()}
				<Btn variant="soft" size="sm" onclick={() => goto('/workouts')}>← Back</Btn>
			{/snippet}

			<div class="meta-row">
				{#if workout.feedback}<Badge tone="soft">{workout.feedback}</Badge>{/if}
				{#if workout.auto_closed_at}<Badge tone="warn">Auto-closed</Badge>{/if}
				<span class="meta">{workout.exercises.length} exercises</span>
			</div>
			{#if workout.notes}
				<p class="notes">{workout.notes}</p>
			{/if}

			<div class="actions">
				<Btn variant="soft" size="sm" onclick={handleSaveAsTemplate} disabled={savingTemplate}>
					{savingTemplate ? 'Saving…' : 'Save as template'}
				</Btn>
				<Btn variant="soft" size="sm" onclick={handleDelete} disabled={deleting}>
					{deleting ? 'Deleting…' : 'Delete'}
				</Btn>
			</div>
			{#if error}<div class="err">{error}</div>{/if}
		</Card>
	{/if}
{/snippet}

{#snippet exercises()}
	{#if workout}
		{#if workout.exercises.length > 0}
			<div class="ex-grid">
				{#each workout.exercises as ex (ex.exercise.id ?? ex.exercise.start_time)}
					<Card>
						{#snippet title()}{ex.exercise.exercise_type}{/snippet}
						{#snippet lede()}
							{formatTime(ex.exercise.start_time)} – {formatTime(ex.exercise.end_time)}
						{/snippet}

						{#if ex.exercise.settings && ex.exercise.settings.length > 0}
							<div class="settings">
								{#each ex.exercise.settings as s (s.id ?? s.key)}
									<Chip size="xs">{s.key}: {s.value}</Chip>
								{/each}
							</div>
						{/if}

						{#if ex.sets.length > 0}
							<div class="pills">
								<SetPillList
									sets={ex.sets.map((s) => ({
										reps: s.reps,
										weight: s.weight,
										weightLeft: s.weight_left,
										weightRight: s.weight_right,
										durationSeconds: s.duration_seconds
									}))}
									perSideWeight={ex.exercise.per_side_weight ?? false}
									splitWeight={ex.exercise.split_weight ?? false}
									size="sm"
								/>
							</div>
						{:else}
							<div class="muted">No sets logged.</div>
						{/if}

						{#if ex.exercise.notes}
							<p class="notes">Notes: {ex.exercise.notes}</p>
						{/if}
					</Card>
				{/each}
			</div>
		{:else}
			<Card><div class="muted">No exercises recorded.</div></Card>
		{/if}
	{/if}
{/snippet}

{#if desktop}
	<SessionDetailDesktop ready={Boolean(workout) && !loadError} {hero} {summary} {exercises} />
{:else}
	<div class="page">
		{@render hero()}
		{@render summary()}
		{@render exercises()}
	</div>
{/if}

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.ex-grid {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	/* Desktop main column flows exercise cards 2-up. */
	@media (min-width: 1024px) {
		.ex-grid {
			display: grid;
			grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
			align-items: start;
		}
	}
	.meta-row {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
		margin-bottom: 10px;
	}
	.meta {
		font:
			500 12px/1 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}
	.notes {
		margin: 6px 0 10px;
		font: italic 400 14px/1.4 'Instrument Serif';
		color: var(--ink-2);
	}
	.actions {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
	}
	.err {
		margin-top: 10px;
		font:
			600 13px/1.4 'Onest',
			system-ui,
			sans-serif;
		color: var(--clay-text);
	}
	.muted {
		font:
			500 13px/1.4 'Onest',
			system-ui,
			sans-serif;
		color: var(--ink-soft);
	}
	.settings {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
		margin-bottom: 8px;
	}
	.pills {
		margin-top: 6px;
	}
</style>
