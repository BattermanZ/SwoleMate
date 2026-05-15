<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { auth } from '$lib/auth';
	import {
		createWorkoutTemplate,
		deleteWorkoutTemplate,
		duplicateWorkoutTemplate,
		getWorkoutTemplate,
		updateWorkoutTemplate,
		startWorkoutFromTemplate
	} from '$lib/api';
	import type {
		WorkoutTemplate,
		WorkoutTemplateDetail,
		WorkoutTemplateExerciseInput
	} from '$lib/types';
	import {
		decodeTrackingFields,
		isTrackingFieldsSetting,
		trackingFieldsSetting,
		TRACKING_FIELDS_SETTING_KEY
	} from '$lib/today/tracking';
	import { Btn, Card, Chip, Chk, PageHero } from '$lib/components/ui';

	interface Props {
		data: { templates: WorkoutTemplate[] };
	}
	let { data }: Props = $props();

	type DraftSetting = { localId: string; key: string; value: string };
	type DraftExercise = {
		localId: string;
		exercise_type: string;
		notes: string;
		per_side_weight: boolean;
		split_weight: boolean;
		tracks_reps: boolean;
		tracks_time: boolean;
		tracks_weight: boolean;
		settings: DraftSetting[];
	};
	type DraftTemplate = { id?: number; name: string; exercises: DraftExercise[] };

	const authState = auth.state;

	let templates = $state<WorkoutTemplate[]>([]);
	$effect(() => {
		templates = data.templates;
	});

	let selectedId = $state<number | 'new' | null>(null);
	let detailError = $state<string | null>(null);
	let pageError = $state<string | null>(null);
	let pageNotice = $state<string | null>(null);
	let loadingDetail = $state(false);
	let saving = $state(false);
	let starting = $state(false);
	let deleting = $state(false);
	let draft = $state<DraftTemplate>(createBlankDraft());

	let canEdit = $derived(!$authState.offline);

	function makeLocalId() {
		return Math.random().toString(36).slice(2, 10);
	}

	function createBlankDraft(): DraftTemplate {
		return { name: '', exercises: [] };
	}

	function toDraft(detail: WorkoutTemplateDetail): DraftTemplate {
		return {
			id: detail.template.id,
			name: detail.template.name,
			exercises: detail.exercises.map((ex) => {
				const tracking = decodeTrackingFields(
					(ex.settings ?? []).find((s) => s.key === TRACKING_FIELDS_SETTING_KEY)?.value
				);
				return {
					localId: `ex-${ex.id}`,
					exercise_type: ex.exercise_type,
					notes: ex.notes ?? '',
					per_side_weight: ex.per_side_weight ?? false,
					split_weight: ex.split_weight ?? false,
					tracks_reps: tracking.reps,
					tracks_time: tracking.time,
					tracks_weight: tracking.weight,
					settings: (ex.settings ?? [])
						.filter((s) => !isTrackingFieldsSetting(s))
						.map((s) => ({
							localId: `set-${s.id}`,
							key: s.key,
							value: s.value
						}))
				};
			})
		};
	}

	function draftToInput(d: DraftTemplate): { name: string; exercises: WorkoutTemplateExerciseInput[] } {
		return {
			name: d.name.trim(),
			exercises: d.exercises.map((ex) => ({
				exercise_type: ex.exercise_type.trim(),
				notes: ex.notes.trim() || null,
				per_side_weight: ex.per_side_weight,
				split_weight: ex.split_weight,
				settings: [
					trackingFieldsSetting({
						reps: ex.tracks_reps,
						time: ex.tracks_time,
						weight: ex.tracks_weight
					}),
					...ex.settings
						.filter((s) => s.key.trim() && s.value.trim())
						.map((s) => ({ key: s.key.trim(), value: s.value.trim() }))
				]
			}))
		};
	}

	async function selectTemplate(id: number) {
		selectedId = id;
		detailError = null;
		loadingDetail = true;
		try {
			const detail = await getWorkoutTemplate(id);
			draft = toDraft(detail);
		} catch (e) {
			detailError = e instanceof Error ? e.message : 'Failed to load template';
			draft = createBlankDraft();
		} finally {
			loadingDetail = false;
		}
	}

	function openNewTemplate() {
		selectedId = 'new';
		draft = createBlankDraft();
		detailError = null;
	}

	function addExercise() {
		draft.exercises = [
			...draft.exercises,
			{
				localId: makeLocalId(),
				exercise_type: '',
				notes: '',
				per_side_weight: false,
				split_weight: false,
				tracks_reps: true,
				tracks_time: false,
				tracks_weight: true,
				settings: []
			}
		];
	}

	function removeExercise(localId: string) {
		draft.exercises = draft.exercises.filter((e) => e.localId !== localId);
	}

	function updateExercise(localId: string, patch: Partial<DraftExercise>) {
		draft.exercises = draft.exercises.map((e) =>
			e.localId === localId ? { ...e, ...patch } : e
		);
	}

	function addSetting(localId: string) {
		updateExercise(localId, {
			settings: [
				...(draft.exercises.find((e) => e.localId === localId)?.settings ?? []),
				{ localId: makeLocalId(), key: '', value: '' }
			]
		});
	}

	function removeSetting(exerciseLocalId: string, settingLocalId: string) {
		const ex = draft.exercises.find((e) => e.localId === exerciseLocalId);
		if (!ex) return;
		updateExercise(exerciseLocalId, {
			settings: ex.settings.filter((s) => s.localId !== settingLocalId)
		});
	}

	function updateSetting(exerciseLocalId: string, settingLocalId: string, patch: Partial<DraftSetting>) {
		const ex = draft.exercises.find((e) => e.localId === exerciseLocalId);
		if (!ex) return;
		updateExercise(exerciseLocalId, {
			settings: ex.settings.map((s) =>
				s.localId === settingLocalId ? { ...s, ...patch } : s
			)
		});
	}

	async function handleSave() {
		pageError = pageNotice = null;
		if (!draft.name.trim()) {
			pageError = 'Template name is required.';
			return;
		}
		saving = true;
		try {
			const input = draftToInput(draft);
			if (typeof selectedId === 'number') {
				const updated = await updateWorkoutTemplate(selectedId, input);
				draft = toDraft(updated);
				pageNotice = 'Template updated.';
			} else {
				const created = await createWorkoutTemplate(input);
				draft = toDraft(created);
				selectedId = created.template.id;
				pageNotice = 'Template created.';
			}
			const all = await import('$lib/api').then((m) => m.getWorkoutTemplates());
			templates = all;
		} catch (e) {
			pageError = e instanceof Error ? e.message : 'Failed to save template';
		} finally {
			saving = false;
		}
	}

	async function handleDuplicate() {
		if (typeof selectedId !== 'number') return;
		saving = true;
		pageError = pageNotice = null;
		try {
			const dup = await duplicateWorkoutTemplate(selectedId, { name: `${draft.name} copy` });
			const all = await import('$lib/api').then((m) => m.getWorkoutTemplates());
			templates = all;
			await selectTemplate(dup.template.id);
			pageNotice = 'Template duplicated.';
		} catch (e) {
			pageError = e instanceof Error ? e.message : 'Failed to duplicate template';
		} finally {
			saving = false;
		}
	}

	async function handleDelete() {
		if (typeof selectedId !== 'number') return;
		if (!confirm('Delete this template?')) return;
		deleting = true;
		pageError = pageNotice = null;
		try {
			await deleteWorkoutTemplate(selectedId);
			templates = templates.filter((t) => t.id !== selectedId);
			openNewTemplate();
			pageNotice = 'Template deleted.';
		} catch (e) {
			pageError = e instanceof Error ? e.message : 'Failed to delete template';
		} finally {
			deleting = false;
		}
	}

	async function handleStartTemplate() {
		if (typeof selectedId !== 'number') return;
		starting = true;
		pageError = pageNotice = null;
		try {
			const startIso = new Date().toISOString();
			const tzOffsetMinutes = new Date(startIso).getTimezoneOffset();
			await startWorkoutFromTemplate(selectedId, {
				date: startIso,
				start_time: startIso,
				timezone_offset_minutes: tzOffsetMinutes
			});
			await goto('/');
		} catch (e) {
			pageError = e instanceof Error ? e.message : 'Failed to start from template';
		} finally {
			starting = false;
		}
	}

	onMount(() => {
		const queryId = Number($page.url.searchParams.get('template'));
		if (queryId && templates.some((t) => t.id === queryId)) {
			void selectTemplate(queryId);
			return;
		}
		if (templates[0]) {
			void selectTemplate(templates[0].id);
			return;
		}
		openNewTemplate();
	});
