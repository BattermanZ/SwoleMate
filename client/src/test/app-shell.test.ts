import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';

describe('app shell mobile viewport', () => {
	it('disables page zoom gestures in the viewport meta tag', () => {
		const source = readFileSync(join(process.cwd(), 'src/app.html'), 'utf8');

		expect(source).toContain('maximum-scale=1');
		expect(source).toContain('user-scalable=no');
	});

	it('uses touch-action manipulation at the page root', () => {
		const source = readFileSync(join(process.cwd(), 'src/app.css'), 'utf8');

		expect(source).toMatch(/html,\s*body\s*\{[^}]*touch-action:\s*manipulation;/s);
	});
});
