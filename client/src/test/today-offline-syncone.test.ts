import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { OfflineSessionRecord } from '$lib/offline/todaySessions';
import type { UiSession } from '$lib/today/types';

const saved = vi.hoisted(() => ({ records: [] as OfflineSessionRecord[] }));

const sessionMocks = vi.hoisted(() => ({
	saveOfflineSession: vi.fn(async (record: OfflineSessionRecord) => {
		saved.records.push(structuredClone(record));
	}),
	deleteOfflineSession: vi.fn(async () => undefined),
	loadOfflineSession: vi.fn(async () => null),
	listOfflineSessions: vi.fn(async () => []),
	sessionKeyForId: (id: number) => `offline.today.session.${id}`
}));

vi.mock('$lib/offline/todaySessions', () => sessionMocks);

import { syncOne, type SyncApi } from '$lib/today/controller/offline';

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

function makeRecord(): OfflineSessionRecord {
	return {
		key: 'offline.today.session.-100',
		status: 'in_progress',
		updated_at: '2026-06-19T10:00:00.000Z',
		session: {
			id: -100,
			startedAt: '2026-06-19T10:00:00.000Z',
			endedAt: '2026-06-19T10:10:00.000Z',
			notes: '',
			timezoneOffsetMinutes: 0,
			exercises: [makeExercise(-1, 'Bench'), makeExercise(-2, 'Squat')]
		} as unknown as UiSession,
		server_exercise_ids_by_local: {},
		deleted_server_exercise_ids: []
	};
}

function makeApi(overrides: Partial<SyncApi> = {}): SyncApi {
	return {
		cancelExercise: vi.fn(async () => undefined),
		cancelWorkout: vi.fn(async () => undefined),
		createExercise: vi.fn(async () => ({ id: Math.floor(Math.random() * 1000) + 1 })),
		createWorkout: vi.fn(async () => ({ id: 555 })),
		endExercise: vi.fn(async () => undefined),
		endWorkout: vi.fn(async () => undefined),
		replaceSets: vi.fn(async () => []),
		...overrides
	};
}

describe('syncOne idempotency across partial failures', () => {
	beforeEach(() => {
		saved.records = [];
		vi.clearAllMocks();
	});

	it('persists server_workout_id and exercise map before a mid-loop failure', async () => {
		let calls = 0;
		const createExercise = vi.fn(async () => {
			calls += 1;
			if (calls === 1) return { id: 11 };
			throw new TypeError('Failed to fetch');
		});
		const api = makeApi({ createExercise });

		await expect(syncOne(makeRecord(), api)).rejects.toThrow();

		// The workout id and the first exercise's mapping must be persisted so a
		// retry resumes rather than re-creating the workout.
		const last = saved.records.at(-1);
		expect(last?.server_workout_id).toBe(555);
		expect(last?.server_exercise_ids_by_local).toEqual({ '-1': 11 });
	});

	it('sends stable idempotency keys derived from local ids (F-HIGH-3)', async () => {
		const createWorkout = vi.fn(async () => ({ id: 555 }));
		const createExercise = vi.fn(async () => ({ id: Math.floor(Math.random() * 1000) + 1 }));
		const api = makeApi({ createWorkout, createExercise });

		await syncOne(makeRecord(), api);

		// Workout key is derived from the session's negative local id, so a retry
		// carries the same key and the server dedups it.
		expect(createWorkout).toHaveBeenCalledWith(expect.anything(), 'w:-100');
		// Each exercise create carries a key derived from its own local id.
		expect(createExercise).toHaveBeenCalledWith(555, expect.anything(), 'e:-1');
		expect(createExercise).toHaveBeenCalledWith(555, expect.anything(), 'e:-2');
	});

	it('retry after partial failure does not re-create the workout or done exercises', async () => {
		// Simulate the persisted state after the first run created workout 555 and
		// mapped the first exercise.
		const record = makeRecord();
		record.server_workout_id = 555;
		record.server_exercise_ids_by_local = { '-1': 11 };

		const api = makeApi();
		await syncOne(record, api);

		expect(api.createWorkout).not.toHaveBeenCalled();
		// Only the still-unmapped second exercise should be created.
		expect(api.createExercise).toHaveBeenCalledTimes(1);
	});
});
