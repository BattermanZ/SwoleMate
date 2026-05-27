import { render } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import MasterDetailFull from './fixtures/MasterDetailFull.svelte';
import MasterDetailNoDetail from './fixtures/MasterDetailNoDetail.svelte';

describe('MasterDetail', () => {
	it('renders the list snippet in the list pane', () => {
		const { getByTestId } = render(MasterDetailFull);
		const list = getByTestId('list-content');
		expect(list).toBeInTheDocument();
		expect(list.closest('.list')).not.toBeNull();
	});

	it('renders the detail snippet in the detail pane when provided', () => {
		const { getByTestId, queryByTestId } = render(MasterDetailFull);
		const detail = getByTestId('detail-content');
		expect(detail).toBeInTheDocument();
		expect(detail.closest('.detail')).not.toBeNull();
		expect(queryByTestId('empty-content')).toBeNull();
	});

	it('renders the empty snippet when no detail snippet is provided', () => {
		const { getByTestId, queryByTestId } = render(MasterDetailNoDetail);
		expect(getByTestId('empty-content')).toBeInTheDocument();
		expect(queryByTestId('detail-content')).toBeNull();
	});
});
