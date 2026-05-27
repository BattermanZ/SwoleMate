import { readable } from 'svelte/store';

/**
 * Desktop breakpoint in px. DUPLICATED in the CSS media query inside
 * routes/+layout.svelte (`@media (min-width: 1024px)`) because CSS cannot read
 * JS values. Keep the two in sync.
 */
export const DESKTOP_MIN_WIDTH = 1024;

const QUERY = `(min-width: ${DESKTOP_MIN_WIDTH}px)`;

/**
 * `true` on desktop, `false` on mobile, `undefined` on the server / before mount.
 * Drives per-screen content-tree selection. Treat `undefined` as mobile via
 * `isDesktopView`.
 */
export const isDesktop = readable<boolean | undefined>(undefined, (set) => {
	if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
		return;
	}
	const mql = window.matchMedia(QUERY);
	set(mql.matches);
	const handler = (e: MediaQueryListEvent) => set(e.matches);
	mql.addEventListener('change', handler);
	return () => mql.removeEventListener('change', handler);
});

/** Safe accessor: unknown/`undefined` resolves to mobile. */
export function isDesktopView(value: boolean | undefined): boolean {
	return value === true;
}
