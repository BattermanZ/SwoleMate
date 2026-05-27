import { describe, expect, it } from 'vitest';
import { isActive } from '$lib/components/shell/nav';

describe('isActive', () => {
	it('returns false when current is undefined', () => {
		expect(isActive('/', undefined)).toBe(false);
	});

	it('matches the root only on exact root path', () => {
		expect(isActive('/', '/')).toBe(true);
		expect(isActive('/', '/workouts')).toBe(false);
	});

	it('matches an exact non-root path', () => {
		expect(isActive('/progress', '/progress')).toBe(true);
	});

	it('matches nested child routes via prefix', () => {
		expect(isActive('/workouts', '/workouts/123')).toBe(true);
	});

	it('does not match unrelated routes', () => {
		expect(isActive('/workouts', '/progress')).toBe(false);
	});

	it('does not treat a path as a prefix of a longer sibling segment', () => {
		expect(isActive('/work', '/workouts')).toBe(false);
	});
});
