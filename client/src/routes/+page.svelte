<script lang="ts">
	import { onMount } from 'svelte';
	import ExerciseComposer from '$lib/components/today/ExerciseComposer.svelte';
	import EndSessionModal from '$lib/components/today/EndSessionModal.svelte';
	import RecentSessions from '$lib/components/today/RecentSessions.svelte';
	import SessionExercise from '$lib/components/today/SessionExercise.svelte';
	import {
		createDemoSession,
		createEmptySession,
		createMockRecentSessions,
		EXERCISE_LIBRARY,
		type UiMood,
		type UiSession
	} from '$lib/mocks/today';
	import { createId } from '$lib/utils/id';
	import { formatDateRelative, formatTime } from '$lib/utils/date';

	let nowMs = Date.now();

	let currentSession: UiSession | null = null;
	let recentSessions: UiSession[] = createMockRecentSessions();

	let openExerciseId: string | null = null;

	let exerciseQuery = '';
	let sessionNotes = '';
	let endMood: UiMood | null = null;
	let endModalOpen = false;
	let endNotes = '';

	const MAX_SUGGESTIONS = 10;

	onMount(() => {
		const timer = window.setInterval(() => (nowMs = Date.now()), 10_000);
		return () => window.clearInterval(timer);
	});

	$: if (currentSession && currentSession.notes !== sessionNotes) {
		currentSession = { ...currentSession, notes: sessionNotes };
	}

	function startSession(mode: 'empty' | 'demo') {
		currentSession = mode === 'demo' ? createDemoSession() : createEmptySession();
		sessionNotes = currentSession.notes;
		openExerciseId = currentSession.exercises[0]?.id ?? null;
		exerciseQuery = '';
		endMood = null;
		endNotes = '';
		endModalOpen = false;
	}

	function cancelSession() {
		currentSession = null;
		sessionNotes = '';
		openExerciseId = null;
		exerciseQuery = '';
		endMood = null;
		endNotes = '';
		endModalOpen = false;
	}

	function openEndModal() {
		if (!currentSession) return;
		endNotes = sessionNotes;
		endMood = null;
		endModalOpen = true;
	}

	function submitEndSession() {
		if (!currentSession || !endMood) return;
		const endedAt = new Date().toISOString();

		const completed: UiSession = {
			...currentSession,
			endedAt,
			mood: endMood,
			notes: endNotes.trim(),
			exercises: currentSession.exercises.map((e) => ({
				...e,
				status: 'done'
			}))
		};

		recentSessions = [completed, ...recentSessions].slice(0, 2);
		cancelSession();
	}

	function toggleExercise(exerciseId: string) {
		openExerciseId = openExerciseId === exerciseId ? null : exerciseId;
	}

	function addExercise(
		name: string,
		options?: {
			notes?: string;
			perSideWeight?: boolean;
			splitWeight?: boolean;
			settings?: Array<{ key: string; value: string }>;
		}
	) {
		if (!currentSession) return;
		const trimmed = name.trim();
		if (!trimmed) return;

		const newExercise = {
			id: createId('ex'),
			name: trimmed,
			notes: options?.notes?.trim() ?? '',
			status: 'active' as const,
			perSideWeight: options?.perSideWeight ?? false,
			splitWeight: options?.splitWeight ?? false,
			settings: (options?.settings ?? []).map((s) => ({
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
	}

	function removeExercise(exerciseId: string) {
		if (!currentSession) return;
		currentSession = {
			...currentSession,
			exercises: currentSession.exercises.filter((e) => e.id !== exerciseId)
		};
		if (openExerciseId === exerciseId) openExerciseId = null;
	}

	function markExerciseDone(exerciseId: string) {
		if (!currentSession) return;
		currentSession = {
			...currentSession,
			exercises: currentSession.exercises.map((e) =>
				e.id === exerciseId ? { ...e, status: 'done' as const } : e
			)
		};
	}

	function addSet(
		exerciseId: string,
		reps: number,
		weight: number,
		weightLeft?: number,
		weightRight?: number
	) {
		if (!currentSession) return;
		currentSession = {
			...currentSession,
			exercises: currentSession.exercises.map((e) => {
				if (e.id !== exerciseId) return e;
				return {
					...e,
					sets: [
						...e.sets,
						{
							id: createId('set'),
							reps,
							weight,
							weightLeft,
							weightRight
						}
					]
				};
			})
		};
	}

	function updateExerciseNotes(exerciseId: string, notes: string) {
		if (!currentSession) return;
		currentSession = {
			...currentSession,
			exercises: currentSession.exercises.map((e) => (e.id === exerciseId ? { ...e, notes } : e))
		};
	}

	function addExerciseSetting(exerciseId: string, key: string, value: string) {
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
	}

	function removeExerciseSetting(exerciseId: string, settingId: string) {
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
	}

	function updateExerciseSetting(
		exerciseId: string,
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
	}

	function toggleExercisePerSideWeight(exerciseId: string, enabled: boolean) {
		if (!currentSession) return;
		currentSession = {
			...currentSession,
			exercises: currentSession.exercises.map((e) => {
				if (e.id !== exerciseId) return e;
				if (!enabled) {
					const normalized = e.sets.map((s) => {
						if (!e.perSideWeight) return s;
						if (!e.splitWeight)
							return { ...s, weight: s.weight * 2, weightLeft: undefined, weightRight: undefined };
						const left = s.weightLeft ?? s.weight;
						const right = s.weightRight ?? s.weight;
						return { ...s, weight: left + right, weightLeft: undefined, weightRight: undefined };
					});
					return { ...e, perSideWeight: false, splitWeight: false, sets: normalized };
				}

				const normalized = e.sets.map((s) => ({
					...s,
					weightLeft: s.weightLeft ?? s.weight,
					weightRight: s.weightRight ?? s.weight
				}));
				return { ...e, perSideWeight: true, splitWeight: e.splitWeight, sets: normalized };
			})
		};
	}

	function toggleExerciseSplitWeight(exerciseId: string, enabled: boolean) {
		if (!currentSession) return;
		currentSession = {
			...currentSession,
			exercises: currentSession.exercises.map((e) => {
				if (e.id !== exerciseId) return e;
				if (!e.perSideWeight) return { ...e, splitWeight: false };
				const normalized = e.sets.map((s) => {
					if (!enabled) return { ...s, weightLeft: undefined, weightRight: undefined };
					return {
						...s,
						weightLeft: s.weightLeft ?? s.weight,
						weightRight: s.weightRight ?? s.weight
					};
				});
				return { ...e, splitWeight: enabled, sets: normalized };
			})
		};
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

		const matches = EXERCISE_LIBRARY.filter((name) => {
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
						>
							Cancel
						</button>
						<button
							type="button"
							class="btn variant-filled-primary flex-1 sm:flex-initial"
							on:click={openEndModal}
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
						>
							Start session
						</button>
						<button
							type="button"
							class="btn variant-soft w-full sm:w-auto"
							on:click={() => startSession('demo')}
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
		<section class="md:col-span-7 lg:col-span-8 space-y-4">
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
					disabled={!currentSession}
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
								disabled={false}
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
						This is a UI-first mock: everything is local, fast, and designed around logging “now”
						while keeping the last two sessions visible.
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
						>
							Start session
						</button>
						<button type="button" class="btn variant-soft" on:click={() => startSession('demo')}>
							Load demo session
						</button>
					</div>
				</div>
			{/if}
		</section>

		<aside class="md:col-span-5 lg:col-span-4">
			<RecentSessions
				sessions={recentSessions}
				canAdd={Boolean(currentSession)}
				disabled={!currentSession}
				on:addExercise={(e) => addExercise(e.detail.name, e.detail)}
			/>
		</aside>
	</div>

	<EndSessionModal
		open={endModalOpen}
		bind:notes={endNotes}
		bind:mood={endMood}
		on:cancel={() => (endModalOpen = false)}
		on:submit={submitEndSession}
	/>
</div>
