const DEMO_MODE_KEY = 'settings.showDemoMode';

export function readDemoModePreference(): boolean {
	if (typeof localStorage === 'undefined') return false;

	try {
		return localStorage.getItem(DEMO_MODE_KEY) === 'true';
	} catch {
		return false;
	}
}

export function writeDemoModePreference(enabled: boolean): void {
	if (typeof localStorage === 'undefined') return;

	try {
		localStorage.setItem(DEMO_MODE_KEY, enabled ? 'true' : 'false');
	} catch {
		// ignore persistence failures
	}
}
