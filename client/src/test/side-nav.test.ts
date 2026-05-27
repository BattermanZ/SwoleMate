import { render } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import SideNavHarness from './fixtures/SideNavHarness.svelte';

describe('SideNav', () => {
	it('renders one link per nav item with its label', () => {
		const { getByRole } = render(SideNavHarness, { current: '/' });
		expect(getByRole('link', { name: /Today/ })).toBeInTheDocument();
		expect(getByRole('link', { name: /Plans/ })).toBeInTheDocument();
		expect(getByRole('link', { name: /Progress/ })).toBeInTheDocument();
	});

	it('marks the current route active via aria-current', () => {
		const { getByRole } = render(SideNavHarness, { current: '/progress' });
		const active = getByRole('link', { name: /Progress/ });
		expect(active).toHaveAttribute('aria-current', 'page');
		expect(getByRole('link', { name: /Today/ })).not.toHaveAttribute('aria-current');
	});

	it('renders a logout button that fires the callback when onLogout is provided', async () => {
		const onLogout = vi.fn();
		const { getByRole } = render(SideNavHarness, { current: '/', onLogout });
		getByRole('button', { name: /Log out/i }).click();
		expect(onLogout).toHaveBeenCalledOnce();
	});

	it('omits the logout button when onLogout is not provided', () => {
		const { queryByRole } = render(SideNavHarness, { current: '/' });
		expect(queryByRole('button', { name: /Log out/i })).toBeNull();
	});
});
