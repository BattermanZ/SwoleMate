import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';

describe('today weight inputs', () => {
	it('increments weight steppers in whole kilograms', () => {
		const source = readFileSync(
			join(process.cwd(), 'src/lib/components/today/SessionExercise.svelte'),
			'utf8'
		);

		expect(source).not.toContain('step={2.5}');
		expect(source).toMatch(/label="Weight"\s+step=\{1\}/);
		expect(source).toMatch(/label="Per side"\s+step=\{1\}/);
		expect(source).toMatch(/label="Left"\s+step=\{1\}/);
		expect(source).toMatch(/label="Right"\s+step=\{1\}/);
	});
});
