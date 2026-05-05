import { fireEvent, render, within } from '@testing-library/svelte';
import { tick } from 'svelte';
import { describe, expect, it, vi } from 'vitest';
import ExerciseComposer from '$lib/components/today/ExerciseComposer.svelte';
import EndSessionModal from '$lib/components/today/EndSessionModal.svelte';
import SessionExercise from '$lib/components/today/SessionExercise.svelte';

describe('today components', () => {
	it('adds from enter, suggestion, and quick pick in exercise composer', async () => {
		const onAdd = vi.fn();
		const { getByPlaceholderText, getByText } = render(ExerciseComposer, {
			props: {
				query: ' Bench Press ',
				suggestions: ['Bench Press'],
				quickPicks: ['Squat'],
				disabled: false
			},
			events: { add: onAdd }
		});

		await fireEvent.keyDown(getByPlaceholderText(/Search/), { key: 'Enter' });
		await fireEvent.click(getByText('Bench Press'));
		await fireEvent.click(getByText('Squat'));

		expect(onAdd).toHaveBeenNthCalledWith(
			1,
			expect.objectContaining({ detail: { name: 'Bench Press' } })
		);
		expect(onAdd).toHaveBeenNthCalledWith(
			2,
			expect.objectContaining({ detail: { name: 'Bench Press' } })
		);
		expect(onAdd).toHaveBeenNthCalledWith(
			3,
			expect.objectContaining({ detail: { name: 'Squat' } })
		);
	});

	it('requires mood before submit in end session modal', async () => {
		const onSubmit = vi.fn();
		const { getByText } = render(EndSessionModal, {
			props: { open: true, notes: '', mood: null, disabled: false },
			events: {
				submit: onSubmit
			}
		});

		const submit = getByText('Submit') as HTMLButtonElement;
		expect(submit.disabled).toBe(true);

		await fireEvent.click(getByText('😊'));
		expect(submit.disabled).toBe(false);
		await fireEvent.click(submit);
		expect(onSubmit).toHaveBeenCalledTimes(1);
	});

	it('defaults reps to 12 for a new exercise with no history', () => {
		const { getByLabelText } = render(SessionExercise, {
			props: {
				exercise: {
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
				},
				isOpen: true,
				disabled: false,
				lastTime: undefined
			}
		});

		expect(getByLabelText('Reps')).toHaveValue(12);
	});

	it('uses circular overlay timer for time-only sets without reps', async () => {
		vi.useFakeTimers();
		const onAddSet = vi.fn();
		try {
			const { getByLabelText, getByRole, queryByLabelText } = render(SessionExercise, {
				props: {
					exercise: {
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
					},
					isOpen: true,
					disabled: false,
					lastTime: undefined
				},
				events: { addSet: onAddSet }
			});

			expect(queryByLabelText('Reps')).not.toBeInTheDocument();

			await fireEvent.input(getByLabelText('Target duration (sec)'), { target: { value: '1' } });
			await fireEvent.click(getByRole('button', { name: 'Start timer' }));
			const dialog = getByRole('dialog', { name: 'Plank timer' });
			expect(dialog).toBeInTheDocument();
			expect(within(dialog).getByRole('button', { name: 'Add set' })).toBeDisabled();

			await vi.advanceTimersByTimeAsync(1000);
			await tick();
			expect(within(dialog).getByText('Complete')).toBeInTheDocument();

			await fireEvent.click(within(dialog).getByRole('button', { name: 'Add set' }));
			expect(onAddSet).toHaveBeenCalledWith(
				expect.objectContaining({
					detail: expect.objectContaining({ reps: 0, weight: 0, durationSeconds: 1 })
				})
			);
		} finally {
			vi.useRealTimers();
		}
	});
});
