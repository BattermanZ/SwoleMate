export function getErrorMessage(e: unknown): string {
	if (e instanceof Error) return e.message;
	return 'Something went wrong';
}

export function isNetworkFailure(e: unknown): boolean {
	if (typeof navigator !== 'undefined' && navigator.onLine === false) return true;
	if (e instanceof TypeError) return true;
	const message = e instanceof Error ? e.message : String(e);
	return /failed to fetch|networkerror|load failed|connection/i.test(message);
}

export function makeLocalNumericId(): number {
	const rand = Math.floor(Math.random() * 1_000_000);
	return -(Date.now() * 1_000_000 + rand);
}