</script>

<div class="page">
	<PageHero kicker="► Plans · templates">
		{#snippet title()}Repeatable <em>workouts.</em>{/snippet}
		{#snippet sub()}Templates preload your exercise plan. Sets and weights are not saved.{/snippet}
		{#snippet actions()}
			<Btn variant="primary" onclick={openNewTemplate}>+ New template</Btn>
		{/snippet}
	</PageHero>

	{#if pageError}
		<Card><div class="err">{pageError}</div></Card>
	{/if}
	{#if pageNotice}
		<Card><div class="ok">{pageNotice}</div></Card>
	{/if}

	<Card>
		{#snippet title()}Your templates <em>({templates.length})</em>{/snippet}

		<div class="list">
			<button
				class="t-card"
				class:selected={selectedId === 'new'}
				onclick={openNewTemplate}
				type="button"
			>
				<div class="t-name">+ New template</div>
				<div class="t-meta">Start from scratch</div>
			</button>
			{#each templates as t (t.id)}
				<button
					class="t-card"
					class:selected={selectedId === t.id}
					onclick={() => selectTemplate(t.id)}
					type="button"
				>
					<div class="t-name">{t.name}</div>
					<div class="t-meta">
						{t.exercise_count} exercise{t.exercise_count === 1 ? '' : 's'}
					</div>
				</button>
			{/each}
		</div>
	</Card>

	<Card>
		{#snippet title()}
			{typeof selectedId === 'number' ? 'Edit template' : 'Create template'}
		{/snippet}
		{#snippet actions()}
			{#if typeof selectedId === 'number'}
				<Btn variant="soft" size="sm" onclick={handleDuplicate} disabled={!canEdit || saving}>
					Duplicate
				</Btn>
				<Btn variant="soft" size="sm" onclick={handleStartTemplate} disabled={!canEdit || starting}>
					{starting ? 'Starting…' : 'Use →'}
				</Btn>
				<Btn variant="soft" size="sm" onclick={handleDelete} disabled={!canEdit || deleting}>
					{deleting ? 'Deleting…' : 'Delete'}
				</Btn>
			{/if}
		{/snippet}

		{#if !canEdit}
			<div class="muted">Offline mode: template changes are unavailable.</div>
		{/if}

		{#if detailError}
			<div class="err">{detailError}</div>
		{:else if loadingDetail}
			<div class="muted">Loading template…</div>
		{:else}
			<label class="block">
				<span class="lbl">Template name</span>
				<input bind:value={draft.name} placeholder="Push Day A" disabled={!canEdit} />
			</label>

			<div class="ex-list">
				<div class="ex-list-head">
					<h3>Exercises</h3>
					<Btn variant="soft" size="sm" onclick={addExercise} disabled={!canEdit}>+ Add</Btn>
				</div>

				{#if draft.exercises.length === 0}
					<div class="empty">
						Add exercises manually, or save a past workout as a template from its detail page.
					</div>
				{:else}
					<div class="ex-cards">
						{#each draft.exercises as ex, i (ex.localId)}
							<article class="ex-card">
								<header class="ex-head">
									<div class="ex-idx">Exercise {i + 1}</div>
									<Btn
										variant="soft"
										size="sm"
										onclick={() => removeExercise(ex.localId)}
										disabled={!canEdit}
									>
										Remove
									</Btn>
								</header>

								<label class="block">
									<span class="lbl">Name</span>
									<input
										value={ex.exercise_type}
										placeholder="Bench Press"
										disabled={!canEdit}
										oninput={(e) =>
											updateExercise(ex.localId, {
												exercise_type: (e.currentTarget as HTMLInputElement).value
											})}
									/>
								</label>

								<label class="block">
									<span class="lbl">Notes</span>
									<textarea
										value={ex.notes}
										rows="2"
										placeholder="Cues, tempo…"
										disabled={!canEdit}
										oninput={(e) =>
											updateExercise(ex.localId, {
												notes: (e.currentTarget as HTMLTextAreaElement).value
											})}
									></textarea>
								</label>

								<div class="toggles">
									<Chk
										label="Reps"
										checked={ex.tracks_reps}
										disabled={!canEdit}
										onchange={(v) => updateExercise(ex.localId, { tracks_reps: v })}
									/>
									<Chk
										label="Time"
										checked={ex.tracks_time}
										disabled={!canEdit}
										onchange={(v) => updateExercise(ex.localId, { tracks_time: v })}
									/>
									<Chk
										label="Weight"
										checked={ex.tracks_weight}
										disabled={!canEdit}
										onchange={(v) => updateExercise(ex.localId, { tracks_weight: v })}
									/>
									<Chk
										label="Per-side"
										checked={ex.per_side_weight}
										disabled={!canEdit}
										onchange={(v) => updateExercise(ex.localId, { per_side_weight: v })}
									/>
									{#if ex.per_side_weight}
										<Chk
											label="Split L/R"
											checked={ex.split_weight}
											disabled={!canEdit}
											onchange={(v) => updateExercise(ex.localId, { split_weight: v })}
										/>
									{/if}
								</div>

								<div class="settings">
									<div class="settings-head">
										<h4>Settings</h4>
										<Btn
											variant="soft"
											size="sm"
											onclick={() => addSetting(ex.localId)}
											disabled={!canEdit}
										>
											+ Setting
										</Btn>
									</div>
									{#if ex.settings.length === 0}
										<div class="muted">No settings yet (bench angle, pin position…).</div>
									{:else}
										{#each ex.settings as s (s.localId)}
											<div class="setting-row">
												<input
													placeholder="Setting"
													value={s.key}
													disabled={!canEdit}
													oninput={(e) =>
														updateSetting(ex.localId, s.localId, {
															key: (e.currentTarget as HTMLInputElement).value
														})}
												/>
												<input
													placeholder="Value"
													value={s.value}
													disabled={!canEdit}
													oninput={(e) =>
														updateSetting(ex.localId, s.localId, {
															value: (e.currentTarget as HTMLInputElement).value
														})}
												/>
												<button
													class="x-btn"
													type="button"
													disabled={!canEdit}
													onclick={() => removeSetting(ex.localId, s.localId)}
												>
													✕
												</button>
											</div>
										{/each}
									{/if}
								</div>
							</article>
						{/each}
					</div>
				{/if}
			</div>

			<div class="save-bar">
				<Btn variant="primary" onclick={handleSave} disabled={!canEdit || saving}>
					{saving ? 'Saving…' : 'Save template'}
				</Btn>
			</div>
		{/if}
	</Card>
</div>

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.t-card {
		text-align: left;
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 12px;
		padding: 12px 14px;
		cursor: pointer;
		color: inherit;
		display: block;
	}
	.t-card.selected {
		border-color: color-mix(in oklab, var(--clay) 50%, var(--line));
		background: color-mix(in oklab, var(--clay) 8%, var(--card-3));
	}
	.t-name {
		font: 800 14px/1 'Onest', system-ui, sans-serif;
	}
	.t-meta {
		margin-top: 4px;
		font: 500 12px/1 'Onest', system-ui, sans-serif;
		color: var(--ink-soft);
	}

	.block {
		display: block;
		margin-bottom: 12px;
	}
	.lbl {
		display: block;
		font: 700 10px/1 'Onest', system-ui, sans-serif;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--ink-soft);
		margin-bottom: 6px;
	}
	input,
	textarea {
		width: 100%;
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 10px;
		padding: 11px 12px;
		font: 500 14px/1.2 'Onest', system-ui, sans-serif;
		color: var(--ink);
		outline: 0;
		resize: vertical;
	}
	textarea {
		min-height: 56px;
		font-family: 'Onest', system-ui, sans-serif;
	}
	input:focus,
	textarea:focus {
		border-color: var(--clay);
		box-shadow: 0 0 0 3px rgba(255, 94, 31, 0.16);
	}

	.ex-list {
		margin-top: 14px;
	}
	.ex-list-head {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 10px;
	}
	.ex-list-head h3 {
		margin: 0;
		font: 800 14px/1 'Onest', system-ui, sans-serif;
		letter-spacing: -0.01em;
	}
	.ex-cards {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.ex-card {
		background: var(--card-3);
		border: 1px solid var(--line);
		border-radius: 14px;
		padding: 12px;
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	.ex-head {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}
	.ex-idx {
		font: 800 13px/1 'Onest', system-ui, sans-serif;
	}
	.toggles {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}
	.settings {
		border-top: 1px solid var(--line);
		padding-top: 10px;
	}
	.settings-head {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 8px;
	}
	.settings-head h4 {
		margin: 0;
		font: 700 10px/1 'Onest', system-ui, sans-serif;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--ink-soft);
	}
	.setting-row {
		display: grid;
		grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) auto;
		gap: 6px;
		align-items: center;
	}
	.setting-row + .setting-row {
		margin-top: 6px;
	}
	.x-btn {
		min-width: 38px;
		height: 38px;
		border: 0;
		border-radius: 10px;
		background: color-mix(in oklab, var(--clay) 10%, transparent);
		color: var(--clay-text);
		font: 800 13px/1 'Onest', system-ui, sans-serif;
		cursor: pointer;
	}

	.empty {
		font: 500 13px/1.4 'Onest', system-ui, sans-serif;
		color: var(--ink-soft);
		padding: 12px;
		border-radius: 10px;
		background: var(--card-3);
		border: 1px dashed var(--line);
	}
	.muted {
		font: 500 13px/1.4 'Onest', system-ui, sans-serif;
		color: var(--ink-soft);
	}
	.err {
		font: 600 13px/1.4 'Onest', system-ui, sans-serif;
		color: var(--clay-text);
	}
	.ok {
		font: 600 13px/1.4 'Onest', system-ui, sans-serif;
		color: var(--sage);
	}

	.save-bar {
		margin-top: 14px;
		display: flex;
		justify-content: flex-end;
	}
</style>
