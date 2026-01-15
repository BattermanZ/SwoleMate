import { describe, expect, it } from 'vitest';
import { summarizeRepPrs } from '$lib/progress/repPrs';

describe('summarizeRepPrs', () => {
	it('keeps the best weight for each rep count and sorts by reps', () => {
		const summarized = summarizeRepPrs([
			{ reps: 12, weight: 60 },
			{ reps: 12, weight: 62 },
			{ reps: 10, weight: 70 },
			{ reps: 10, weight: 67 }
		]);

		expect(summarized).toEqual([
			{ reps: 10, weight: 70 },
			{ reps: 12, weight: 62 }
		]);
	});

	it('ignores non-finite inputs', () => {
		const summarized = summarizeRepPrs([
			{ reps: 8, weight: 40 },
			{ reps: Number.NaN, weight: 50 },
			{ reps: 8, weight: Number.POSITIVE_INFINITY }
		]);

		expect(summarized).toEqual([{ reps: 8, weight: 40 }]);
	});
});
