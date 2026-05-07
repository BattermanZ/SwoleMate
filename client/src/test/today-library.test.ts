import { describe, expect, it } from 'vitest';
import { mergeExerciseLibrary } from '$lib/today/controller/actions/library';

describe('today exercise library', () => {
	it('deduplicates built-in and history exercise names case-insensitively', () => {
		const merged = mergeExerciseLibrary(
			['Bench Press', 'Chest Press', 'Cable Row'],
			['Chest press', 'Seated Row']
		);

		expect(merged).toContain('Chest press');
		expect(merged).not.toContain('Chest Press');
		expect(merged).toEqual(['Bench Press', 'Cable Row', 'Chest press', 'Seated Row']);
	});
});
