import { afterEach, describe, expect, it } from 'vitest';
import { toggleTheme } from '$lib/components/shell/theme';

afterEach(() => {
	document.documentElement.classList.remove('dark');
	document.documentElement.removeAttribute('data-theme');
	localStorage.clear();
});

describe('toggleTheme', () => {
	it('switches from light to dark, setting attribute, class and storage', () => {
		toggleTheme();
		const root = document.documentElement;
		expect(root.getAttribute('data-theme')).toBe('dark');
		expect(root.classList.contains('dark')).toBe(true);
		expect(localStorage.getItem('theme')).toBe('dark');
	});

	it('switches from dark back to light, clearing attribute and class', () => {
		const root = document.documentElement;
		root.setAttribute('data-theme', 'dark');
		root.classList.add('dark');
		toggleTheme();
		expect(root.getAttribute('data-theme')).toBe(null);
		expect(root.classList.contains('dark')).toBe(false);
		expect(localStorage.getItem('theme')).toBe('light');
	});
});
