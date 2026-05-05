import { render } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import SessionExercise from '$lib/components/today/SessionExercise.svelte';
import type { UiExercise } from '$lib/today/types';

describe('SessionExercise', () => {
	it('prefills reps/weight from last session first set', () => {
		const exercise: UiExercise = {
			id: 1,
			name: 'Bench Press',
			notes: '',
			startedAt: '2026-01-01T10:00:00.000Z',
			endedAt: '2026-01-01T10:00:00.000Z',
			sets: [],
			settings: [],
			perSideWeight: false,
			splitWeight: false,
			status: 'active'
		};

		const { getByLabelText } = render(SessionExercise, {
			props: {
				exercise,
				isOpen: true,
				disabled: false,
				lastTime: {
					startedAt: '2025-12-20T10:00:00.000Z',
					notes: '',
					sets: [{ reps: 8, weight: 60 }],
					perSideWeight: false,
					splitWeight: false
				}
			}
		});

		const reps = getByLabelText('Reps') as HTMLInputElement;
		const weight = getByLabelText('Weight (kg)') as HTMLInputElement;

		expect(reps.value).toBe('8');
		expect(weight.value).toBe('60');
	});

	it('prefills target duration from the last timed set', () => {
		const exercise: UiExercise = {
			id: 1,
			name: 'Plank',
			notes: '',
			startedAt: '2026-01-01T10:00:00.000Z',
			endedAt: '2026-01-01T10:00:00.000Z',
			sets: [],
			settings: [],
			tracksReps: false,
			tracksTime: true,
			tracksWeight: false,
			perSideWeight: false,
			splitWeight: false,
			status: 'active'
		};

		const { getByLabelText } = render(SessionExercise, {
			props: {
				exercise,
				isOpen: true,
				disabled: false,
				lastTime: {
					startedAt: '2025-12-20T10:00:00.000Z',
					notes: '',
					sets: [
						{ reps: 0, weight: 0, durationSeconds: 30 },
						{ reps: 0, weight: 0, durationSeconds: 45 }
					],
					perSideWeight: false,
					splitWeight: false
				}
			}
		});

		const duration = getByLabelText('Target duration (sec)') as HTMLInputElement;

		expect(duration.value).toBe('45');
	});
});
