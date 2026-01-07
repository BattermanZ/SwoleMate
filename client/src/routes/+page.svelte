<script lang="ts">
	import { onMount } from 'svelte';
	import ExerciseComposer from '$lib/components/today/ExerciseComposer.svelte';
	import EndSessionModal from '$lib/components/today/EndSessionModal.svelte';
	import RecentSessions from '$lib/components/today/RecentSessions.svelte';
	import SessionExercise from '$lib/components/today/SessionExercise.svelte';
	import { EXERCISE_LIBRARY, createDemoSession } from '$lib/mocks/today';
	import { formatDateRelative, formatTime } from '$lib/utils/date';
	import { createId } from '$lib/utils/id';
	import {
		cancelExercise,
		cancelWorkout,
		createExercise,
		createSet,
		createWorkout,
		endExercise,
		endWorkout,
		getExerciseTypes,
		getWorkout,
		getWorkouts,
		replaceSets
	} from '$lib/api';
	import { toUiSession, workoutIsActive } from '$lib/today/backend';
	import type { UiMood, UiSession } from '$lib/today/types';

	let nowMs = Date.now();

	let currentSession: UiSession | null = null;
	let recentSessions: UiSession[] = [];

	let openExerciseId: number | null = null;

	let exerciseQuery = '';
	let sessionNotes = '';
	let endMood: UiMood | null = null;
	let endModalOpen = false;
	let endNotes = '';
	let loading = false;
	let error: string | null = null;

	const MAX_SUGGESTIONS = 10;
	let exerciseLibrary: string[] = EXERCISE_LIBRARY;

	const syncTimers = new Map<number, number>();

	onMount(() => {
		void refreshFromBackend();
		void hydrateExerciseLibrary();
		const timer = window.setInterval(() => (nowMs = Date.now()), 10_000);
		return () => window.clearInterval(timer);
	});

	$: if (currentSession && currentSession.notes !== sessionNotes) {
		currentSession = { ...currentSession, notes: sessionNotes };
	}

	async function hydrateExerciseLibrary() {
		try {
			const types = await getExerciseTypes();
			const merged = new Set<string>([...EXERCISE_LIBRARY, ...types]);
			exerciseLibrary = Array.from(merged).sort((a, b) => a.localeCompare(b));
		} catch {
			// ignore: local library is good enough
		}
	}

	function getErrorMessage(e: unknown): string {
		if (e instanceof Error) return e.message;
		return 'Something went wrong';
	}

	function resetLocalSessionUi() {
		exerciseQuery = '';
		endMood = null;
		endNotes = '';
		endModalOpen = false;
	}

	async function refreshFromBackend() {
		loading = true;
		error = null;

		try {
			const workouts = await getWorkouts();
			const active = workouts.find((w) => workoutIsActive(w) && w.id != null);

			if (active?.id != null) {
				const data = await getWorkout(active.id);
				currentSession = toUiSession(data.workout, data.exercises);
				sessionNotes = currentSession.notes;
				openExerciseId = currentSession.exercises[0]?.id ?? null;
			} else {
				currentSession = null;
				sessionNotes = '';
				openExerciseId = null;
			}

			const completed = workouts.filter((w) => w.id != null && !workoutIsActive(w)).slice(0, 2);
			const recent = await Promise.all(completed.map((w) => getWorkout(w.id!)));
			recentSessions = recent.map((d) => toUiSession(d.workout, d.exercises));

			resetLocalSessionUi();
		} catch (e) {
			error = getErrorMessage(e);
		} finally {
			loading = false;
		}
	}

	async function startSession(mode: 'empty' | 'demo') {
		if (currentSession) return;
		loading = true;
		error = null;

		try {
			const demo = mode === 'demo' ? createDemoSession() : null;
			const startIso = demo?.startedAt ?? new Date().toISOString();

			const created = await createWorkout({
				date: startIso,
				start_time: startIso,
				notes: demo?.notes?.trim() || undefined
			});

			currentSession = {
				id: created.id,
				startedAt: startIso,
				notes: demo?.notes ?? '',
				exercises: []
			};
			sessionNotes = currentSession.notes;
			openExerciseId = null;
			resetLocalSessionUi();

			if (demo) {
				for (const ex of demo.exercises) {
					await addExercise(
						ex.name,
						{
							notes: ex.notes,
							perSideWeight: ex.perSideWeight,
							splitWeight: ex.splitWeight,
							settings: ex.settings.map((s) => ({ key: s.key, value: s.value }))
						},
						ex.sets.map((s) => ({ reps: s.reps, weight: s.weight }))
					);
				}
			}

			await refreshFromBackend();
		} catch (e) {
			error = getErrorMessage(e);
			await refreshFromBackend();
		} finally {
			loading = false;
		}
	}

	async function cancelSession() {
		if (!currentSession) return;
		loading = true;
		error = null;

		try {
			await cancelWorkout(currentSession.id);
			currentSession = null;
			sessionNotes = '';
			openExerciseId = null;
			resetLocalSessionUi();
			await refreshFromBackend();
		} catch (e) {
			error = getErrorMessage(e);
		} finally {
			loading = false;
		}
	}

	function openEndModal() {
		if (!currentSession) return;
		endNotes = sessionNotes;
		endMood = null;
		endModalOpen = true;
	}

	async function submitEndSession() {
		if (!currentSession || !endMood) return;
		loading = true;
		error = null;

		try {
			const endedAt = new Date().toISOString();

			await Promise.all(
				currentSession.exercises
					.filter((e) => e.status !== 'done')
					.map((e) =>
						endExercise(e.id, {
							end_time: endedAt,
							notes: e.notes || undefined,
							per_side_weight: e.perSideWeight,
							split_weight: e.splitWeight,
							settings: e.settings.map((s) => ({ key: s.key, value: s.value }))
						})
					)
			);

			await endWorkout(currentSession.id, {
				end_time: endedAt,
				notes: endNotes.trim() || undefined,
				feedback: endMood
			});

			currentSession = null;
			sessionNotes = '';
			openExerciseId = null;
			resetLocalSessionUi();
			await refreshFromBackend();
		} catch (e) {
			error = getErrorMessage(e);
		} finally {
			loading = false;
		}
	}

	function toggleExercise(exerciseId: number) {
		openExerciseId = openExerciseId === exerciseId ? null : exerciseId;
	}

	async function addExercise(
		name: string,
		options?: {
			notes?: string;
			perSideWeight?: boolean;
			splitWeight?: boolean;
			settings?: Array<{ key: string; value: string }>;
		},
		seedSets?: Array<{ reps: number; weight: number; weightLeft?: number; weightRight?: number }>
	) {
		if (!currentSession) return;
		const trimmed = name.trim();
		if (!trimmed) return;

		loading = true;
		error = null;

		try {
			const startIso = new Date().toISOString();
			const perSideWeight = options?.perSideWeight ?? false;
			const splitWeight = options?.splitWeight ?? false;
			const settings = options?.settings ?? [];

			const created = await createExercise(currentSession.id, {
				exercise_type: trimmed,
				start_time: startIso,
				notes: options?.notes?.trim() || undefined,
				per_side_weight: perSideWeight,
				split_weight: splitWeight,
				settings: settings.length
					? settings.map((s) => ({ key: s.key, value: s.value }))
					: undefined
			});

			const newExercise = {
				id: created.id,
				name: trimmed,
				notes: options?.notes?.trim() ?? '',
				startedAt: startIso,
				endedAt: startIso,
				status: 'active' as const,
				perSideWeight,
				splitWeight,
				settings: settings.map((s) => ({
					id: createId('setting'),
					key: s.key,
					value: s.value
				})),
				sets: []
			};

			currentSession = {
				...currentSession,
				exercises: [...currentSession.exercises, newExercise]
			};

			openExerciseId = newExercise.id;
			exerciseQuery = '';

			if (seedSets?.length) {
				for (const s of seedSets) {
					await addSet(newExercise.id, s.reps, s.weight, s.weightLeft, s.weightRight);
				}
			}
		} catch (e) {
			error = getErrorMessage(e);
		} finally {
			loading = false;
		}
	}

	async function removeExercise(exerciseId: number) {
		if (!currentSession) return;
		loading = true;
		error = null;

		try {
			await cancelExercise(exerciseId);
			currentSession = {
				...currentSession,
				exercises: currentSession.exercises.filter((e) => e.id !== exerciseId)
			};
			if (openExerciseId === exerciseId) openExerciseId = null;
		} catch (e) {
			error = getErrorMessage(e);
			await refreshFromBackend();
		} finally {
			loading = false;
		}
	}

	async function markExerciseDone(exerciseId: number) {
		if (!currentSession) return;
		loading = true;
		error = null;

		try {
			const endedAt = new Date().toISOString();
			const ex = currentSession.exercises.find((e) => e.id === exerciseId);
			if (!ex) return;

			await endExercise(exerciseId, {
				end_time: endedAt,
				notes: ex.notes || undefined,
				per_side_weight: ex.perSideWeight,
				split_weight: ex.splitWeight,
				settings: ex.settings.map((s) => ({ key: s.key, value: s.value }))
			});

			currentSession = {
				...currentSession,
				exercises: currentSession.exercises.map((e) =>
					e.id === exerciseId ? { ...e, status: 'done' as const, endedAt } : e
				)
			};
		} catch (e) {
			error = getErrorMessage(e);
		} finally {
			loading = false;
		}
	}

	async function addSet(
		exerciseId: number,
		reps: number,
		weight: number,
		weightLeft?: number,
		weightRight?: number
	) {
		if (!currentSession) return;
		loading = true;
		error = null;

		try {
			const created = await createSet(exerciseId, {
				reps,
				weight,
				weight_left: weightLeft,
				weight_right: weightRight,
				notes: undefined
			});

			currentSession = {
				...currentSession,
				exercises: currentSession.exercises.map((e) => {
					if (e.id !== exerciseId) return e;
					return {
						...e,
						sets: [
							...e.sets,
							{
								id: created.id,
								reps,
								weight,
								weightLeft,
								weightRight
							}
						]
					};
				})
			};
		} catch (e) {
			error = getErrorMessage(e);
		} finally {
			loading = false;
		}
	}

	function scheduleExerciseSync(exerciseId: number) {
		if (typeof window === 'undefined') return;
		const existing = syncTimers.get(exerciseId);
		if (existing) window.clearTimeout(existing);
		const timer = window.setTimeout(() => void syncExercise(exerciseId), 650);
		syncTimers.set(exerciseId, timer);
	}

	async function syncExercise(exerciseId: number) {
		if (!currentSession) return;
		const ex = currentSession.exercises.find((e) => e.id === exerciseId);
		if (!ex) return;

		try {
			await endExercise(exerciseId, {
				end_time: ex.endedAt,
				notes: ex.notes || undefined,
				per_side_weight: ex.perSideWeight,
				split_weight: ex.splitWeight,
				settings: ex.settings.map((s) => ({ key: s.key, value: s.value }))
			});
		} catch (e) {
			error = getErrorMessage(e);
		}
	}

	function updateExerciseNotes(exerciseId: number, notes: string) {
		if (!currentSession) return;
		currentSession = {
			...currentSession,
			exercises: currentSession.exercises.map((e) => (e.id === exerciseId ? { ...e, notes } : e))
		};
		scheduleExerciseSync(exerciseId);
	}

	function addExerciseSetting(exerciseId: number, key: string, value: string) {
		if (!currentSession) return;
		currentSession = {
			...currentSession,
			exercises: currentSession.exercises.map((e) => {
				if (e.id !== exerciseId) return e;
				return {
					...e,
					settings: [...e.settings, { id: createId('setting'), key, value }]
				};
			})
		};
		scheduleExerciseSync(exerciseId);
	}

	function removeExerciseSetting(exerciseId: number, settingId: string) {
		if (!currentSession) return;
		currentSession = {
			...currentSession,
			exercises: currentSession.exercises.map((e) => {
				if (e.id !== exerciseId) return e;
				return {
					...e,
					settings: e.settings.filter((s) => s.id !== settingId)
				};
			})
		};
		scheduleExerciseSync(exerciseId);
	}

	function updateExerciseSetting(
		exerciseId: number,
		settingId: string,
		key: string,
		value: string
	) {
		if (!currentSession) return;
		currentSession = {
			...currentSession,
			exercises: currentSession.exercises.map((e) => {
				if (e.id !== exerciseId) return e;
				return {
					...e,
					settings: e.settings.map((s) => (s.id === settingId ? { ...s, key, value } : s))
				};
			})
		};
		scheduleExerciseSync(exerciseId);
	}

	async function toggleExercisePerSideWeight(exerciseId: number, enabled: boolean) {
		if (!currentSession) return;
		const ex = currentSession.exercises.find((e) => e.id === exerciseId);
		if (!ex || ex.status === 'done') return;
		if (enabled === ex.perSideWeight) return;

		let nextSets = ex.sets;
		let nextSplit = ex.splitWeight;

		if (!enabled) {
			nextSplit = false;
			nextSets = ex.sets.map((s) => {
				if (!ex.perSideWeight) return s;
				if (!ex.splitWeight) {
					return { ...s, weight: s.weight * 2, weightLeft: undefined, weightRight: undefined };
				}
				const left = s.weightLeft ?? s.weight;
				const right = s.weightRight ?? s.weight;
				return { ...s, weight: left + right, weightLeft: undefined, weightRight: undefined };
			});
		} else {
			nextSets = ex.sets.map((s) => ({
				...s,
				weight: s.weight / 2,
				weightLeft: undefined,
				weightRight: undefined
			}));
		}

		currentSession = {
			...currentSession,
			exercises: currentSession.exercises.map((e) =>
				e.id === exerciseId
					? { ...e, perSideWeight: enabled, splitWeight: nextSplit, sets: nextSets }
					: e
			)
		};

		try {
			await endExercise(exerciseId, {
				end_time: ex.endedAt,
				notes: ex.notes || undefined,
				per_side_weight: enabled,
				split_weight: nextSplit,
				settings: ex.settings.map((s) => ({ key: s.key, value: s.value }))
			});

			if (nextSets.length) {
				const replaced = await replaceSets(
					exerciseId,
					nextSets.map((s) => ({
						reps: s.reps,
						weight: s.weight,
						weight_left: s.weightLeft,
						weight_right: s.weightRight,
						notes: undefined
					}))
				);

				currentSession = {
					...currentSession,
					exercises: currentSession.exercises.map((e) =>
						e.id === exerciseId
							? {
									...e,
									sets: replaced.map((s) => ({
										id: s.id ?? 0,
										reps: Number(s.reps),
										weight: s.weight,
										weightLeft: s.weight_left ?? undefined,
										weightRight: s.weight_right ?? undefined
									}))
								}
							: e
					)
				};
			}
		} catch (e) {
			error = getErrorMessage(e);
			await refreshFromBackend();
		}
	}

	async function toggleExerciseSplitWeight(exerciseId: number, enabled: boolean) {
		if (!currentSession) return;
		const ex = currentSession.exercises.find((e) => e.id === exerciseId);
		if (!ex || ex.status === 'done') return;
		if (!ex.perSideWeight) return;
		if (enabled === ex.splitWeight) return;

		const nextSets = ex.sets.map((s) => {
			if (!enabled) {
				const left = s.weightLeft ?? s.weight;
				const right = s.weightRight ?? s.weight;
				return {
					...s,
					weight: Math.max(left, right),
					weightLeft: undefined,
					weightRight: undefined
				};
			}
			return {
				...s,
				weightLeft: s.weightLeft ?? s.weight,
				weightRight: s.weightRight ?? s.weight
			};
		});

		currentSession = {
			...currentSession,
			exercises: currentSession.exercises.map((e) =>
				e.id === exerciseId ? { ...e, splitWeight: enabled, sets: nextSets } : e
			)
		};

		try {
			await endExercise(exerciseId, {
				end_time: ex.endedAt,
				notes: ex.notes || undefined,
				per_side_weight: true,
				split_weight: enabled,
				settings: ex.settings.map((s) => ({ key: s.key, value: s.value }))
			});

			if (nextSets.length) {
				const replaced = await replaceSets(
					exerciseId,
					nextSets.map((s) => ({
						reps: s.reps,
						weight: s.weight,
						weight_left: s.weightLeft,
						weight_right: s.weightRight,
						notes: undefined
					}))
				);

				currentSession = {
					...currentSession,
					exercises: currentSession.exercises.map((e) =>
						e.id === exerciseId
							? {
									...e,
									sets: replaced.map((s) => ({
										id: s.id ?? 0,
										reps: Number(s.reps),
										weight: s.weight,
										weightLeft: s.weight_left ?? undefined,
										weightRight: s.weight_right ?? undefined
									}))
								}
							: e
					)
				};
			}
		} catch (e) {
			error = getErrorMessage(e);
			await refreshFromBackend();
		}
	}

	function getLastTimeForExercise(name: string) {
		for (const session of recentSessions) {
			const match = session.exercises.find((e) => e.name === name);
			if (!match) continue;
			return {
				startedAt: session.startedAt,
				notes: match.notes,
				sets: match.sets,
				perSideWeight: match.perSideWeight,
				splitWeight: match.splitWeight
			};
		}
		return undefined;
	}

	function getQuickPicks(sessions: UiSession[]): string[] {
		const picks: string[] = [];
		const seen = new Set<string>();
		for (const session of sessions) {
			for (const ex of session.exercises) {
				if (seen.has(ex.name)) continue;
				seen.add(ex.name);
				picks.push(ex.name);
			}
		}
		return picks.slice(0, 6);
	}

	function getSuggestions(
		query: string,
		sessions: UiSession[],
		activeSession: UiSession | null
	): string[] {
		const term = query.trim().toLowerCase();
		if (!term) return [];

		const recentSet = new Set(getQuickPicks(sessions));
		const inSession = new Set((activeSession?.exercises ?? []).map((e) => e.name.toLowerCase()));

		const matches = exerciseLibrary.filter((name) => {
			if (inSession.has(name.toLowerCase())) return false;
			return name.toLowerCase().includes(term);
		});

		return matches
			.sort((a, b) => {
				const aRecent = recentSet.has(a);
				const bRecent = recentSet.has(b);
				if (aRecent && !bRecent) return -1;
				if (!aRecent && bRecent) return 1;

				const aStarts = a.toLowerCase().startsWith(term);
				const bStarts = b.toLowerCase().startsWith(term);
				if (aStarts && !bStarts) return -1;
				if (!aStarts && bStarts) return 1;

				return a.localeCompare(b);
			})
			.slice(0, MAX_SUGGESTIONS);
	}

	$: suggestions = getSuggestions(exerciseQuery, recentSessions, currentSession);

	function elapsedLabel(): string {
		if (!currentSession) return '';
		const diffMs = Math.max(0, nowMs - new Date(currentSession.startedAt).getTime());
		const minutes = Math.floor(diffMs / 60_000);
		if (minutes < 60) return `${minutes}m`;
		const hours = Math.floor(minutes / 60);
		const rem = minutes % 60;
		return rem ? `${hours}h ${rem}m` : `${hours}h`;
	}

	function totalSets(): number {
		if (!currentSession) return 0;
		return currentSession.exercises.reduce((count, e) => count + e.sets.length, 0);
	}

	function totalVolumeKg(): number {
		if (!currentSession) return 0;
		return currentSession.exercises.reduce(
			(total, e) =>
				total +
				e.sets.reduce((t, s) => {
					if (!e.perSideWeight) return t + s.reps * s.weight;
					if (!e.splitWeight) return t + s.reps * (s.weight * 2);
					const left = s.weightLeft ?? s.weight;
					const right = s.weightRight ?? s.weight;
					return t + s.reps * (left + right);
				}, 0),
			0
		);
	}
