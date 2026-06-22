import { writable } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { OfflineSessionRecord } from '$lib/offline/todaySessions';
import type { UiSession } from '$lib/today/types';

const store = vi.hoisted(() => ({ records: [] as OfflineSessionRecord[], saved: [] as OfflineSessionRecord[], deleted: [] as string[] }));

const sessionMocks = vi.hoisted(() => ({
	saveOfflineSession: vi.fn(async (record: OfflineSessionRecord) => {
		store.saved.push(structuredClone(record));
	}),
	deleteOfflineSession: vi.fn(async (key: string) => {
		store.deleted.push(key);
	}),
	loadOfflineSession: vi.fn(async () => null),
	listOfflineSessions: vi.fn(async () => store.records),
	sessionKeyForId: (id: number) => `offline.today.session.${id}`
}));

vi.mock('$lib/offline/todaySessions', () => sessionMocks);

import { refreshPendingSyncCount, syncOne, type SyncApi } from '$lib/today/controller/offline';

function makeExercise(id: number, name: string) {
	return {
		id,
		name,
		startedAt: '2026-06-19T10:00:00.000Z',
		endedAt: '2026-06-19T10:05:00.000Z',
		notes: '',
		perSideWeight: false,
		splitWeight: false,
		settings: [],
		tracksReps: true,
		tracksTime: false,
		tracksWeight: true,
		status: 'done',
		sets: [{ id: -1, reps: 5, weight: 100 }]
	};
}

// A server-started session: positive ids, server_workout_id set, edited offline.
function serverStartedRecord(): OfflineSessionRecord {
	return {
		key: 'offline.today.session.42',
		status: 'in_progress',
		updated_at: '2026-06-19T10:00:00.000Z',
		session: {
			id: 42,
			startedAt: '2026-06-19T10:00:00.000Z',
			endedAt: '2026-06-19T10:10:00.000Z',
			notes: '',
			timezoneOffsetMinutes: 0,
			exercises: [makeExercise(7, 'Bench')]
		} as unknown as UiSession,
		server_workout_id: 42,
		server_exercise_ids_by_local: {},
		deleted_server_exercise_ids: []
	};
}

function makeApi(overrides: Partial<SyncApi> = {}): SyncApi {
	return {
		cancelExercise: vi.fn(async () => undefined),
		cancelWorkout: vi.fn(async () => undefined),
		createExercise: vi.fn(async () => ({ id: 1 })),
		createWorkout: vi.fn(async () => ({ id: 999 })),
		endExercise: vi.fn(async () => undefined),
		endWorkout: vi.fn(async () => undefined),
		replaceSets: vi.fn(async () => []),
		...overrides
	};
}

describe('offline edits to a server-started session (audit #3)', () => {
	beforeEach(() => {
		store.records = [];
		store.saved = [];
		store.deleted = [];
		vi.clearAllMocks();
	});

	it('counts a server-started in-progress record as pending', async () => {
		store.records = [serverStartedRecord()];
		const pendingSyncCount = writable(0);

		const count = await refreshPendingSyncCount({ pendingSyncCount });

		expect(count).toBe(1);
	});

	it('syncOne pushes edits then deletes the server-started record (no perpetual pending)', async () => {
		const api = makeApi();
		const record = serverStartedRecord();

		await syncOne(record, api);

		// Edits pushed to the existing server workout, not a new one.
		expect(api.createWorkout).not.toHaveBeenCalled();
		expect(api.replaceSets).toHaveBeenCalledTimes(1);
		// Record removed so it is not counted as pending forever.
		expect(store.deleted).toContain(record.key);
	});
});
