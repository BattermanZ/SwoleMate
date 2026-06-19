import { kvGet } from '$lib/offline/storage';
import {
	deleteOfflineSession,
	listOfflineSessions,
	loadOfflineSession,
	saveOfflineSession,
	sessionKeyForId,
	type OfflineSessionRecord
} from '$lib/offline/todaySessions';
import type { Set } from '$lib/types';
import type { UiSession } from '$lib/today/types';
import { trackingFieldsSetting } from '$lib/today/tracking';
import { scopedKey } from '$lib/auth/scope';
import { get, type Writable } from 'svelte/store';

export const RECENT_SESSIONS_KEY = 'offline.today.recentSessions';

function recentSessionsKey(): string {
	return scopedKey(RECENT_SESSIONS_KEY);
}

export function getRecentSessionsStorageKey(): string {
	return recentSessionsKey();
}

export type OfflineStoreAccess = {
	currentSession: Writable<UiSession | null>;
	recentSessions: Writable<UiSession[]>;
	sessionNotes: Writable<string>;
	openExerciseIds: Writable<number[]>;
	notice: Writable<string | null>;
	offlineMode: Writable<boolean>;
	pendingSyncCount: Writable<number>;
};

export function setOffline(
	access: Pick<OfflineStoreAccess, 'offlineMode' | 'notice'>,
	message?: string
) {
	access.offlineMode.set(true);
	access.notice.set(
		message ?? 'Offline mode: changes are saved on this device and will sync later.'
	);
}

export async function refreshPendingSyncCount(
	access: Pick<OfflineStoreAccess, 'pendingSyncCount'>
): Promise<number> {
	const sessions = await listOfflineSessions().catch(() => []);
	const count = sessions.filter(
		(r) =>
			r.cancel_workout ||
			r.status === 'pending_sync' ||
			(r.status === 'in_progress' && r.session.id < 0 && !r.server_workout_id)
	).length;
	access.pendingSyncCount.set(count);
	return count;
}

export async function findInProgressOffline(): Promise<OfflineSessionRecord | null> {
	const records = await listOfflineSessions().catch(() => []);
	return records.find((r) => r.status === 'in_progress') ?? null;
}

export async function hydrateOfflineState(access: OfflineStoreAccess) {
	await refreshPendingSyncCount(access);

	const cachedRecent = await kvGet<UiSession[]>(recentSessionsKey()).catch(() => null);
	if (cachedRecent?.length) access.recentSessions.set(cachedRecent);

	const inProgress = await findInProgressOffline();
	if (!get(access.currentSession) && inProgress?.session) {
		access.currentSession.set(inProgress.session);
		access.sessionNotes.set(inProgress.session.notes);
		access.openExerciseIds.set(
			inProgress.session.exercises
				.filter((exercise) => exercise.status !== 'done')
				.map((exercise) => exercise.id)
		);
	}
}

export async function persistInProgressSession(
	access: Pick<OfflineStoreAccess, 'currentSession' | 'pendingSyncCount'>,
	extra?: Partial<OfflineSessionRecord>
) {
	const session = get(access.currentSession);
	if (!session) return;
	const key = sessionKeyForId(session.id);
	const existing = await loadOfflineSession(key).catch(() => null);
	const record: OfflineSessionRecord = {
		key,
		status: 'in_progress',
		updated_at: new Date().toISOString(),
		session,
		server_workout_id: existing?.server_workout_id ?? (session.id > 0 ? session.id : undefined),
		server_exercise_ids_by_local: existing?.server_exercise_ids_by_local ?? {},
		deleted_server_exercise_ids: existing?.deleted_server_exercise_ids ?? [],
		cancel_workout: existing?.cancel_workout,
		...extra
	};
	await saveOfflineSession(record);
	await refreshPendingSyncCount(access);
}

export type SyncApi = {
	cancelExercise: (id: number) => Promise<void>;
	cancelWorkout: (id: number) => Promise<void>;
	createExercise: (
		workoutId: number,
		exercise: {
			exercise_type: string;
			start_time: string;
			notes?: string;
			per_side_weight: boolean;
			split_weight: boolean;
			settings?: Array<{ key: string; value: string }>;
		}
	) => Promise<{ id: number }>;
	createWorkout: (workout: {
		date: string;
		start_time: string;
		notes?: string;
		timezone_offset_minutes?: number;
	}) => Promise<{ id: number }>;
	endExercise: (
		id: number,
		payload: {
			end_time: string;
			notes?: string;
			per_side_weight: boolean;
			split_weight: boolean;
			settings: Array<{ key: string; value: string }>;
		}
	) => Promise<void>;
	endWorkout: (
		id: number,
		payload: {
			end_time: string;
			notes?: string;
			feedback: '😊' | '😐' | '😞';
		}
	) => Promise<void>;
	replaceSets: (
		exerciseId: number,
		sets: Array<{
			reps: number;
			weight: number;
			weight_left?: number;
			weight_right?: number;
			duration_seconds?: number;
			notes?: string;
		}>
	) => Promise<Set[]>;
};

