import { render } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import DesktopTopBarHarness from './fixtures/DesktopTopBarHarness.svelte';

describe('DesktopTopBar', () => {
	it('renders the title', () => {
		const { getByText } = render(DesktopTopBarHarness, { title: 'Plans' });
		expect(getByText('Plans')).toBeInTheDocument();
	});

	it('renders the subtitle when provided', () => {
		const { getByText } = render(DesktopTopBarHarness, {
			title: 'Plans',
			subtitle: 'your templates'
		});
		expect(getByText('your templates')).toBeInTheDocument();
	});

	it('renders the actions snippet when provided', () => {
		const { getByTestId } = render(DesktopTopBarHarness, { title: 'Plans', withActions: true });
		expect(getByTestId('action-btn')).toBeInTheDocument();
	});

	it('omits the actions region when no actions snippet is provided', () => {
		const { container } = render(DesktopTopBarHarness, { title: 'Plans' });
		expect(container.querySelector('.actions')).toBeNull();
	});
});
