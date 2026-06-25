import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
	__setTimerChimeElementForTesting,
	playTimerChime,
	unlockTimerChime
} from '$lib/audio/timerChime';
import { writeTimerSoundPreference } from '$lib/preferences/timerSound';

function fakeAudio() {
	return {
		muted: false,
		currentTime: 5,
		play: vi.fn().mockResolvedValue(undefined),
		pause: vi.fn()
	};
}

describe('timer chime', () => {
	beforeEach(() => {
		localStorage.clear();
	});

	afterEach(() => {
		__setTimerChimeElementForTesting(null);
		// remove any audioSession stub we added
		delete (navigator as unknown as { audioSession?: unknown }).audioSession;
	});

	it('plays the chime from the start when the preference is on', async () => {
		const el = fakeAudio();
		__setTimerChimeElementForTesting(el as unknown as HTMLAudioElement);

		playTimerChime();

		expect(el.currentTime).toBe(0);
		expect(el.play).toHaveBeenCalledTimes(1);
	});

	it('does not play when the preference is off', () => {
		writeTimerSoundPreference(false);
		const el = fakeAudio();
		__setTimerChimeElementForTesting(el as unknown as HTMLAudioElement);

		playTimerChime();

		expect(el.play).not.toHaveBeenCalled();
	});

	it('swallows playback rejection without throwing', () => {
		const el = fakeAudio();
		el.play = vi.fn().mockRejectedValue(new Error('NotAllowedError'));
		__setTimerChimeElementForTesting(el as unknown as HTMLAudioElement);

		expect(() => playTimerChime()).not.toThrow();
	});

	it('primes the element on unlock and leaves it unmuted', async () => {
		const el = fakeAudio();
		__setTimerChimeElementForTesting(el as unknown as HTMLAudioElement);

		await unlockTimerChime();

		expect(el.play).toHaveBeenCalled();
		expect(el.pause).toHaveBeenCalled();
		expect(el.muted).toBe(false);
		expect(el.currentTime).toBe(0);
	});

	it('claims an ambient audio session on unlock so other audio keeps playing', async () => {
		const session = { type: 'auto' };
		(navigator as unknown as { audioSession: typeof session }).audioSession = session;
		const el = fakeAudio();
		__setTimerChimeElementForTesting(el as unknown as HTMLAudioElement);

		await unlockTimerChime();

		expect(session.type).toBe('ambient');
	});

	it('claims a transient-solo session when the chime rings so music resumes after', () => {
		const session = { type: 'ambient' };
		(navigator as unknown as { audioSession: typeof session }).audioSession = session;
		const el = fakeAudio();
		__setTimerChimeElementForTesting(el as unknown as HTMLAudioElement);

		playTimerChime();

		expect(session.type).toBe('transient-solo');
		expect(el.play).toHaveBeenCalledTimes(1);
	});

	it('does not throw on unlock when the audio session API is absent', async () => {
		const el = fakeAudio();
		__setTimerChimeElementForTesting(el as unknown as HTMLAudioElement);

		await expect(unlockTimerChime()).resolves.toBeUndefined();
	});
});
