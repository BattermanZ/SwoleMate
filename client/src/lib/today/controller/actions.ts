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
import { kvSet } from '$lib/offline/storage';
import {
	deleteOfflineSession,
	listOfflineSessions,
	loadOfflineSession,
	saveOfflineSession,
	sessionKeyForId
} from '$lib/offline/todaySessions';
import { toUiSession, workoutIsActive } from '$lib/today/backend';
import type { UiSession } from '$lib/today/types';
import { createId } from '$lib/utils/id';
import { get } from 'svelte/store';
import {
	findInProgressOffline,
	hydrateOfflineState,
	persistInProgressSession,
	RECENT_SESSIONS_KEY,
	refreshPendingSyncCount,
	setOffline,
	syncOne
} from './offline';
import type { TodayState } from './state';
import { getErrorMessage, isNetworkFailure, makeLocalNumericId } from './utils';

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
	settings: UiSession['exercises'][number]['settings'];
	perSideWeight: boolean;
	splitWeight: boolean;
};

export function createTodayActions(state: TodayState) {
	const syncTimers = new Map<number, number>();
	let persistTimer: number | null = null;

	function schedulePersist() {
		if (typeof window === 'undefined') return;
		if (persistTimer) window.clearTimeout(persistTimer);
		persistTimer = window.setTimeout(() => {
			persistTimer = null;
			void persistInProgressSession(state);
		}, 450);
	}

	function resetLocalSessionUi() {
		state.exerciseQuery.set('');
		state.endMood.set(null);
		state.endNotes.set('');
		state.endModalOpen.set(false);
	}

	async function hydrateExerciseLibrary() {
		try {
			const types = await getExerciseTypes();
			const merged = new Set<string>([...EXERCISE_LIBRARY, ...types]);
			state.exerciseLibrary.set(Array.from(merged).sort((a, b) => a.localeCompare(b)));
		} catch {
			// ignore: local library is good enough
		}
	}

	async function refreshFromBackend() {
		state.loading.set(true);
		state.error.set(null);

		try {
			const workouts = await getWorkouts();
			const active = workouts.find((w) => workoutIsActive(w) && w.id != null);

			if (active?.id != null) {
				const data = await getWorkout(active.id);
				const next = toUiSession(data.workout, data.exercises);
				state.currentSession.set(next);
				state.sessionNotes.set(next.notes);
				state.openExerciseId.set(next.exercises[0]?.id ?? null);
			} else {
				const offlineInProgress = await findInProgressOffline();
				if (offlineInProgress?.session) {
					state.currentSession.set(offlineInProgress.session);
					state.sessionNotes.set(offlineInProgress.session.notes);
					state.openExerciseId.set(offlineInProgress.session.exercises[0]?.id ?? null);
					state.notice.set('Local session in progress. You can keep logging and sync later.');
				} else {
					state.currentSession.set(null);
					state.sessionNotes.set('');
					state.openExerciseId.set(null);
				}
			}

			const completed = workouts.filter((w) => w.id != null && !workoutIsActive(w)).slice(0, 2);
			const recent = await Promise.all(completed.map((w) => getWorkout(w.id!)));
			const nextRecent = recent.map((d) => toUiSession(d.workout, d.exercises));
			state.recentSessions.set(nextRecent);
			void kvSet(RECENT_SESSIONS_KEY, nextRecent);

			resetLocalSessionUi();
			state.offlineMode.set(false);
			await refreshPendingSyncCount(state);
			if (get(state.pendingSyncCount) && !get(state.notice)) {
				state.notice.set('Offline changes pending sync.');
			}
			if (!get(state.pendingSyncCount) && !get(state.currentSession)) {
				state.notice.set(null);
			}
		} catch (e) {
			if (isNetworkFailure(e)) {
				if (!get(state.offlineMode)) setOffline(state);
				state.error.set(null);
				await hydrateOfflineState(state);
			} else {
				state.error.set(getErrorMessage(e));
			}
		} finally {
			state.loading.set(false);
		}
	}

	async function startSession(mode: 'empty' | 'demo') {
		if (get(state.currentSession)) return;
		state.error.set(null);

		try {
			const demo = mode === 'demo' ? createDemoSession() : null;
			const startIso = demo?.startedAt ?? new Date().toISOString();

			state.loading.set(true);
			const created = await createWorkout({
				date: startIso,
				start_time: startIso,
				notes: demo?.notes?.trim() || undefined
			});

			state.currentSession.set({
				id: created.id,
				startedAt: startIso,
				notes: demo?.notes ?? '',
				exercises: []
			});
			state.sessionNotes.set(demo?.notes ?? '');
			state.openExerciseId.set(null);
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
			if (isNetworkFailure(e)) {
				setOffline(state, 'Offline mode: started a local session (will sync when back online).');
				const demo = mode === 'demo' ? createDemoSession() : null;
				const startIso = demo?.startedAt ?? new Date().toISOString();
				const localId = makeLocalNumericId();
				state.currentSession.set({
					id: localId,
					startedAt: startIso,
					notes: demo?.notes ?? '',
					exercises: []
				});
				state.sessionNotes.set(demo?.notes ?? '');
				state.openExerciseId.set(null);
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

				await persistInProgressSession(state);
			} else {
				state.error.set(getErrorMessage(e));
			}
			await refreshFromBackend();
		} finally {
			state.loading.set(false);
		}
	}

	async function cancelSession() {
		const session = get(state.currentSession);
		if (!session) return;
		state.error.set(null);

		try {
			if (get(state.offlineMode) || session.id < 0) {
				const key = sessionKeyForId(session.id);
				const existing = await loadOfflineSession(key).catch(() => null);
				if (existing?.server_workout_id) {
					await saveOfflineSession({
						...existing,
						status: 'pending_sync',
						cancel_workout: true,
						updated_at: new Date().toISOString()
					});
				} else {
					await deleteOfflineSession(key).catch(() => undefined);
				}
				state.currentSession.set(null);
				state.sessionNotes.set('');
				state.openExerciseId.set(null);
				resetLocalSessionUi();
				await refreshPendingSyncCount(state);
				state.notice.set('Session canceled locally.');
				return;
			}

			state.loading.set(true);
			await cancelWorkout(session.id);
			state.currentSession.set(null);
			state.sessionNotes.set('');
			state.openExerciseId.set(null);
			resetLocalSessionUi();
			await refreshFromBackend();
		} catch (e) {
			if (isNetworkFailure(e)) {
				setOffline(state);
				await persistInProgressSession(state);
				await hydrateOfflineState(state);
			} else {
				state.error.set(getErrorMessage(e));
			}
		} finally {
			state.loading.set(false);
		}
	}

	function openEndModal() {
		const session = get(state.currentSession);
		if (!session) return;
		state.endNotes.set(get(state.sessionNotes));
		state.endMood.set(null);
		state.endModalOpen.set(true);
	}

	async function submitEndSession() {
		const session = get(state.currentSession);
		const mood = get(state.endMood);
		if (!session || !mood) return;
		state.error.set(null);

		try {
			const endedAt = new Date().toISOString();

			if (get(state.offlineMode) || session.id < 0) {
				const endedExercises = session.exercises.map((e) =>
					e.status === 'done' ? e : { ...e, status: 'done' as const, endedAt }
				);
				const ended: UiSession = { ...session, endedAt, mood, exercises: endedExercises };
				state.currentSession.set(null);
				state.sessionNotes.set('');
				state.openExerciseId.set(null);
				resetLocalSessionUi();

				const key = sessionKeyForId(session.id);
				const existing = await loadOfflineSession(key).catch(() => null);
				await saveOfflineSession({
					key,
					status: 'pending_sync',
					updated_at: new Date().toISOString(),
					session: ended,
					end_mood: mood,
					end_notes: get(state.endNotes).trim() || undefined,
					server_workout_id: existing?.server_workout_id,
					server_exercise_ids_by_local: existing?.server_exercise_ids_by_local ?? {},
					deleted_server_exercise_ids: existing?.deleted_server_exercise_ids ?? []
				});
				await refreshPendingSyncCount(state);
				setOffline(state, 'Saved locally. Sync when you’re back online.');
				state.endModalOpen.set(false);
				return;
			}

			state.loading.set(true);
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
				notes: get(state.endNotes).trim() || undefined,
				feedback: mood
			});

			state.currentSession.set(null);
			state.sessionNotes.set('');
			state.openExerciseId.set(null);
			resetLocalSessionUi();
			await refreshFromBackend();
		} catch (e) {
			if (isNetworkFailure(e)) {
				setOffline(state);
				await persistInProgressSession(state);
				await hydrateOfflineState(state);
			} else {
				state.error.set(getErrorMessage(e));
			}
		} finally {
			state.loading.set(false);
		}
	}

	function toggleExercise(exerciseId: number) {
		state.openExerciseId.update((current) => (current === exerciseId ? null : exerciseId));
	}

	async function addExercise(name: string, options?: ExerciseSeedOptions, seedSets?: SeedSet[]) {
		const session = get(state.currentSession);
		if (!session) return;
		const trimmed = name.trim();
		if (!trimmed) return;

		state.error.set(null);

		try {
			const last = getLastTimeForExercise(trimmed);
			const startIso = new Date().toISOString();
			const perSideWeight = options?.perSideWeight ?? last?.perSideWeight ?? false;
			const requestedSplit = options?.splitWeight ?? last?.splitWeight ?? false;
			const splitWeight = perSideWeight ? requestedSplit : false;
			const settings =
				options?.settings ?? last?.settings.map((s) => ({ key: s.key, value: s.value })) ?? [];

			if (get(state.offlineMode) || session.id < 0) {
				const localExerciseId = makeLocalNumericId();
				const newExercise = {
					id: localExerciseId,
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

				state.currentSession.set({
					...session,
					exercises: [...session.exercises, newExercise]
				});
				state.openExerciseId.set(newExercise.id);
				state.exerciseQuery.set('');
				await persistInProgressSession(state);

				if (seedSets?.length) {
					for (const s of seedSets) {
						await addSet(newExercise.id, s.reps, s.weight, s.weightLeft, s.weightRight);
					}
				}
				return;
			}

			state.loading.set(true);
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

			state.currentSession.set({
				...session,
				exercises: [...session.exercises, newExercise]
			});

			state.openExerciseId.set(newExercise.id);
			state.exerciseQuery.set('');

			if (seedSets?.length) {
				for (const s of seedSets) {
					await addSet(newExercise.id, s.reps, s.weight, s.weightLeft, s.weightRight);
				}
			}
		} catch (e) {
			if (isNetworkFailure(e)) {
				setOffline(state);
				await persistInProgressSession(state);
				await addExercise(trimmed, options, seedSets);
			} else {
				state.error.set(getErrorMessage(e));
			}
		} finally {
			state.loading.set(false);
		}
	}

	async function removeExercise(exerciseId: number) {
		const session = get(state.currentSession);
		if (!session) return;
		state.error.set(null);

		try {
			if (get(state.offlineMode) || session.id < 0) {
				state.currentSession.set({
					...session,
					exercises: session.exercises.filter((e) => e.id !== exerciseId)
				});
				state.openExerciseId.update((current) => (current === exerciseId ? null : current));
				if (exerciseId > 0) {
					const key = sessionKeyForId(session.id);
					const existing = await loadOfflineSession(key).catch(() => null);
					const prev = existing?.deleted_server_exercise_ids ?? [];
					await persistInProgressSession(state, {
						deleted_server_exercise_ids: Array.from(new Set([...prev, exerciseId]))
					});
				} else {
					await persistInProgressSession(state);
				}
				return;
			}

			state.loading.set(true);
			await cancelExercise(exerciseId);
			state.currentSession.set({
				...session,
				exercises: session.exercises.filter((e) => e.id !== exerciseId)
			});
			state.openExerciseId.update((current) => (current === exerciseId ? null : current));
		} catch (e) {
			if (isNetworkFailure(e)) {
				setOffline(state);
				await persistInProgressSession(state);
				await hydrateOfflineState(state);
			} else {
				state.error.set(getErrorMessage(e));
				await refreshFromBackend();
			}
		} finally {
			state.loading.set(false);
		}
	}

	async function markExerciseDone(exerciseId: number) {
		const session = get(state.currentSession);
		if (!session) return;
		state.error.set(null);

		try {
			const endedAt = new Date().toISOString();
			const ex = session.exercises.find((e) => e.id === exerciseId);
			if (!ex) return;

			if (get(state.offlineMode) || session.id < 0 || exerciseId < 0) {
				state.currentSession.set({
					...session,
					exercises: session.exercises.map((e) =>
						e.id === exerciseId ? { ...e, status: 'done' as const, endedAt } : e
					)
				});
				await persistInProgressSession(state);
				return;
			}

			state.loading.set(true);
			await endExercise(exerciseId, {
				end_time: endedAt,
				notes: ex.notes || undefined,
				per_side_weight: ex.perSideWeight,
				split_weight: ex.splitWeight,
				settings: ex.settings.map((s) => ({ key: s.key, value: s.value }))
			});

			state.currentSession.set({
				...session,
				exercises: session.exercises.map((e) =>
					e.id === exerciseId ? { ...e, status: 'done' as const, endedAt } : e
				)
			});
		} catch (e) {
			if (isNetworkFailure(e)) {
				setOffline(state);
				await persistInProgressSession(state);
				await markExerciseDone(exerciseId);
			} else {
				state.error.set(getErrorMessage(e));
			}
		} finally {
			state.loading.set(false);
		}
	}

	async function addSet(
		exerciseId: number,
		reps: number,
		weight: number,
		weightLeft?: number,
		weightRight?: number
	) {
		const session = get(state.currentSession);
		if (!session) return;
		state.error.set(null);

		try {
			if (get(state.offlineMode) || session.id < 0 || exerciseId < 0) {
				const setId = makeLocalNumericId();
				state.currentSession.set({
					...session,
					exercises: session.exercises.map((e) => {
						if (e.id !== exerciseId) return e;
						return {
							...e,
							sets: [
								...e.sets,
								{
									id: setId,
									reps,
									weight,
									weightLeft,
									weightRight
								}
							]
						};
					})
				});
				await persistInProgressSession(state);
				return;
			}

			state.loading.set(true);
			const created = await createSet(exerciseId, {
				reps,
				weight,
				weight_left: weightLeft,
				weight_right: weightRight,
				notes: undefined
			});

			state.currentSession.set({
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
			if (isNetworkFailure(e)) {
				setOffline(state);
				await persistInProgressSession(state);
				await addSet(exerciseId, reps, weight, weightLeft, weightRight);
			} else {
				state.error.set(getErrorMessage(e));
			}
		} finally {
			state.loading.set(false);
		}
	}

	function scheduleExerciseSync(exerciseId: number) {
		if (typeof window === 'undefined') return;
		if (get(state.offlineMode) || exerciseId < 0) {
			schedulePersist();
			return;
		}
		const existing = syncTimers.get(exerciseId);
		if (existing) window.clearTimeout(existing);
		const timer = window.setTimeout(() => void syncExercise(exerciseId), 650);
		syncTimers.set(exerciseId, timer);
	}

	async function syncExercise(exerciseId: number) {
		if (get(state.offlineMode) || exerciseId < 0) return;
		const session = get(state.currentSession);
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
			if (isNetworkFailure(e)) {
				setOffline(state);
				await persistInProgressSession(state);
			} else {
				state.error.set(getErrorMessage(e));
			}
		}
	}

	function updateExerciseNotes(exerciseId: number, notes: string) {
		const session = get(state.currentSession);
		if (!session) return;
		state.currentSession.set({
			...session,
			exercises: session.exercises.map((e) => (e.id === exerciseId ? { ...e, notes } : e))
		});
		scheduleExerciseSync(exerciseId);
	}

	function addExerciseSetting(exerciseId: number, key: string, value: string) {
		const session = get(state.currentSession);
		if (!session) return;
		state.currentSession.set({
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
		const session = get(state.currentSession);
		if (!session) return;
		state.currentSession.set({
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
		const session = get(state.currentSession);
		if (!session) return;
		state.currentSession.set({
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
		const session = get(state.currentSession);
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

		state.currentSession.set({
			...session,
			exercises: session.exercises.map((e) =>
				e.id === exerciseId
					? { ...e, perSideWeight: enabled, splitWeight: nextSplit, sets: nextSets }
					: e
			)
		});

		if (get(state.offlineMode) || session.id < 0 || exerciseId < 0) {
			await persistInProgressSession(state);
			return;
		}

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

				const updated = get(state.currentSession);
				if (!updated) return;
				state.currentSession.set({
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
			if (isNetworkFailure(e)) {
				setOffline(state);
				await persistInProgressSession(state);
			} else {
				state.error.set(getErrorMessage(e));
				await refreshFromBackend();
			}
		}
	}

	async function toggleExerciseSplitWeight(exerciseId: number, enabled: boolean) {
		const session = get(state.currentSession);
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

		state.currentSession.set({
			...session,
			exercises: session.exercises.map((e) =>
				e.id === exerciseId ? { ...e, splitWeight: enabled, sets: nextSets } : e
			)
		});

		if (get(state.offlineMode) || session.id < 0 || exerciseId < 0) {
			await persistInProgressSession(state);
			return;
		}

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

				const updated = get(state.currentSession);
				if (!updated) return;
				state.currentSession.set({
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
			if (isNetworkFailure(e)) {
				setOffline(state);
				await persistInProgressSession(state);
			} else {
				state.error.set(getErrorMessage(e));
				await refreshFromBackend();
			}
		}
	}

	function getLastTimeForExercise(name: string): LastTime | undefined {
		const sessions = get(state.recentSessions);
		for (const session of sessions) {
			const match = session.exercises.find((e) => e.name === name);
			if (!match) continue;
			return {
				startedAt: session.startedAt,
				notes: match.notes,
				sets: match.sets,
				settings: match.settings,
				perSideWeight: match.perSideWeight,
				splitWeight: match.splitWeight
			};
		}
		return undefined;
	}

	state.sessionNotes.subscribe((notes) => {
		const session = get(state.currentSession);
		if (!session) return;
		if (session.notes === notes) return;
		state.currentSession.set({ ...session, notes });
		if (get(state.offlineMode) || session.id < 0) schedulePersist();
	});

	async function syncPendingSessions() {
		state.error.set(null);
		state.notice.set('Syncing offline changes…');

		try {
			const records = await listOfflineSessions();
			if (!records.length) {
				state.notice.set(null);
				return;
			}

			state.loading.set(true);
			for (const record of records) {
				await syncOne(record, {
					cancelExercise,
					cancelWorkout,
					createExercise,
					createWorkout,
					endExercise,
					endWorkout,
					replaceSets
				});
			}
			await refreshPendingSyncCount(state);
			state.offlineMode.set(false);
			state.notice.set(get(state.pendingSyncCount) ? 'Some changes are still pending sync.' : null);
			await refreshFromBackend();
		} catch (e) {
			if (isNetworkFailure(e)) {
				setOffline(state, 'Still offline. Your changes are safe and will sync later.');
			} else {
				state.error.set(getErrorMessage(e));
			}
		} finally {
			state.loading.set(false);
		}
	}

	function start() {
		void hydrateOfflineState(state);
		void refreshFromBackend();
		void hydrateExerciseLibrary();

		const onOnline = () => {
			if (get(state.pendingSyncCount)) void syncPendingSessions();
		};
		const onOffline = () => {
			setOffline(state);
			void persistInProgressSession(state);
		};
		window.addEventListener('online', onOnline);
		window.addEventListener('offline', onOffline);

		const timer = window.setInterval(() => state.nowMs.set(Date.now()), 10_000);
		return () => {
			window.clearInterval(timer);
			window.removeEventListener('online', onOnline);
			window.removeEventListener('offline', onOffline);
		};
	}

	return {
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
		syncPendingSessions,
		toggleExercise,
		toggleExercisePerSideWeight,
		toggleExerciseSplitWeight,
		updateExerciseNotes,
		updateExerciseSetting,

		// helpers
		getLastTimeForExercise
	};
}
