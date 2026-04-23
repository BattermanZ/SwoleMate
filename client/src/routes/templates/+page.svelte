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
		getWorkoutTemplates,
		startWorkoutFromTemplate,
		updateWorkoutTemplate
	} from '$lib/api';
	import type {
		WorkoutTemplate,
		WorkoutTemplateDetail,
		WorkoutTemplateExerciseInput
	} from '$lib/types';

	export let data: { templates: WorkoutTemplate[] };

	type DraftSetting = {
		localId: string;
		key: string;
		value: string;
	};

	type DraftExercise = {
		localId: string;
		exercise_type: string;
		notes: string;
		per_side_weight: boolean;
		split_weight: boolean;
		settings: DraftSetting[];
	};

	type DraftTemplate = {
		id?: number;
		name: string;
		exercises: DraftExercise[];
	};

	let templates = data.templates;
	let selectedId: number | 'new' | null = null;
	let detailError: string | null = null;
	let pageError: string | null = null;
	let pageNotice: string | null = null;
	let loadingDetail = false;
	let saving = false;
	let starting = false;
	let deleting = false;
	let draft: DraftTemplate = createBlankDraft();
	let loadedTemplateId: number | null = null;
	const authState = auth.state;

	$: canEditTemplates = !$authState.offline;

	$: selectedSummary =
		typeof selectedId === 'number'
			? templates.find((template) => template.id === selectedId)
			: null;

	onMount(() => {
		const queryTemplateId = Number($page.url.searchParams.get('template'));
		if (queryTemplateId && templates.some((template) => template.id === queryTemplateId)) {
			void selectTemplate(queryTemplateId);
			return;
		}

		if (templates[0]) {
			void selectTemplate(templates[0].id);
			return;
		}

		openNewTemplate();
	});

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
			exercises: detail.exercises.map((exercise) => ({
				localId: `exercise-${exercise.id}`,
				exercise_type: exercise.exercise_type,
				notes: exercise.notes ?? '',
				per_side_weight: exercise.per_side_weight ?? false,
				split_weight: exercise.split_weight ?? false,
				settings: (exercise.settings ?? []).map((setting) => ({
					localId: `setting-${setting.id}`,
					key: setting.key,
					value: setting.value
				}))
			}))
		};
	}

	function toPayload(currentDraft: DraftTemplate) {
		return {
			name: currentDraft.name.trim(),
			exercises: currentDraft.exercises.map(
				(exercise): WorkoutTemplateExerciseInput => ({
					exercise_type: exercise.exercise_type.trim(),
					notes: exercise.notes.trim() || null,
					per_side_weight: exercise.per_side_weight,
					split_weight: exercise.split_weight,
					settings: exercise.settings
						.map((setting) => ({
							key: setting.key.trim(),
							value: setting.value.trim()
						}))
						.filter((setting) => setting.key && setting.value)
				})
			)
		};
	}

	async function refreshTemplates(selectId?: number | 'new') {
		templates = await getWorkoutTemplates();
		const nextSelection =
			selectId !== undefined
				? selectId
				: typeof selectedId === 'number' && templates.some((template) => template.id === selectedId)
					? selectedId
					: (templates[0]?.id ?? 'new');

		if (nextSelection === 'new') {
			openNewTemplate();
			return;
		}

		await selectTemplate(nextSelection);
	}

	async function loadTemplate(templateId: number) {
		loadingDetail = true;
		detailError = null;
		pageNotice = null;
		try {
			const detail = await getWorkoutTemplate(templateId);
			draft = toDraft(detail);
			selectedId = templateId;
		} catch (error) {
			detailError = error instanceof Error ? error.message : 'Failed to load template';
		} finally {
			loadedTemplateId = templateId;
			loadingDetail = false;
		}
	}

	async function selectTemplate(templateId: number) {
		if (templateId === loadedTemplateId && selectedId === templateId) return;
		selectedId = templateId;
		await loadTemplate(templateId);
	}

	function openNewTemplate() {
		selectedId = 'new';
		loadedTemplateId = null;
		detailError = null;
		pageError = null;
		pageNotice = null;
		draft = createBlankDraft();
	}

	function addExercise() {
		draft = {
			...draft,
			exercises: [
				...draft.exercises,
				{
					localId: makeLocalId(),
					exercise_type: '',
					notes: '',
					per_side_weight: false,
					split_weight: false,
					settings: []
				}
			]
		};
	}

	function updateExercise(localId: string, patch: Partial<DraftExercise>) {
		draft = {
			...draft,
			exercises: draft.exercises.map((exercise) =>
				exercise.localId === localId ? { ...exercise, ...patch } : exercise
			)
		};
	}

	function removeExercise(localId: string) {
		draft = {
			...draft,
			exercises: draft.exercises.filter((exercise) => exercise.localId !== localId)
		};
	}

	function addSetting(localId: string) {
		draft = {
			...draft,
			exercises: draft.exercises.map((exercise) =>
				exercise.localId === localId
					? {
							...exercise,
							settings: [...exercise.settings, { localId: makeLocalId(), key: '', value: '' }]
						}
					: exercise
			)
		};
	}

	function updateSetting(exerciseId: string, settingId: string, patch: Partial<DraftSetting>) {
		draft = {
			...draft,
			exercises: draft.exercises.map((exercise) =>
				exercise.localId === exerciseId
					? {
							...exercise,
							settings: exercise.settings.map((setting) =>
								setting.localId === settingId ? { ...setting, ...patch } : setting
							)
						}
					: exercise
			)
		};
	}

	function removeSetting(exerciseId: string, settingId: string) {
		draft = {
			...draft,
			exercises: draft.exercises.map((exercise) =>
				exercise.localId === exerciseId
					? {
							...exercise,
							settings: exercise.settings.filter((setting) => setting.localId !== settingId)
						}
					: exercise
			)
		};
	}

	async function handleSave() {
		if (!canEditTemplates) {
			pageError = 'Offline mode: templates are available when you are back online.';
			return;
		}

		saving = true;
		pageError = null;
		pageNotice = null;

		try {
			const payload = toPayload(draft);
			const wasExisting = typeof selectedId === 'number';
			const saved = wasExisting
				? await updateWorkoutTemplate(selectedId as number, payload)
				: await createWorkoutTemplate(payload);
			await refreshTemplates(saved.template.id);
			draft = toDraft(saved);
			loadedTemplateId = saved.template.id;
			pageNotice = wasExisting ? 'Template updated.' : 'Template created.';
			selectedId = saved.template.id;
		} catch (error) {
			pageError = error instanceof Error ? error.message : 'Failed to save template';
		} finally {
			saving = false;
		}
	}

	async function handleDuplicate() {
		if (typeof selectedId !== 'number') return;
		const nextName = prompt(
			'Duplicate template as:',
			`${draft.name || selectedSummary?.name || 'Template'} Copy`
		);
		if (!nextName) return;

		pageError = null;
		pageNotice = null;
		try {
			const duplicated = await duplicateWorkoutTemplate(selectedId, { name: nextName });
			await refreshTemplates(duplicated.template.id);
			draft = toDraft(duplicated);
			loadedTemplateId = duplicated.template.id;
			pageNotice = 'Template duplicated.';
		} catch (error) {
			pageError = error instanceof Error ? error.message : 'Failed to duplicate template';
		}
	}

	async function handleDelete() {
		if (typeof selectedId !== 'number') return;
		if (!confirm('Delete this template?')) return;

		deleting = true;
		pageError = null;
		pageNotice = null;
		try {
			await deleteWorkoutTemplate(selectedId);
			await refreshTemplates();
			if (!templates.length) {
				selectedId = 'new';
				loadedTemplateId = null;
				draft = createBlankDraft();
			} else if (typeof selectedId === 'number') {
				await loadTemplate(selectedId);
			}
			pageNotice = 'Template deleted.';
		} catch (error) {
			pageError = error instanceof Error ? error.message : 'Failed to delete template';
		} finally {
			deleting = false;
		}
	}

	async function handleStartTemplate() {
		if (typeof selectedId !== 'number') return;
		starting = true;
		pageError = null;
		pageNotice = null;

		try {
			const startIso = new Date().toISOString();
			const timezoneOffsetMinutes = new Date(startIso).getTimezoneOffset();
			await startWorkoutFromTemplate(selectedId, {
				date: startIso,
				start_time: startIso,
				timezone_offset_minutes: timezoneOffsetMinutes
			});
			await goto('/');
		} catch (error) {
			pageError = error instanceof Error ? error.message : 'Failed to start workout from template';
		} finally {
			starting = false;
		}
	}
