import { afterEach, describe, expect, it } from 'vitest';
import {
	readTimerSoundPreference,
	writeTimerSoundPreference
} from '$lib/preferences/timerSound';

describe('timer sound preference', () => {
	afterEach(() => {
		localStorage.clear();
	});

	it('defaults to on when nothing is stored', () => {
		expect(readTimerSoundPreference()).toBe(true);
	});

	it('round-trips a disabled preference', () => {
		writeTimerSoundPreference(false);
		expect(readTimerSoundPreference()).toBe(false);
	});

	it('round-trips an enabled preference', () => {
		writeTimerSoundPreference(false);
		writeTimerSoundPreference(true);
		expect(readTimerSoundPreference()).toBe(true);
	});
});
