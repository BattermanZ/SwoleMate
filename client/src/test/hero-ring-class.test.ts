import { render } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import ProgressHero from '$lib/components/progress/ProgressHero.svelte';
import SessionHero from '$lib/components/today/SessionHero.svelte';

describe('hero ring wrappers', () => {
	it('avoids Tailwind ring utility collisions in the progress hero', () => {
		const { container } = render(ProgressHero, {
			consistencyDone: 11,
			consistencyWindow: 30,
			totalWorkouts: 42,
			perWeek: 4.2,
			avgDurationMin: 52
		});

		expect(container.querySelector('.progress-ring')).toBeInTheDocument();
		expect(container.querySelector('.ring')).not.toBeInTheDocument();
	});

	it('avoids Tailwind ring utility collisions in the session hero', () => {
		const { container } = render(SessionHero, {
			elapsedLabel: '11:30',
			exerciseCount: 3,
			setCount: 8,
			volumeKg: 1200,
			durationSeconds: 690
		});

		expect(container.querySelector('.session-ring')).toBeInTheDocument();
		expect(container.querySelector('.ring')).not.toBeInTheDocument();
	});
});
