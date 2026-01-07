import { render } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import SetPillsHybrid from '$lib/components/ui/SetPillsHybrid.svelte';

describe('SetPillsHybrid', () => {
	it('groups repeated sets and renders count + reps + weight segments', () => {
		const { getByText, queryAllByText } = render(SetPillsHybrid, {
			props: {
				sets: [
					{ reps: 12, weight: 62 },
					{ reps: 12, weight: 62 },
					{ reps: 10, weight: 60 }
				],
				perSideWeight: false,
				splitWeight: false,
				size: 'xs'
			}
		});

		expect(getByText('2×')).toBeInTheDocument();
		expect(getByText('12×')).toBeInTheDocument();
		expect(getByText('62kg')).toBeInTheDocument();

		expect(getByText('10×')).toBeInTheDocument();
		expect(getByText('60kg')).toBeInTheDocument();

		expect(queryAllByText('2×')).toHaveLength(1);
	});

	it('labels split per-side weights as L/R kg', () => {
		const { getByText } = render(SetPillsHybrid, {
			props: {
				sets: [{ reps: 8, weight: 20, weightLeft: 17.5, weightRight: 20 }],
				perSideWeight: true,
				splitWeight: true,
				size: 'xs'
			}
		});

		expect(getByText('8×')).toBeInTheDocument();
		expect(getByText('17.5/20kg')).toBeInTheDocument();
	});
});
