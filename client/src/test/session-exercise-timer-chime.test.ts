import { fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import SessionExercise from '$lib/components/today/SessionExercise.svelte';
import type { UiExercise } from '$lib/today/types';
import { playTimerChime, unlockTimerChime } from '$lib/audio/timerChime';

vi.mock('$lib/audio/timerChime', () => ({
	playTimerChime: vi.fn(),
	unlockTimerChime: vi.fn()
}));

const timedExercise: UiExercise = {
	id: 10,
	name: 'Plank',
	notes: '',
	startedAt: '2026-01-02T10:00:00.000Z',
	endedAt: '2026-01-02T10:00:00.000Z',
	sets: [],
	settings: [],
	tracksReps: false,
	tracksTime: true,
	tracksWeight: false,
	perSideWeight: false,
	splitWeight: false,
	status: 'active'
};

describe('session exercise timer chime', () => {
	beforeEach(() => {
		vi.useFakeTimers();
		vi.clearAllMocks();
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	it('unlocks the chime when the countdown is started', async () => {
		render(SessionExercise, { exercise: timedExercise, isOpen: true, onToggle: vi.fn() });

		await fireEvent.click(screen.getByRole('button', { name: 'Start countdown timer' }));

		expect(unlockTimerChime).toHaveBeenCalledTimes(1);
		expect(playTimerChime).not.toHaveBeenCalled();
	});

	it('plays the chime when the countdown reaches zero', async () => {
		render(SessionExercise, { exercise: timedExercise, isOpen: true, onToggle: vi.fn() });

		await fireEvent.click(screen.getByRole('button', { name: 'Start countdown timer' }));

		// Default countdown is 60s; run it out.
		vi.advanceTimersByTime(61_000);

		expect(playTimerChime).toHaveBeenCalledTimes(1);
	});
});
