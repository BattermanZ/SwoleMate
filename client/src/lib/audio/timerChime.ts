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

function setAudioSessionType(type: 'ambient' | 'transient-solo'): void {
	try {
		const session = (navigator as unknown as { audioSession?: { type: string } }).audioSession;
		if (session) session.type = type;
	} catch {
		// not supported — best effort only
	}
}

/**
 * Unlock playback. Must be called from within a user gesture (e.g. starting the
 * timer) so the chime is allowed to play later without interaction. Safe to call
 * repeatedly and on every platform.
 */
export async function unlockTimerChime(): Promise<void> {
	// Claim an "ambient" session for priming so we mix with — and never interrupt —
	// the user's music/podcast while the timer is running. We only switch to an
	// exclusive session at the moment the chime actually rings (see playTimerChime).
	setAudioSessionType('ambient');

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

	// "transient-solo" briefly pauses other audio (like a navigation prompt),
	// plays the chime exclusively — through the hardware mute switch on iOS —
	// and lets the system resume the user's music/podcast when it finishes.
	setAudioSessionType('transient-solo');

	try {
		el.currentTime = 0;
		const result = el.play();
		if (result && typeof result.catch === 'function') result.catch(() => {});
	} catch {
		// ignore playback failures
	}
}
