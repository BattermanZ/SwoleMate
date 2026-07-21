import { afterEach, describe, expect, it, vi } from 'vitest';

import { isNetworkFailure } from '$lib/today/controller/utils';

describe('isNetworkFailure', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('treats fetch failures as offline', () => {
		expect(isNetworkFailure(new TypeError('Failed to fetch'))).toBe(true); // Chrome
		expect(isNetworkFailure(new TypeError('NetworkError when attempting to fetch resource'))).toBe(
			true
		); // Firefox
		expect(isNetworkFailure(new TypeError('Load failed'))).toBe(true); // Safari
		expect(isNetworkFailure(new Error('connection reset'))).toBe(true);
	});

	it('treats a request timeout/abort as offline (F-MED-7)', () => {
		expect(isNetworkFailure(new DOMException('signal timed out', 'TimeoutError'))).toBe(true);
		expect(isNetworkFailure(new DOMException('aborted', 'AbortError'))).toBe(true);
		expect(isNetworkFailure(new Error('The operation was aborted'))).toBe(true);
	});

	it('does NOT treat an arbitrary TypeError (a real bug) as offline', () => {
		expect(isNetworkFailure(new TypeError('x is not a function'))).toBe(false);
		expect(isNetworkFailure(new TypeError("Cannot read properties of undefined (reading 'id')"))).toBe(
			false
		);
	});

	it('treats any error as offline when the browser reports offline', () => {
		vi.stubGlobal('navigator', { onLine: false });
		expect(isNetworkFailure(new TypeError('x is not a function'))).toBe(true);
		expect(isNetworkFailure(new Error('anything'))).toBe(true);
	});
});
