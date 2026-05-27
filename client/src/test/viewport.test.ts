import { get } from 'svelte/store';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { DESKTOP_MIN_WIDTH, isDesktop, isDesktopView } from '$lib/stores/viewport';

type Listener = (e: { matches: boolean }) => void;

function installMatchMedia(initialMatches: boolean) {
	let listeners: Listener[] = [];
	const mql = {
		matches: initialMatches,
		media: '',
		addEventListener: (_type: string, cb: Listener) => {
			listeners.push(cb);
		},
		removeEventListener: (_type: string, cb: Listener) => {
			listeners = listeners.filter((l) => l !== cb);
		}
	};
	Object.defineProperty(window, 'matchMedia', {
		configurable: true,
		writable: true,
		value: vi.fn(() => mql)
	});
	return {
		emit(matches: boolean) {
			mql.matches = matches;
			listeners.forEach((l) => l({ matches }));
		}
	};
}

afterEach(() => {
	vi.restoreAllMocks();
});

describe('isDesktopView', () => {
	it('treats undefined and false as not-desktop (mobile-safe default)', () => {
		expect(isDesktopView(undefined)).toBe(false);
		expect(isDesktopView(false)).toBe(false);
	});

	it('treats true as desktop', () => {
		expect(isDesktopView(true)).toBe(true);
	});
});

describe('isDesktop store', () => {
	it('exposes the 1024px breakpoint constant', () => {
		expect(DESKTOP_MIN_WIDTH).toBe(1024);
	});

	it('reflects the initial matchMedia state on subscribe', () => {
		installMatchMedia(true);
		let value: boolean | undefined;
		const unsub = isDesktop.subscribe((v) => (value = v));
		expect(value).toBe(true);
		unsub();
	});

	it('updates when the media query changes', () => {
		const mm = installMatchMedia(false);
		let value: boolean | undefined;
		const unsub = isDesktop.subscribe((v) => (value = v));
		expect(value).toBe(false);
		mm.emit(true);
		expect(value).toBe(true);
		unsub();
	});
});
