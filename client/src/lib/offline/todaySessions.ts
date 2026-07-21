import type { UiMood, UiSession } from '$lib/today/types';
import { kvDelete, kvGet, kvListKeys, kvSet } from '$lib/offline/storage';
import { hasActiveUserScope, scopedKey } from '$lib/auth/scope';
import { logger } from '$lib/logger';

export type OfflineSessionStatus = 'in_progress' | 'pending_sync';

export type OfflineSessionRecord = {
	key: string;
	status: OfflineSessionStatus;
	updated_at: string;
	session: UiSession;
	end_mood?: UiMood;
	end_notes?: string;
	server_workout_id?: number;
	server_exercise_ids_by_local?: Record<number, number>;
	cancel_workout?: boolean;
	deleted_server_exercise_ids?: number[];
};

const PREFIX = 'offline.today.session.';

function sessionPrefix(): string {
	return scopedKey(PREFIX);
}

export function sessionKeyForId(sessionId: number): string {
	return `${sessionPrefix()}${sessionId}`;
}

export async function saveOfflineSession(record: OfflineSessionRecord): Promise<void> {
	// Never persist an offline session while the active user is unknown: it would
	// land in the unscoped `anon:` bucket, be orphaned once the id resolves (scoped
	// reads use `u<id>:`), and be momentarily visible to another account on a shared
	// device (F-LOW-2). The in-memory session survives; a later persist (after the
	// scope resolves) writes it under the correct key.
	if (!hasActiveUserScope()) {
		logger.warn('offline', 'skipping offline session persist: no active user scope');
		return;
	}
	await kvSet(record.key, record);
}

export async function loadOfflineSession(key: string): Promise<OfflineSessionRecord | null> {
	return kvGet<OfflineSessionRecord>(key);
}

export async function deleteOfflineSession(key: string): Promise<void> {
	await kvDelete(key);
}

export async function listOfflineSessions(): Promise<OfflineSessionRecord[]> {
	const keys = await kvListKeys(sessionPrefix());
	const records = await Promise.all(keys.map((k) => kvGet<OfflineSessionRecord>(k)));
	return records
		.filter((r): r is OfflineSessionRecord => Boolean(r))
		.sort((a, b) => b.updated_at.localeCompare(a.updated_at));
}
