import { beforeEach, describe, expect, it, vi } from 'vitest';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'content-type': 'application/json' }
	});
}

describe('api client behavior', () => {
	beforeEach(() => {
		vi.resetModules();
	});

	it('throws ApiError and triggers unauthorized handler on 401 json errors', async () => {
		const api = await import('$lib/api');
		const onUnauthorized = vi.fn();
		api.setUnauthorizedHandler(onUnauthorized);

		const fetcher = vi.fn(async () => jsonResponse({ message: 'Unauthorized' }, 401));
		await expect(api.getWorkouts(fetcher as unknown as typeof fetch)).rejects.toMatchObject({
			name: 'ApiError',
			status: 401,
			message: 'Unauthorized'
		});
		expect(onUnauthorized).toHaveBeenCalledTimes(1);
	});

	it('uses text error body when json is unavailable', async () => {
		const { getBackups } = await import('$lib/api');
		const fetcher = vi.fn(
			async () =>
				new Response('backup endpoint down', {
					status: 503,
					headers: { 'content-type': 'text/plain' }
				})
		);

		await expect(getBackups(fetcher as unknown as typeof fetch)).rejects.toMatchObject({
			name: 'ApiError',
			status: 503,
			message: 'backup endpoint down'
		});
	});

	it('returns undefined for 204 responses', async () => {
		const { cancelWorkout } = await import('$lib/api');
		const fetcher = vi.fn(async () => new Response(null, { status: 204 }));

		await expect(cancelWorkout(9, fetcher as unknown as typeof fetch)).resolves.toBeUndefined();
	});

	it('returns undefined for successful non-json and empty-json bodies', async () => {
		const { authLogout } = await import('$lib/api');

		const textFetcher = vi.fn(
			async () =>
				new Response('ok', {
					status: 200,
					headers: { 'content-type': 'text/plain' }
				})
		);
		await expect(authLogout(textFetcher as unknown as typeof fetch)).resolves.toBeUndefined();

		const emptyJsonFetcher = vi.fn(
			async () =>
				new Response('', {
					status: 200,
					headers: { 'content-type': 'application/json' }
				})
		);
		await expect(authLogout(emptyJsonFetcher as unknown as typeof fetch)).resolves.toBeUndefined();
	});

	it('encodes exercise type params and maps tuple response for getLastExerciseData', async () => {
		const { getLastExerciseData, getVolumeStats, getExerciseProgress } = await import('$lib/api');
		const seenUrls: string[] = [];

		const fetcher = vi.fn(async (input: URL | RequestInfo) => {
			seenUrls.push(String(input));
			return jsonResponse([
				{
					id: 4,
					workout_id: 1,
					exercise_type: 'Bench & Press/5',
					start_time: '2026-01-01T10:00:00.000Z',
					end_time: '2026-01-01T10:20:00.000Z'
				},
				[{ id: 7, exercise_id: 4, reps: 5, weight: 80 }]
			]);
		});

		const exerciseType = 'Bench & Press/5';
		const result = await getLastExerciseData(exerciseType, fetcher as unknown as typeof fetch);
		expect(result?.exercise.exercise_type).toBe(exerciseType);
		expect(result?.sets).toHaveLength(1);

		const encoded = encodeURIComponent(exerciseType);
		expect(seenUrls[0]).toContain(`/api/exercises/last/${encoded}`);

		const volumeFetcher = vi.fn(async (input: URL | RequestInfo) => {
			seenUrls.push(String(input));
			return jsonResponse({
				weekly_volume: [],
				monthly_volume: [],
				personal_records: { all_time_max_weight: 0, max_volume: 0, estimated_max_1rm: 0 }
			});
		});
		await getVolumeStats(exerciseType, volumeFetcher as unknown as typeof fetch);
		expect(seenUrls[1]).toContain(`exercise_type=${encoded}`);

		const progressFetcher = vi.fn(async (input: URL | RequestInfo) => {
			seenUrls.push(String(input));
			return jsonResponse([]);
		});
		await getExerciseProgress(exerciseType, progressFetcher as unknown as typeof fetch);
		expect(seenUrls[2]).toContain(`/api/progress/exercise/${encoded}`);
	});
});
