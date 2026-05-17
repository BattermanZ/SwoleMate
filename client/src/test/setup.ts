import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/svelte';
import { afterEach, beforeEach, vi } from 'vitest';

function installBrowserMocks() {
	Object.defineProperty(window, 'matchMedia', {
		writable: true,
		value: (query: string) => ({
			matches: false,
			media: query,
			onchange: null,
			addListener: () => {},
			removeListener: () => {},
			addEventListener: () => {},
			removeEventListener: () => {},
			dispatchEvent: () => false
		})
	});

	Object.defineProperty(globalThis, 'caches', {
		configurable: true,
		writable: true,
		value: {
			keys: vi.fn(async () => []),
			delete: vi.fn(async () => true)
		}
	});

	Object.defineProperty(window.navigator, 'serviceWorker', {
		configurable: true,
		value: {
			getRegistrations: vi.fn(async () => []),
			register: vi.fn(async () => ({ scope: '/' }))
		}
	});
}

beforeEach(() => {
	installBrowserMocks();
});

afterEach(() => {
	cleanup();
	localStorage.clear();
	sessionStorage.clear();
	document.documentElement.className = '';
	document.documentElement.removeAttribute('style');
	document.documentElement.removeAttribute('data-theme');
	vi.unstubAllGlobals();
});
