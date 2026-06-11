import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';

describe('today estimated 1RM PR wiring', () => {
	it('passes real estimated 1RM baselines into exercise cards instead of using a heaviest-set heuristic', () => {
		const route = readFileSync(join(process.cwd(), 'src/routes/+page.svelte'), 'utf8');
		const exercise = readFileSync(
			join(process.cwd(), 'src/lib/components/today/SessionExercise.svelte'),
			'utf8'
		);

		expect(route).toContain('estimated1RmBaselines');
		expect(route).toContain('loadEstimated1RmBaseline(ex.name)');
		expect(route).toContain('estimated1RmBaseline={$estimated1RmBaselines[ex.name]}');
		expect(exercise).toContain('estimatedOneRepMaxPrGroupIndex');
		expect(exercise).not.toContain('crude heuristic');
	});

	it('renders PR stars with a high-contrast badge treatment', () => {
		const spill = readFileSync(join(process.cwd(), 'src/lib/components/ui/Spill.svelte'), 'utf8');

		expect(spill).toMatch(
			/\.spill\.pr \.weight::after\s*\{[^}]*background:\s*var\(--surface-deep\);/s
		);
		expect(spill).toMatch(/\.spill\.pr \.weight::after\s*\{[^}]*color:\s*var\(--gold\);/s);
	});
});