</script>

<div class="space-y-6">
	{#if error}
		<div class="alert variant-filled-error">{error}</div>
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
				{#if currentSession}
					<div class="text-sm opacity-80">
						Started {formatTime(currentSession.startedAt)} • {formatDateRelative(
							currentSession.startedAt
						)}
					</div>
					<div class="flex gap-2 w-full sm:w-auto">
						<button
							type="button"
							class="btn variant-soft-error flex-1 sm:flex-initial"
							on:click={cancelSession}
							disabled={loading}
						>
							Cancel
						</button>
						<button
							type="button"
							class="btn variant-filled-primary flex-1 sm:flex-initial"
							on:click={openEndModal}
							disabled={loading}
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
							disabled={loading}
						>
							Start session
						</button>
						<button
							type="button"
							class="btn variant-soft w-full sm:w-auto"
							on:click={() => startSession('demo')}
							disabled={loading}
						>
							Load demo
						</button>
					</div>
				{/if}
			</div>
		</div>

		{#if currentSession}
			<div class="relative mt-5 grid gap-3 grid-cols-2 sm:grid-cols-4">
				<div class="card variant-glass-surface p-3">
					<div class="text-xs font-semibold opacity-70">Elapsed</div>
					<div class="text-lg font-bold">{elapsedLabel()}</div>
				</div>
				<div class="card variant-glass-surface p-3">
					<div class="text-xs font-semibold opacity-70">Exercises</div>
					<div class="text-lg font-bold">{currentSession.exercises.length}</div>
				</div>
				<div class="card variant-glass-surface p-3">
					<div class="text-xs font-semibold opacity-70">Sets</div>
					<div class="text-lg font-bold">{totalSets()}</div>
				</div>
				<div class="card variant-glass-surface p-3">
					<div class="text-xs font-semibold opacity-70">Volume</div>
					<div class="text-lg font-bold">{Math.round(totalVolumeKg())} kg</div>
				</div>
			</div>
		{/if}
	</header>

	<div class="grid gap-6 md:grid-cols-12">
		<section class="md:col-span-7 lg:col-span-8 space-y-4 min-w-0">
			{#if currentSession}
				<div class="card variant-glass-surface p-4 space-y-2">
					<label class="block">
						<span class="text-sm font-semibold opacity-80">Session notes</span>
						<textarea
							class="textarea mt-1"
							rows="2"
							placeholder="How did it feel? Any cues to remember…"
							bind:value={sessionNotes}
						></textarea>
					</label>
				</div>

				<ExerciseComposer
					bind:query={exerciseQuery}
					disabled={loading || !currentSession}
					{suggestions}
					quickPicks={getQuickPicks(recentSessions)}
					on:add={(e) => addExercise(e.detail.name)}
				/>

				{#if currentSession.exercises.length === 0}
					<div class="card variant-ghost p-6 text-center space-y-2">
						<div class="text-lg font-semibold">Add your first exercise</div>
						<p class="opacity-70 text-sm max-w-prose mx-auto">
							Use the search above or tap a quick pick from your recent sessions.
						</p>
					</div>
				{:else}
					<div class="space-y-3">
						{#each currentSession.exercises as ex (ex.id)}
							<SessionExercise
								exercise={ex}
								isOpen={openExerciseId === ex.id}
								disabled={loading}
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
										e.detail.weightRight
									)}
								on:updateNotes={(e) => updateExerciseNotes(ex.id, e.detail.notes)}
								on:addSetting={(e) => addExerciseSetting(ex.id, e.detail.key, e.detail.value)}
								on:removeSetting={(e) => removeExerciseSetting(ex.id, e.detail.id)}
								on:updateSetting={(e) =>
									updateExerciseSetting(ex.id, e.detail.id, e.detail.key, e.detail.value)}
								on:togglePerSideWeight={(e) => toggleExercisePerSideWeight(ex.id, e.detail.enabled)}
								on:toggleSplitWeight={(e) => toggleExerciseSplitWeight(ex.id, e.detail.enabled)}
							/>
						{/each}
					</div>
				{/if}
			{:else}
				<div class="card variant-glass-surface p-6 space-y-3">
					<h2 class="text-xl font-semibold tracking-tight">Your landing page, rebuilt</h2>
					<p class="opacity-75">
						Log your session now, backed by your database, while keeping the last two sessions
						visible for instant recall.
					</p>
					<ol class="grid gap-2 text-sm opacity-80 list-decimal pl-5">
						<li>Start a session</li>
						<li>Add exercises (search + quick picks)</li>
						<li>Log sets + notes</li>
						<li>End session with mood + notes</li>
					</ol>
					<div class="flex flex-col sm:flex-row gap-2 pt-2">
						<button
							type="button"
							class="btn variant-filled-primary"
							on:click={() => startSession('empty')}
							disabled={loading}
						>
							Start session
						</button>
						<button
							type="button"
							class="btn variant-soft"
							on:click={() => startSession('demo')}
							disabled={loading}
						>
							Load demo session
						</button>
					</div>
				</div>
			{/if}
		</section>

		<aside class="md:col-span-5 lg:col-span-4 min-w-0">
			<RecentSessions
				sessions={recentSessions}
				canAdd={Boolean(currentSession) && !loading}
				disabled={loading || !currentSession}
				on:addExercise={(e) => addExercise(e.detail.name, e.detail)}
			/>
		</aside>
	</div>

	<EndSessionModal
		open={endModalOpen}
		bind:notes={endNotes}
		bind:mood={endMood}
		disabled={loading}
		on:cancel={() => (endModalOpen = false)}
		on:submit={submitEndSession}
	/>
</div>
