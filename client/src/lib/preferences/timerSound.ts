const TIMER_SOUND_KEY = 'settings.timerSound';

export function readTimerSoundPreference(): boolean {
	if (typeof localStorage === 'undefined') return true;

	try {
		// Default on: only an explicit opt-out disables the sound.
		return localStorage.getItem(TIMER_SOUND_KEY) !== 'false';
	} catch {
		return true;
	}
}

export function writeTimerSoundPreference(enabled: boolean): void {
	if (typeof localStorage === 'undefined') return;

	try {
		localStorage.setItem(TIMER_SOUND_KEY, enabled ? 'true' : 'false');
	} catch {
		// ignore persistence failures
	}
}
