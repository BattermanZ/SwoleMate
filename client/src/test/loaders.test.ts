import { beforeEach, describe, expect, it, vi } from 'vitest';

const apiMocks = vi.hoisted(() => ({
	getWorkouts: vi.fn(),
	getBackups: vi.fn(),
	getWorkout: vi.fn()
}));

const loggerMocks = vi.hoisted(() => ({
	error: vi.fn()
}));

vi.mock('$lib/api', () => ({
	getWorkouts: apiMocks.getWorkouts,
	getBackups: apiMocks.getBackups,
	getWorkout: apiMocks.getWorkout
}));

vi.mock('$lib/logger', () => ({
	logger: loggerMocks
}));

describe('route loaders', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('loads workouts list and falls back to empty array on error', async () => {
		const { load } = await import('../routes/workouts/+page');
		apiMocks.getWorkouts.mockResolvedValueOnce([{ id: 1 }]);
		expect(await load({ fetch: vi.fn() } as never)).toEqual({ workouts: [{ id: 1 }] });

		apiMocks.getWorkouts.mockRejectedValueOnce(new Error('down'));
		expect(await load({ fetch: vi.fn() } as never)).toEqual({ workouts: [] });
		expect(loggerMocks.error).toHaveBeenCalled();
	});

	it('loads backups list and falls back to empty array on error', async () => {
		const { load } = await import('../routes/backups/+page');
		apiMocks.getBackups.mockResolvedValueOnce([{ filename: 'a' }]);
		expect(await load({ fetch: vi.fn() } as never)).toEqual({ backups: [{ filename: 'a' }] });

		apiMocks.getBackups.mockRejectedValueOnce(new Error('down'));
		expect(await load({ fetch: vi.fn() } as never)).toEqual({ backups: [] });
		expect(loggerMocks.error).toHaveBeenCalled();
	});

	it('loads workout detail and handles invalid id and errors', async () => {
		const { load } = await import('../routes/workouts/[id]/+page');

		expect(await load({ fetch: vi.fn(), params: { id: 'abc' } } as never)).toEqual({
			workout: null,
			error: 'Invalid workout ID'
		});

		apiMocks.getWorkout.mockResolvedValueOnce({
			workout: { id: 9 },
			exercises: []
		});
		expect(await load({ fetch: vi.fn(), params: { id: '9' } } as never)).toEqual({
			workout: { id: 9, exercises: [] },
			error: null
		});

		apiMocks.getWorkout.mockRejectedValueOnce(new Error('boom'));
		expect(await load({ fetch: vi.fn(), params: { id: '9' } } as never)).toEqual({
			workout: null,
			error: 'boom'
		});
		expect(loggerMocks.error).toHaveBeenCalled();
	});
});
