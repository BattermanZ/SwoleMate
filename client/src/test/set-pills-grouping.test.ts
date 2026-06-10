import { describe, it, expect } from 'vitest';
import { estimatedOneRepMax, estimatedOneRepMaxPrGroupIndex, groupSets } from '$lib/today/setPills';

describe('groupSets', () => {
	it('groups identical sets and increments count', () => {
		const groups = groupSets([
			{ reps: 10, weight: 50 },
			{ reps: 10, weight: 50 },
			{ reps: 10, weight: 50 }
		]);
		expect(groups).toHaveLength(1);
		expect(groups[0].count).toBe(3);
		expect(groups[0].reps).toBe(10);
		expect(groups[0].weightLabel).toBe('50kg');
	});

	it('keeps distinct sets separate', () => {
		const groups = groupSets([
			{ reps: 10, weight: 50 },
			{ reps: 8, weight: 55 }
		]);
		expect(groups).toHaveLength(2);
		expect(groups[0].count).toBe(1);
		expect(groups[1].count).toBe(1);
	});

	it('formats per-side weights', () => {
		const groups = groupSets([{ reps: 8, weight: 22.5 }], {
			perSideWeight: true,
			splitWeight: false
		});
		expect(groups[0].weightLabel).toBe('22.5kg/side');
	});

	it('renders split L/R when values differ', () => {
		const groups = groupSets([{ reps: 8, weight: 27.5, weightLeft: 27.5, weightRight: 22.5 }], {
			perSideWeight: true,
			splitWeight: true
		});
		expect(groups[0].weightLabel).toBe('27.5/22.5kg');
	});

	it('collapses split L=R back to /side notation', () => {
		const groups = groupSets([{ reps: 8, weight: 25, weightLeft: 25, weightRight: 25 }], {
			perSideWeight: true,
			splitWeight: true
		});
		expect(groups[0].weightLabel).toBe('25kg/side');
	});

	it('marks bodyweight sets and omits weight label', () => {
		const groups = groupSets([{ reps: 10, weight: 0 }]);
		expect(groups[0].bodyweight).toBe(true);
		expect(groups[0].weightLabel).toBeUndefined();
	});

	it('renders duration label for timed sets (sub-minute → "30s")', () => {
		const groups = groupSets([
			{ reps: 0, weight: 0, durationSeconds: 30 },
			{ reps: 0, weight: 0, durationSeconds: 30 }
		]);
		expect(groups).toHaveLength(1);
		expect(groups[0].count).toBe(2);
		expect(groups[0].durationLabel).toBe('30s');
		expect(groups[0].weightLabel).toBeUndefined();
	});

	it('formats minute+ durations as "M:SS"', () => {
		const groups = groupSets([{ reps: 0, weight: 0, durationSeconds: 90 }]);
		expect(groups[0].durationLabel).toBe('1:30');
	});

	it('computes intensity ramp from min/max within the group', () => {
		const groups = groupSets([
			{ reps: 10, weight: 100 },
			{ reps: 9, weight: 102.5 },
			{ reps: 8, weight: 105 }
		]);
		expect(groups).toHaveLength(3);
		expect(groups[0].intensity).toBeCloseTo(0.38, 2);
		expect(groups[2].intensity).toBeCloseTo(0.85, 2);
	});

	it('uses a single intensity value when min == max', () => {
		const groups = groupSets([
			{ reps: 10, weight: 100 },
			{ reps: 8, weight: 100 }
		]);
		expect(groups[0].intensity).toBeCloseTo(0.65, 2);
		expect(groups[1].intensity).toBeCloseTo(0.65, 2);
	});

	it('matches the server estimated 1RM formula for eligible weighted sets', () => {
		expect(estimatedOneRepMax({ reps: 8, weight: 100 })).toBe(124.14);
		expect(estimatedOneRepMax({ reps: 13, weight: 100 })).toBeNull();
		expect(estimatedOneRepMax({ reps: 8, weight: 0 })).toBeNull();
	});

	it('returns the grouped pill index for the best estimated 1RM above baseline', () => {
		const sets = [
			{ reps: 10, weight: 90 },
			{ reps: 8, weight: 100 },
			{ reps: 5, weight: 110 }
		];

		expect(estimatedOneRepMaxPrGroupIndex(sets, 123)).toBe(1);
		expect(estimatedOneRepMaxPrGroupIndex(sets, 130)).toBeNull();
		expect(estimatedOneRepMaxPrGroupIndex(sets, null)).toBeNull();
	});

	it('uses effective total weight for split per-side estimated 1RM PRs', () => {
		const sets = [{ reps: 8, weight: 0, weightLeft: 50, weightRight: 52 }];

		expect(
			estimatedOneRepMaxPrGroupIndex(sets, 125, { perSideWeight: true, splitWeight: true })
		).toBe(0);
	});
});
