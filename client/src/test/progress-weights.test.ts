import { describe, expect, it } from 'vitest';
import { getEffectiveWeight, getSetVolume } from '$lib/progress/weights';
import type { Exercise, Set } from '$lib/types';

function exercise(overrides: Partial<Exercise> = {}): Exercise {
	return {
		id: 1,
		workout_id: 1,
		exercise_type: 'Bench Press',
		start_time: '2026-01-01T10:00:00.000Z',
		end_time: '2026-01-01T10:10:00.000Z',
		per_side_weight: false,
		split_weight: false,
		settings: [],
		...overrides
	};
}

function set(overrides: Partial<Set> = {}): Set {
	return {
		id: 1,
		exercise_id: 1,
		reps: 8,
		weight: 40,
		notes: null,
		...overrides
	};
}

describe('progress weight helpers', () => {
	it('uses the set weight for regular exercises', () => {
		expect(getEffectiveWeight(set({ weight: 60 }), exercise())).toBe(60);
		expect(getSetVolume(set({ reps: 5, weight: 60 }), exercise())).toBe(300);
	});

	it('doubles the base weight for per-side exercises without split weights', () => {
		const ex = exercise({ per_side_weight: true, split_weight: false });

		expect(getEffectiveWeight(set({ weight: 22.5 }), ex)).toBe(45);
		expect(getSetVolume(set({ reps: 10, weight: 22.5 }), ex)).toBe(450);
	});

	it('sums left and right weights for split per-side exercises', () => {
		const ex = exercise({ per_side_weight: true, split_weight: true });

		expect(getEffectiveWeight(set({ weight: 20, weight_left: 18, weight_right: 22 }), ex)).toBe(40);
		expect(getSetVolume(set({ reps: 8, weight: 20, weight_left: 18, weight_right: 22 }), ex)).toBe(
			320
		);
	});

	it('falls back to doubled base weight when split data is incomplete', () => {
		const ex = exercise({ per_side_weight: true, split_weight: true });

		expect(
			getEffectiveWeight(set({ weight: 20, weight_left: 18, weight_right: undefined }), ex)
		).toBe(40);
	});
});
