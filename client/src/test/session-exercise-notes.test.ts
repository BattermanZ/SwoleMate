import { render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import SessionExercise from '$lib/components/today/SessionExercise.svelte';
import type { UiExercise } from '$lib/today/types';

const exercise: UiExercise = {
	id: 10,
	name: 'Bench Press',
	notes: '',
	startedAt: '2026-01-02T10:00:00.000Z',
	endedAt: '2026-01-02T10:00:00.000Z',
	sets: [],
	settings: [],
	tracksReps: true,
	tracksTime: false,
	tracksWeight: true,
	perSideWeight: false,
	splitWeight: false,
	status: 'active'
};

describe('session exercise notes', () => {
	it('prefills the add-set controls from the first set last time', () => {
		render(SessionExercise, {
			exercise,
			isOpen: true,
			lastTime: {
				startedAt: '2025-12-20T10:00:00.000Z',
				notes: '',
				sets: [{ reps: 8, weight: 60 }],
				perSideWeight: false,
				splitWeight: false
			},
			onToggle: vi.fn()
		});

		expect(screen.getByRole('button', { name: 'Edit Reps' })).toHaveTextContent('8');
		expect(screen.getByRole('button', { name: 'Edit Weight' })).toHaveTextContent('60');
	});

	it('shows previous session notes in the exercise card without filling today notes', () => {
		render(SessionExercise, {
			exercise,
			isOpen: true,
			lastTime: {
				startedAt: '2025-12-20T10:00:00.000Z',
				notes: 'Keep elbows tucked',
				sets: [{ reps: 8, weight: 60 }],
				perSideWeight: false,
				splitWeight: false
			},
			onToggle: vi.fn()
		});

		expect(screen.getByText('Last session notes')).toBeInTheDocument();
		expect(screen.getByText('Keep elbows tucked')).toBeInTheDocument();
		expect(screen.getByPlaceholderText('Cues, tempo, how it felt…')).toHaveValue('');
	});
});
