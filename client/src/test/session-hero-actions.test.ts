import { fireEvent, render, screen } from '@testing-library/svelte';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import type { ComponentProps } from 'svelte';
import { describe, expect, it, vi } from 'vitest';
import SessionHero from '$lib/components/today/SessionHero.svelte';

type SessionHeroProps = ComponentProps<typeof SessionHero>;

function renderSessionHero(overrides: Partial<SessionHeroProps> = {}) {
	return render(SessionHero, {
		elapsedLabel: '11:30',
		exerciseCount: 3,
		setCount: 8,
		volumeKg: 1200,
		durationSeconds: 690,
		...overrides
	});
}

describe('session hero actions', () => {
	it('fires cancel and end session handlers from their buttons', async () => {
		const onCancel = vi.fn();
		const onEnd = vi.fn();
		renderSessionHero({ onCancel, onEnd });

		await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
		await fireEvent.click(screen.getByRole('button', { name: 'End session →' }));

		expect(onCancel).toHaveBeenCalledTimes(1);
		expect(onEnd).toHaveBeenCalledTimes(1);
	});

	it('keeps decorative hero glows out of the pointer hit-test layer', () => {
		const source = readFileSync(
			join(process.cwd(), 'src/lib/components/ui/PageHero.svelte'),
			'utf8'
		);

		expect(source).toMatch(/\.hero::before\s*\{[^}]*pointer-events:\s*none;/s);
		expect(source).toMatch(/\.hero::after\s*\{[^}]*pointer-events:\s*none;/s);
	});
});