</script>

<div class="space-y-6">
	<header
		class="relative overflow-hidden rounded-2xl border border-surface-200/50 dark:border-surface-700/50 bg-gradient-to-br from-primary-500/10 via-transparent to-tertiary-500/10 p-5 sm:p-6"
	>
		<div class="relative flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
			<div class="space-y-1">
				<h1 class="text-3xl sm:text-4xl font-black tracking-tight">Templates</h1>
				<p class="text-sm sm:text-base opacity-80 max-w-prose">
					Build repeatable workout structures and start sessions from them without carrying over
					reps or weight.
				</p>
			</div>
			<button type="button" class="btn variant-filled-primary" on:click={openNewTemplate}>
				New template
			</button>
		</div>
	</header>

	{#if pageError}
		<div class="alert variant-filled-error">{pageError}</div>
	{/if}
	{#if pageNotice}
		<div class="alert variant-filled-primary">{pageNotice}</div>
	{/if}

	<div class="grid gap-6 lg:grid-cols-[20rem_minmax(0,1fr)]">
		<aside class="card variant-glass-surface p-4 space-y-3">
			<div class="flex items-center justify-between gap-3">
				<h2 class="text-lg font-semibold">Your templates</h2>
				<span class="text-sm opacity-70">{templates.length}</span>
			</div>

			<div class="space-y-2">
				<button
					type="button"
					class="w-full text-left rounded-xl border p-3 transition {selectedId === 'new'
						? 'border-primary-500/60 bg-primary-500/10'
						: 'border-surface-200/50 bg-surface-50/60 dark:border-surface-700/50 dark:bg-surface-950/30'}"
					on:click={openNewTemplate}
				>
					<div class="font-semibold">New template</div>
					<div class="text-sm opacity-70">Start from scratch</div>
				</button>

				{#each templates as template (template.id)}
					<button
						type="button"
						class="w-full text-left rounded-xl border p-3 transition {selectedId === template.id
							? 'border-primary-500/60 bg-primary-500/10'
							: 'border-surface-200/50 bg-surface-50/60 dark:border-surface-700/50 dark:bg-surface-950/30'}"
						on:click={() => selectTemplate(template.id)}
					>
						<div class="font-semibold truncate">{template.name}</div>
						<div class="text-sm opacity-70">
							{template.exercise_count} exercise{template.exercise_count === 1 ? '' : 's'}
						</div>
					</button>
				{/each}
			</div>
		</aside>

		<section class="card variant-glass-surface p-4 space-y-4 min-w-0">
			<div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
				<div>
					<h2 class="text-lg font-semibold">
						{typeof selectedId === 'number' ? 'Edit template' : 'Create template'}
					</h2>
					<p class="text-sm opacity-70">
						Exercise order, notes, settings, and weight mode flags are saved. Sets are not.
					</p>
				</div>

				{#if typeof selectedId === 'number'}
					<div class="flex flex-wrap gap-2">
						<button
							type="button"
							class="btn btn-sm variant-soft"
							on:click={handleDuplicate}
							disabled={!canEditTemplates || saving || deleting || starting}
						>
							Duplicate
						</button>
						<button
							type="button"
							class="btn btn-sm variant-soft"
							on:click={handleStartTemplate}
							disabled={!canEditTemplates || saving || deleting || starting}
						>
							Use template
						</button>
						<button
							type="button"
							class="btn btn-sm variant-soft-error"
							on:click={handleDelete}
							disabled={!canEditTemplates || saving || deleting || starting}
						>
							Delete
						</button>
					</div>
				{/if}
			</div>

			{#if $authState.offline}
				<div class="card variant-ghost p-3 text-sm opacity-80">
					Offline mode: template changes and template starts are available when you are back online.
				</div>
			{/if}

			{#if detailError}
				<div class="alert variant-filled-error">{detailError}</div>
			{:else if loadingDetail}
				<div class="text-sm opacity-70">Loading template…</div>
			{:else}
				<label class="block">
					<span class="text-sm font-semibold opacity-80">Template name</span>
					<input class="input mt-1" bind:value={draft.name} placeholder="Push Day A" />
				</label>

				<div class="space-y-3">
					<div class="flex items-center justify-between gap-3">
						<h3 class="text-base font-semibold">Exercises</h3>
						<button type="button" class="btn btn-sm variant-soft" on:click={addExercise}>
							Add exercise
						</button>
					</div>

					{#if draft.exercises.length === 0}
						<div class="card variant-ghost p-4 text-sm opacity-80">
							Add exercises manually or save a past workout as a template from History.
						</div>
					{:else}
						<div class="space-y-4">
							{#each draft.exercises as exercise, index (exercise.localId)}
								<article
									class="rounded-2xl border border-surface-200/50 bg-surface-50/60 p-4 dark:border-surface-700/50 dark:bg-surface-950/30 space-y-3"
								>
									<div class="flex items-start justify-between gap-3">
										<div class="font-semibold">Exercise {index + 1}</div>
										<button
											type="button"
											class="btn btn-sm variant-soft-error"
											on:click={() => removeExercise(exercise.localId)}
										>
											Remove
										</button>
									</div>

									<label class="block">
										<span class="text-sm font-semibold opacity-80">Name</span>
										<input
											class="input mt-1"
											value={exercise.exercise_type}
											on:input={(event) =>
												updateExercise(exercise.localId, {
													exercise_type: (event.currentTarget as HTMLInputElement).value
												})}
											placeholder="Bench Press"
										/>
									</label>

									<label class="block">
										<span class="text-sm font-semibold opacity-80">Notes</span>
										<textarea
											class="textarea mt-1"
											rows="2"
											value={exercise.notes}
											on:input={(event) =>
												updateExercise(exercise.localId, {
													notes: (event.currentTarget as HTMLTextAreaElement).value
												})}
											placeholder="Setup cues, target tempo, machine choice…"
										></textarea>
									</label>

									<div class="flex flex-wrap gap-4 text-sm">
										<label class="inline-flex items-center gap-2">
											<input
												type="checkbox"
												checked={exercise.per_side_weight}
												on:change={(event) =>
													updateExercise(exercise.localId, {
														per_side_weight: (event.currentTarget as HTMLInputElement).checked,
														split_weight: (event.currentTarget as HTMLInputElement).checked
															? exercise.split_weight
															: false
													})}
											/>
											<span>Per-side weight</span>
										</label>
										<label class="inline-flex items-center gap-2">
											<input
												type="checkbox"
												checked={exercise.split_weight}
												on:change={(event) =>
													updateExercise(exercise.localId, {
														split_weight: (event.currentTarget as HTMLInputElement).checked,
														per_side_weight: (event.currentTarget as HTMLInputElement).checked
															? true
															: exercise.per_side_weight
													})}
											/>
											<span>Split left/right weight</span>
										</label>
									</div>

									<div class="space-y-2">
										<div class="flex items-center justify-between gap-3">
											<div class="text-sm font-semibold opacity-80">Settings</div>
											<button
												type="button"
												class="btn btn-sm variant-soft"
												on:click={() => addSetting(exercise.localId)}
											>
												Add setting
											</button>
										</div>

										{#if exercise.settings.length === 0}
											<div class="text-sm opacity-60">No settings saved for this exercise.</div>
										{:else}
											<div class="space-y-2">
												{#each exercise.settings as setting (setting.localId)}
													<div class="grid gap-2 sm:grid-cols-[1fr_1fr_auto]">
														<input
															class="input"
															value={setting.key}
															on:input={(event) =>
																updateSetting(exercise.localId, setting.localId, {
																	key: (event.currentTarget as HTMLInputElement).value
																})}
															placeholder="Seat"
														/>
														<input
															class="input"
															value={setting.value}
															on:input={(event) =>
																updateSetting(exercise.localId, setting.localId, {
																	value: (event.currentTarget as HTMLInputElement).value
																})}
															placeholder="5"
														/>
														<button
															type="button"
															class="btn btn-sm variant-soft-error"
															on:click={() => removeSetting(exercise.localId, setting.localId)}
														>
															Remove
														</button>
													</div>
												{/each}
											</div>
										{/if}
									</div>
								</article>
							{/each}
						</div>
					{/if}
				</div>

				<div class="flex justify-end">
					<button
						type="button"
						class="btn variant-filled-primary"
						on:click={handleSave}
						disabled={!canEditTemplates || saving || deleting || starting}
					>
						{typeof selectedId === 'number' ? 'Save changes' : 'Create template'}
					</button>
				</div>
			{/if}
		</section>
	</div>
</div>
