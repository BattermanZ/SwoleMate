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
import { EXERCISE_LIBRARY, createDemoSession } from '$lib/mocks/today';
import { toUiSession, workoutIsActive } from '$lib/today/backend';
import type { UiMood, UiSession } from '$lib/today/types';
import { createId } from '$lib/utils/id';
import { derived, get, writable } from 'svelte/store';

type ExerciseSeedOptions = {
	notes?: string;
	perSideWeight?: boolean;
	splitWeight?: boolean;
	settings?: Array<{ key: string; value: string }>;
};

type SeedSet = { reps: number; weight: number; weightLeft?: number; weightRight?: number };

type LastTime = {
	startedAt: string;
	notes: string;
	sets: UiSession['exercises'][number]['sets'];
	perSideWeight: boolean;
	splitWeight: boolean;
};

function getErrorMessage(e: unknown): string {
	if (e instanceof Error) return e.message;
	return 'Something went wrong';
}

export function createTodayController() {
	const nowMs = writable(Date.now());

	const currentSession = writable<UiSession | null>(null);
	const recentSessions = writable<UiSession[]>([]);
	const openExerciseId = writable<number | null>(null);

	const exerciseQuery = writable('');
	const sessionNotes = writable('');
	const endMood = writable<UiMood | null>(null);
	const endModalOpen = writable(false);
	const endNotes = writable('');
	const loading = writable(false);
	const error = writable<string | null>(null);

	const exerciseLibrary = writable<string[]>(EXERCISE_LIBRARY);

	const syncTimers = new Map<number, number>();

	function resetLocalSessionUi() {
		exerciseQuery.set('');
		endMood.set(null);
		endNotes.set('');
		endModalOpen.set(false);
	}

	const elapsedLabel = derived([nowMs, currentSession], ([$nowMs, $currentSession]) => {
		if (!$currentSession) return '';
		const diffMs = Math.max(0, $nowMs - new Date($currentSession.startedAt).getTime());
		const minutes = Math.floor(diffMs / 60_000);
		if (minutes < 60) return `${minutes}m`;
		const hours = Math.floor(minutes / 60);
		const rem = minutes % 60;
		return rem ? `${hours}h ${rem}m` : `${hours}h`;
	});

	const totalSets = derived(currentSession, ($currentSession) => {
		if (!$currentSession) return 0;
		return $currentSession.exercises.reduce((count, e) => count + e.sets.length, 0);
	});

	const totalVolumeKg = derived(currentSession, ($currentSession) => {
		if (!$currentSession) return 0;
		return $currentSession.exercises.reduce(
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
	});

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
		activeSession: UiSession | null,
		library: string[]
	): string[] {
		const term = query.trim().toLowerCase();
		if (!term) return [];

		const recentSet = new Set(getQuickPicks(sessions));
		const inSession = new Set((activeSession?.exercises ?? []).map((e) => e.name.toLowerCase()));

		const matches = library.filter((name) => {
			if (inSession.has(name.toLowerCase())) return false;
			return name.toLowerCase().includes(term);
		});

		const MAX_SUGGESTIONS = 10;
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

	const quickPicks = derived(recentSessions, ($recentSessions) => getQuickPicks($recentSessions));

	const suggestions = derived(
		[exerciseQuery, recentSessions, currentSession, exerciseLibrary],
		([$exerciseQuery, $recentSessions, $currentSession, $exerciseLibrary]) =>
			getSuggestions($exerciseQuery, $recentSessions, $currentSession, $exerciseLibrary)
	);

	async function hydrateExerciseLibrary() {
		try {
			const types = await getExerciseTypes();
			const merged = new Set<string>([...EXERCISE_LIBRARY, ...types]);
			exerciseLibrary.set(Array.from(merged).sort((a, b) => a.localeCompare(b)));
		} catch {
			// ignore: local library is good enough
		}
	}

	async function refreshFromBackend() {
		loading.set(true);
		error.set(null);

		try {
			const workouts = await getWorkouts();
			const active = workouts.find((w) => workoutIsActive(w) && w.id != null);

			if (active?.id != null) {
				const data = await getWorkout(active.id);
				const next = toUiSession(data.workout, data.exercises);
				currentSession.set(next);
				sessionNotes.set(next.notes);
				openExerciseId.set(next.exercises[0]?.id ?? null);
			} else {
				currentSession.set(null);
				sessionNotes.set('');
				openExerciseId.set(null);
			}

			const completed = workouts.filter((w) => w.id != null && !workoutIsActive(w)).slice(0, 2);
			const recent = await Promise.all(completed.map((w) => getWorkout(w.id!)));
			recentSessions.set(recent.map((d) => toUiSession(d.workout, d.exercises)));

			resetLocalSessionUi();
		} catch (e) {
			error.set(getErrorMessage(e));
		} finally {
			loading.set(false);
		}
	}

	async function startSession(mode: 'empty' | 'demo') {
		if (get(currentSession)) return;
		loading.set(true);
		error.set(null);

		try {
			const demo = mode === 'demo' ? createDemoSession() : null;
			const startIso = demo?.startedAt ?? new Date().toISOString();

			const created = await createWorkout({
				date: startIso,
				start_time: startIso,
				notes: demo?.notes?.trim() || undefined
			});

			currentSession.set({
				id: created.id,
				startedAt: startIso,
				notes: demo?.notes ?? '',
				exercises: []
			});
			sessionNotes.set(demo?.notes ?? '');
			openExerciseId.set(null);
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
			error.set(getErrorMessage(e));
			await refreshFromBackend();
		} finally {
			loading.set(false);
		}
	}

	async function cancelSession() {
		const session = get(currentSession);
		if (!session) return;
		loading.set(true);
		error.set(null);

		try {
			await cancelWorkout(session.id);
			currentSession.set(null);
			sessionNotes.set('');
			openExerciseId.set(null);
			resetLocalSessionUi();
			await refreshFromBackend();
		} catch (e) {
			error.set(getErrorMessage(e));
		} finally {
			loading.set(false);
		}
	}

	function openEndModal() {
		const session = get(currentSession);
		if (!session) return;
		endNotes.set(get(sessionNotes));
		endMood.set(null);
		endModalOpen.set(true);
	}

	async function submitEndSession() {
		const session = get(currentSession);
		const mood = get(endMood);
		if (!session || !mood) return;
		loading.set(true);
		error.set(null);

		try {
			const endedAt = new Date().toISOString();

			await Promise.all(
				session.exercises
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

			await endWorkout(session.id, {
				end_time: endedAt,
				notes: get(endNotes).trim() || undefined,
				feedback: mood
			});

			currentSession.set(null);
			sessionNotes.set('');
			openExerciseId.set(null);
			resetLocalSessionUi();
			await refreshFromBackend();
		} catch (e) {
			error.set(getErrorMessage(e));
		} finally {
			loading.set(false);
		}
	}

	function toggleExercise(exerciseId: number) {
		openExerciseId.update((current) => (current === exerciseId ? null : exerciseId));
	}

	async function addExercise(name: string, options?: ExerciseSeedOptions, seedSets?: SeedSet[]) {
		const session = get(currentSession);
		if (!session) return;
		const trimmed = name.trim();
		if (!trimmed) return;

		loading.set(true);
		error.set(null);

		try {
			const startIso = new Date().toISOString();
			const perSideWeight = options?.perSideWeight ?? false;
			const splitWeight = options?.splitWeight ?? false;
			const settings = options?.settings ?? [];

			const created = await createExercise(session.id, {
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

			currentSession.set({
				...session,
				exercises: [...session.exercises, newExercise]
			});

			openExerciseId.set(newExercise.id);
			exerciseQuery.set('');

			if (seedSets?.length) {
				for (const s of seedSets) {
					await addSet(newExercise.id, s.reps, s.weight, s.weightLeft, s.weightRight);
				}
			}
		} catch (e) {
			error.set(getErrorMessage(e));
		} finally {
			loading.set(false);
		}
	}

	async function removeExercise(exerciseId: number) {
		const session = get(currentSession);
		if (!session) return;
		loading.set(true);
		error.set(null);

		try {
			await cancelExercise(exerciseId);
			currentSession.set({
				...session,
				exercises: session.exercises.filter((e) => e.id !== exerciseId)
			});
			openExerciseId.update((current) => (current === exerciseId ? null : current));
		} catch (e) {
			error.set(getErrorMessage(e));
			await refreshFromBackend();
		} finally {
			loading.set(false);
		}
	}

	async function markExerciseDone(exerciseId: number) {
		const session = get(currentSession);
		if (!session) return;
		loading.set(true);
		error.set(null);

		try {
			const endedAt = new Date().toISOString();
			const ex = session.exercises.find((e) => e.id === exerciseId);
			if (!ex) return;

			await endExercise(exerciseId, {
				end_time: endedAt,
				notes: ex.notes || undefined,
				per_side_weight: ex.perSideWeight,
				split_weight: ex.splitWeight,
				settings: ex.settings.map((s) => ({ key: s.key, value: s.value }))
			});

			currentSession.set({
				...session,
				exercises: session.exercises.map((e) =>
					e.id === exerciseId ? { ...e, status: 'done' as const, endedAt } : e
				)
			});
		} catch (e) {
			error.set(getErrorMessage(e));
		} finally {
			loading.set(false);
		}
	}

	async function addSet(
		exerciseId: number,
		reps: number,
		weight: number,
		weightLeft?: number,
		weightRight?: number
	) {
		const session = get(currentSession);
		if (!session) return;
		loading.set(true);
		error.set(null);

		try {
			const created = await createSet(exerciseId, {
				reps,
				weight,
				weight_left: weightLeft,
				weight_right: weightRight,
				notes: undefined
			});

			currentSession.set({
				...session,
				exercises: session.exercises.map((e) => {
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
			});
		} catch (e) {
			error.set(getErrorMessage(e));
		} finally {
			loading.set(false);
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
		const session = get(currentSession);
		if (!session) return;
		const ex = session.exercises.find((e) => e.id === exerciseId);
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
			error.set(getErrorMessage(e));
		}
	}

	function updateExerciseNotes(exerciseId: number, notes: string) {
		const session = get(currentSession);
		if (!session) return;
		currentSession.set({
			...session,
			exercises: session.exercises.map((e) => (e.id === exerciseId ? { ...e, notes } : e))
		});
		scheduleExerciseSync(exerciseId);
	}

	function addExerciseSetting(exerciseId: number, key: string, value: string) {
		const session = get(currentSession);
		if (!session) return;
		currentSession.set({
			...session,
			exercises: session.exercises.map((e) => {
				if (e.id !== exerciseId) return e;
				return {
					...e,
					settings: [...e.settings, { id: createId('setting'), key, value }]
				};
			})
		});
		scheduleExerciseSync(exerciseId);
	}

	function removeExerciseSetting(exerciseId: number, settingId: string) {
		const session = get(currentSession);
		if (!session) return;
		currentSession.set({
			...session,
			exercises: session.exercises.map((e) => {
				if (e.id !== exerciseId) return e;
				return {
					...e,
					settings: e.settings.filter((s) => s.id !== settingId)
				};
			})
		});
		scheduleExerciseSync(exerciseId);
	}

	function updateExerciseSetting(
		exerciseId: number,
		settingId: string,
		key: string,
		value: string
	) {
		const session = get(currentSession);
		if (!session) return;
		currentSession.set({
			...session,
			exercises: session.exercises.map((e) => {
				if (e.id !== exerciseId) return e;
				return {
					...e,
					settings: e.settings.map((s) => (s.id === settingId ? { ...s, key, value } : s))
				};
			})
		});
		scheduleExerciseSync(exerciseId);
	}

	async function toggleExercisePerSideWeight(exerciseId: number, enabled: boolean) {
		const session = get(currentSession);
		if (!session) return;
		const ex = session.exercises.find((e) => e.id === exerciseId);
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

		currentSession.set({
			...session,
			exercises: session.exercises.map((e) =>
				e.id === exerciseId
					? { ...e, perSideWeight: enabled, splitWeight: nextSplit, sets: nextSets }
					: e
			)
		});

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

				const updated = get(currentSession);
				if (!updated) return;
				currentSession.set({
					...updated,
					exercises: updated.exercises.map((e) =>
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
				});
			}
		} catch (e) {
			error.set(getErrorMessage(e));
			await refreshFromBackend();
		}
	}

	async function toggleExerciseSplitWeight(exerciseId: number, enabled: boolean) {
		const session = get(currentSession);
		if (!session) return;
		const ex = session.exercises.find((e) => e.id === exerciseId);
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

		currentSession.set({
			...session,
			exercises: session.exercises.map((e) =>
				e.id === exerciseId ? { ...e, splitWeight: enabled, sets: nextSets } : e
			)
		});

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

				const updated = get(currentSession);
				if (!updated) return;
				currentSession.set({
					...updated,
					exercises: updated.exercises.map((e) =>
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
				});
			}
		} catch (e) {
			error.set(getErrorMessage(e));
			await refreshFromBackend();
		}
	}

	function getLastTimeForExercise(name: string): LastTime | undefined {
		const sessions = get(recentSessions);
		for (const session of sessions) {
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

	sessionNotes.subscribe((notes) => {
		const session = get(currentSession);
		if (!session) return;
		if (session.notes === notes) return;
		currentSession.set({ ...session, notes });
	});

	function start() {
		void refreshFromBackend();
		void hydrateExerciseLibrary();
		const timer = window.setInterval(() => nowMs.set(Date.now()), 10_000);
		return () => window.clearInterval(timer);
	}

	return {
		// stores
		currentSession,
		elapsedLabel,
		endModalOpen,
		endMood,
		endNotes,
		error,
		exerciseQuery,
		loading,
		openExerciseId,
		quickPicks,
		recentSessions,
		sessionNotes,
		suggestions,
		totalSets,
		totalVolumeKg,

		// actions
		addExercise,
		addExerciseSetting,
		addSet,
		cancelSession,
		markExerciseDone,
		openEndModal,
		refreshFromBackend,
		removeExercise,
		removeExerciseSetting,
		start,
		startSession,
		submitEndSession,
		toggleExercise,
		toggleExercisePerSideWeight,
		toggleExerciseSplitWeight,
		updateExerciseNotes,
		updateExerciseSetting,

		// helpers
		getLastTimeForExercise
	};
}
