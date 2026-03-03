import { fireEvent, render } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import EditWorkoutTimesModal from '$lib/components/history/EditWorkoutTimesModal.svelte';
import WorkoutDetailsCard from '$lib/components/history/WorkoutDetailsCard.svelte';

describe('history components', () => {
	it('validates times and emits submit from edit modal', async () => {
		const workout = {
			id: 1,
			date: '2026-01-01T10:00:00.000Z',
			start_time: '2026-01-01T10:00:00.000Z',
			end_time: '2026-01-01T11:00:00.000Z',
			notes: 'note',
			feedback: '😊'
		};

		const onSubmit = vi.fn();
		const { getByText, getAllByDisplayValue } = render(EditWorkoutTimesModal, {
			props: { open: true, workout: workout as never, disabled: false, error: null },
			events: { submit: onSubmit }
		});

		const [startInput, endInput] = getAllByDisplayValue(/2026-01-01T/);
		await fireEvent.input(startInput, { target: { value: '2026-01-01T12:00' } });
		await fireEvent.input(endInput, { target: { value: '2026-01-01T11:00' } });
		expect(getByText('End time must be after start time.')).toBeInTheDocument();

		await fireEvent.input(endInput, { target: { value: '2026-01-01T12:30' } });
		await fireEvent.click(getByText('Save'));

		expect(onSubmit).toHaveBeenCalledTimes(1);
		const payload = onSubmit.mock.calls[0]?.[0]?.detail as {
			start_time: string;
			end_time: string;
		};
		expect(payload).toBeTruthy();
		expect(new Date(payload.end_time).getTime()).toBeGreaterThan(new Date(payload.start_time).getTime());
	});

	it('renders summary metrics and exercise details in workout details card', () => {
		const workout = {
			id: 1,
			date: '2026-01-01T10:00:00.000Z',
			start_time: '2026-01-01T10:00:00.000Z',
			end_time: '2026-01-01T11:00:00.000Z',
			notes: 'Felt strong',
			feedback: '😊',
			auto_closed_at: null,
			exercises: [
				{
					exercise: {
						id: 1,
						exercise_type: 'Bench Press',
						notes: 'Pause reps',
						per_side_weight: false,
						split_weight: false,
						settings: [{ id: 1, exercise_id: 1, key: 'Grip', value: 'Medium' }]
					},
					sets: [{ id: 1, exercise_id: 1, reps: 10, weight: 60, notes: null }]
				}
			]
		};

		const { getByText } = render(WorkoutDetailsCard, {
			props: { workout: workout as never, loading: false, error: null }
		});

		expect(getByText('Workout details')).toBeInTheDocument();
		expect(getByText('Exercises')).toBeInTheDocument();
		expect(getByText('Bench Press')).toBeInTheDocument();
		expect(getByText(/Notes:\s*Pause reps/)).toBeInTheDocument();
		expect(getByText('Felt strong')).toBeInTheDocument();
		expect(getByText('600 kg')).toBeInTheDocument();
	});
});
