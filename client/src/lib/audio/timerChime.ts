import { readTimerSoundPreference } from '$lib/preferences/timerSound';

const CHIME_SRC = '/timer-done.wav';

let element: HTMLAudioElement | null = null;
let primed = false;

/** Test seam: inject (or clear) the audio element and reset prime state. */
export function __setTimerChimeElementForTesting(el: HTMLAudioElement | null): void {
	element = el;
	primed = false;
}

function resolveElement(): HTMLAudioElement | null {
	if (element) return element;
	if (typeof Audio === 'undefined') return null;
	element = new Audio(CHIME_SRC);
	element.preload = 'auto';
	return element;
}

function declarePlaybackSession(): void {
	try {
		// On iOS Safari (16.4+) this routes audio into the "playback" category,
		// which plays through the hardware mute switch.
		const session = (navigator as unknown as { audioSession?: { type: string } }).audioSession;
		if (session) session.type = 'playback';
	} catch {
		// not supported — the <audio> element path still bypasses mute on iOS
	}
}

/**
 * Unlock playback. Must be called from within a user gesture (e.g. starting the
 * timer) so the chime is allowed to play later without interaction. Safe to call
 * repeatedly and on every platform.
 */
export async function unlockTimerChime(): Promise<void> {
	declarePlaybackSession();

	const el = resolveElement();
	if (!el || primed) return;

	try {
		el.muted = true;
		await el.play();
		el.pause();
		el.currentTime = 0;
		el.muted = false;
		primed = true;
	} catch {
		// Leave unprimed so the next gesture can retry.
		el.muted = false;
	}
}

/** Play the completion chime, honouring the user's preference. Best-effort. */
export function playTimerChime(): void {
	if (!readTimerSoundPreference()) return;

	const el = resolveElement();
	if (!el) return;

	try {
		el.currentTime = 0;
		const result = el.play();
		if (result && typeof result.catch === 'function') result.catch(() => {});
	} catch {
		// ignore playback failures
	}
}
