import { describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import type { UiSession } from '$lib/today/types';
import { createTodayController } from '$lib/today/controller';
import { createExercise } from '$lib/api';

vi.mock('$lib/api', () => ({
	cancelExercise: vi.fn(),
	cancelWorkout: vi.fn(),
	createExercise: vi.fn(async () => ({ id: 123 })),
	createSet: vi.fn(),
	createWorkout: vi.fn(),
	endExercise: vi.fn(),
	endWorkout: vi.fn(),
	getExerciseTypes: vi.fn(),
	getWorkout: vi.fn(),
	getWorkouts: vi.fn(),
	replaceSets: vi.fn()
}));

describe('today controller carry-over', () => {
	it('carries over settings and weight modes when adding exercise', async () => {
		const controller = createTodayController();

		const session: UiSession = {
			id: 99,
			startedAt: '2026-01-02T10:00:00.000Z',
			notes: '',
			exercises: []
		};

		controller.currentSession.set(session);
		controller.recentSessions.set([
			{
				id: 1,
				startedAt: '2025-12-20T10:00:00.000Z',
				endedAt: '2025-12-20T11:00:00.000Z',
				notes: '',
				exercises: [
					{
						id: 10,
						name: 'Bench Press',
						notes: '',
						startedAt: '2025-12-20T10:00:00.000Z',
						endedAt: '2025-12-20T10:30:00.000Z',
						status: 'done',
						perSideWeight: true,
						splitWeight: false,
						sets: [{ id: 20, reps: 8, weight: 30 }],
						settings: [
							{ id: 's1', key: 'Bench', value: 'Flat' },
							{ id: 's2', key: 'Rack height', value: '6' }
						]
					}
				]
			}
		]);

		await controller.addExercise('Bench Press');

		expect(createExercise).toHaveBeenCalledWith(
			99,
			expect.objectContaining({
				exercise_type: 'Bench Press',
				per_side_weight: true,
				split_weight: false,
				settings: [
					{ key: 'Bench', value: 'Flat' },
					{ key: 'Rack height', value: '6' }
				]
			})
		);

		const updated = get(controller.currentSession);
		expect(updated?.exercises).toHaveLength(1);
		expect(updated?.exercises[0]?.perSideWeight).toBe(true);
		expect(updated?.exercises[0]?.splitWeight).toBe(false);
		expect(updated?.exercises[0]?.settings.map((s) => `${s.key}:${s.value}`)).toEqual([
			'Bench:Flat',
			'Rack height:6'
		]);
	});
});
