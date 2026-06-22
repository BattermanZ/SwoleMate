import { describe, expect, it } from 'vitest';

import { makeLocalNumericId } from '$lib/today/controller/utils';

describe('makeLocalNumericId', () => {
	it('produces safe, negative integers', () => {
		for (let i = 0; i < 1000; i++) {
			const id = makeLocalNumericId();
			expect(Number.isSafeInteger(id)).toBe(true);
			expect(id).toBeLessThan(0);
		}
	});

	it('never collides across rapid successive calls', () => {
		const seen = new Set<number>();
		for (let i = 0; i < 200_000; i++) {
			const id = makeLocalNumericId();
			expect(seen.has(id)).toBe(false);
			seen.add(id);
		}
		expect(seen.size).toBe(200_000);
	});

	it('is strictly monotonically decreasing', () => {
		let prev = makeLocalNumericId();
		for (let i = 0; i < 1000; i++) {
			const id = makeLocalNumericId();
			expect(id).toBeLessThan(prev);
			prev = id;
		}
	});
});
