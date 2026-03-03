import { fireEvent, render } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import ExerciseComposer from '$lib/components/today/ExerciseComposer.svelte';
import EndSessionModal from '$lib/components/today/EndSessionModal.svelte';

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

		expect(onAdd).toHaveBeenNthCalledWith(1, expect.objectContaining({ detail: { name: 'Bench Press' } }));
		expect(onAdd).toHaveBeenNthCalledWith(2, expect.objectContaining({ detail: { name: 'Bench Press' } }));
		expect(onAdd).toHaveBeenNthCalledWith(3, expect.objectContaining({ detail: { name: 'Squat' } }));
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
});
