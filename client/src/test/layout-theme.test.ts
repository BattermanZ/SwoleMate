import { fireEvent, render } from '@testing-library/svelte';
import { readable } from 'svelte/store';
import { describe, expect, it, vi } from 'vitest';

vi.mock('$app/stores', () => {
	return {
		page: readable({
			url: new URL('http://localhost/')
		})
	};
});

vi.mock('$lib/logger', () => {
	return {
		logger: {
			setRemoteEnabled: vi.fn(),
			debug: vi.fn(),
			info: vi.fn(),
			error: vi.fn()
		}
	};
});

describe('Layout theme toggle', () => {
	it('toggles the .dark class and persists to localStorage', async () => {
		localStorage.removeItem('theme');
		document.documentElement.classList.remove('dark');

		const { default: Layout } = await import('../routes/+layout.svelte');

		const { getByRole } = render(Layout);

		const toggle = getByRole('button', { name: /toggle dark mode/i });
		expect(document.documentElement.classList.contains('dark')).toBe(false);

		await fireEvent.click(toggle);
		expect(document.documentElement.classList.contains('dark')).toBe(true);
		expect(localStorage.getItem('theme')).toBe('dark');

		await fireEvent.click(toggle);
		expect(document.documentElement.classList.contains('dark')).toBe(false);
		expect(localStorage.getItem('theme')).toBe('light');
	}, 30_000);
});
