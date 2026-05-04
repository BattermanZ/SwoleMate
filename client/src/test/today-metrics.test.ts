import { describe, expect, it } from 'vitest';
import {
	calculateTotalDurationSeconds,
	calculateTotalVolumeKg
} from '$lib/today/controller/metrics';
import type { UiSession } from '$lib/today/types';

function sessionWithSets(sets: UiSession['exercises'][number]['sets']): UiSession {
	return {
		id: 1,
		startedAt: '2026-01-01T10:00:00.000Z',
		notes: '',
		exercises: [
			{
				id: 2,
				name: 'Plank',
				notes: '',
				startedAt: '2026-01-01T10:00:00.000Z',
				endedAt: '2026-01-01T10:05:00.000Z',
				status: 'active',
				perSideWeight: false,
				splitWeight: false,
				settings: [],
				sets
			}
		]
	};
}

describe('today metrics', () => {
	it('tracks timed set duration separately from volume', () => {
		const session = sessionWithSets([
			{ id: 1, reps: 0, weight: 0, durationSeconds: 75 },
			{ id: 2, reps: 0, weight: 10, durationSeconds: 60 }
		]);

		expect(calculateTotalDurationSeconds(session)).toBe(135);
		expect(calculateTotalVolumeKg(session)).toBe(0);
	});

	it('keeps volume for rep and weight work while preserving timed duration', () => {
		const session = sessionWithSets([{ id: 1, reps: 8, weight: 40, durationSeconds: 45 }]);

		expect(calculateTotalVolumeKg(session)).toBe(320);
		expect(calculateTotalDurationSeconds(session)).toBe(45);
	});
});
