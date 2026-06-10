import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';

describe('today mark-done scrolling', () => {
	it('waits for the completed exercise to collapse before scrolling to the composer', () => {
		const source = readFileSync(join(process.cwd(), 'src/routes/+page.svelte'), 'utf8');

		expect(source).toContain("import { onMount, tick } from 'svelte';");
		expect(source).toMatch(/async function markDoneAndScroll\(exerciseId: number\)/);
		expect(source).toMatch(/await c\.markExerciseDone\(exerciseId\);[\s\S]*await tick\(\);/);
		expect(source).toMatch(/await tick\(\);[\s\S]*composerEl\?\.scrollIntoView/);
	});
});