export async function syncOne(record: OfflineSessionRecord, api: SyncApi) {
	if (record.cancel_workout) {
		const id = record.server_workout_id ?? (record.session.id > 0 ? record.session.id : undefined);
		if (id) await api.cancelWorkout(id);
		await deleteOfflineSession(record.key);
		return;
	}

	let workoutId =
		record.server_workout_id ?? (record.session.id > 0 ? record.session.id : undefined);
	const exerciseMap = record.server_exercise_ids_by_local ?? {};

	// Persist mapping progress so that a failure mid-replay (likely, since we just
	// regained connectivity) resumes from where it stopped rather than restarting
	// — restarting would re-run createWorkout/createExercise and duplicate data.
	const checkpoint = async () => {
		await saveOfflineSession({
			...record,
			server_workout_id: workoutId,
			server_exercise_ids_by_local: exerciseMap,
			updated_at: new Date().toISOString()
		});
	};

	if (!workoutId) {
		const created = await api.createWorkout({
			date: record.session.startedAt,
			start_time: record.session.startedAt,
			notes: record.session.notes.trim() || undefined,
			timezone_offset_minutes: record.session.timezoneOffsetMinutes
		});
		workoutId = created.id;
		await checkpoint();
	}

	for (const ex of record.session.exercises) {
		let exerciseId = ex.id > 0 ? ex.id : exerciseMap[ex.id];
		if (!exerciseId) {
			const created = await api.createExercise(workoutId, {
				exercise_type: ex.name,
				start_time: ex.startedAt,
				notes: ex.notes.trim() || undefined,
				per_side_weight: ex.perSideWeight,
				split_weight: ex.splitWeight,
				settings: ex.settings.length
					? [
							...ex.settings.map((s) => ({ key: s.key, value: s.value })),
							trackingFieldsSetting({
								reps: ex.tracksReps ?? true,
								time: ex.tracksTime ?? false,
								weight: ex.tracksWeight ?? true
							})
						]
					: [
							trackingFieldsSetting({
								reps: ex.tracksReps ?? true,
								time: ex.tracksTime ?? false,
								weight: ex.tracksWeight ?? true
							})
						]
			});
			exerciseId = created.id;
			exerciseMap[ex.id] = created.id;
			await checkpoint();
		}

		await api.replaceSets(
			exerciseId,
			ex.sets.map((s) => ({
				reps: s.reps,
				weight: s.weight,
				weight_left: s.weightLeft,
				weight_right: s.weightRight,
				duration_seconds: s.durationSeconds,
				notes: undefined
			}))
		);

		const endTime =
			ex.status === 'done'
				? ex.endedAt
				: record.session.endedAt
					? record.session.endedAt
					: ex.endedAt;
		await api.endExercise(exerciseId, {
			end_time: endTime,
			notes: ex.notes.trim() || undefined,
			per_side_weight: ex.perSideWeight,
			split_weight: ex.splitWeight,
			settings: [
				...ex.settings.map((s) => ({ key: s.key, value: s.value })),
				trackingFieldsSetting({
					reps: ex.tracksReps ?? true,
					time: ex.tracksTime ?? false,
					weight: ex.tracksWeight ?? true
				})
			]
		});
	}

	if (record.deleted_server_exercise_ids?.length) {
		for (const id of record.deleted_server_exercise_ids) {
			await api.cancelExercise(id);
		}
	}

	if (record.status === 'pending_sync' && record.end_mood && record.session.endedAt) {
		await api.endWorkout(workoutId, {
			end_time: record.session.endedAt,
			notes: record.end_notes?.trim() || undefined,
			feedback: record.end_mood
		});
		await deleteOfflineSession(record.key);
		return;
	}

	await saveOfflineSession({
		...record,
		server_workout_id: workoutId,
		server_exercise_ids_by_local: exerciseMap,
		deleted_server_exercise_ids: [],
		updated_at: new Date().toISOString()
	});
}
