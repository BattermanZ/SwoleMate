import { describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import { createTodayController } from '$lib/today/controller';
import { createExercise } from '$lib/api';
import { loadOfflineSession, sessionKeyForId } from '$lib/offline/todaySessions';

vi.mock('$lib/api', () => ({
	cancelExercise: vi.fn(),
	cancelWorkout: vi.fn(),
	createExercise: vi.fn(),
	createSet: vi.fn(),
	createWorkout: vi.fn(async () => {
		throw new TypeError('Failed to fetch');
	}),
	endExercise: vi.fn(),
	endWorkout: vi.fn(),
	getExerciseTypes: vi.fn(async () => []),
	getWorkout: vi.fn(),
	getWorkouts: vi.fn(async () => {
		throw new TypeError('Failed to fetch');
	}),
	replaceSets: vi.fn()
}));

describe('offline today flow', () => {
	it('starts and persists a local session when offline', async () => {
		localStorage.clear();
		// A real offline session happens while logged in, so an active user scope
		// exists for persistence (F-LOW-2 guard).
		localStorage.setItem('auth.activeUserId', '1');
		const controller = createTodayController();

		await controller.startSession('empty');

		const session = get(controller.currentSession);
		expect(session).toBeTruthy();
		expect(session!.id).toBeLessThan(0);

		const record = await loadOfflineSession(sessionKeyForId(session!.id));
		expect(record?.status).toBe('in_progress');
		expect(record?.session.id).toBe(session!.id);
		expect(get(controller.notice)).toBeTruthy();
	});

	it('adds exercises locally without calling the API', async () => {
		localStorage.clear();
		// A real offline session happens while logged in, so an active user scope
		// exists for persistence (F-LOW-2 guard).
		localStorage.setItem('auth.activeUserId', '1');
		const controller = createTodayController();

		await controller.startSession('empty');
		await controller.addExercise('Bench Press');

		expect(createExercise).not.toHaveBeenCalled();
		const session = get(controller.currentSession);
		expect(session?.exercises).toHaveLength(1);
		expect(session?.exercises[0]?.id).toBeLessThan(0);
	});

	it('collapses only the completed exercise card when marked done', async () => {
		localStorage.clear();
		// A real offline session happens while logged in, so an active user scope
		// exists for persistence (F-LOW-2 guard).
		localStorage.setItem('auth.activeUserId', '1');
		const controller = createTodayController();

		await controller.startSession('empty');
		await controller.addExercise('Bench Press');
		await controller.addExercise('Cable Row');

		const session = get(controller.currentSession);
		const exerciseId = session?.exercises[0]?.id;
		const nextExerciseId = session?.exercises[1]?.id;
		expect(exerciseId).toBeTruthy();
		expect(nextExerciseId).toBeTruthy();
		expect(get(controller.openExerciseIds)).toEqual([exerciseId, nextExerciseId]);

		await controller.markExerciseDone(exerciseId!);

		expect(get(controller.openExerciseIds)).toEqual([nextExerciseId]);
	});
});
