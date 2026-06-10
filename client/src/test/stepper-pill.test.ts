import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import StepperPill from '$lib/components/ui/StepperPill.svelte';

describe('StepperPill', () => {
	it('switches the value button to a text input while editing', async () => {
		render(StepperPill, { value: 8, label: 'Reps' });

		await fireEvent.click(screen.getByRole('button', { name: 'Edit Reps' }));

		expect(screen.getByRole('textbox', { name: 'Reps' })).toHaveValue('8');
	});

	it('keeps the editing input at the same visual size as the resting value', () => {
		const source = readFileSync(
			join(process.cwd(), 'src/lib/components/ui/StepperPill.svelte'),
			'utf8'
		);

		expect(source).toMatch(/\.value-input\s*\{[^}]*font-size:\s*22px\s*!important;/s);
	});
});
