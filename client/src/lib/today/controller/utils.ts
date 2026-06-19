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

// Local (offline) entities use negative integer ids as a sentinel for
// "not yet persisted on the server" (see the `id < 0` checks throughout the
// controller). A naive `-(Date.now() * 1e6 + rand)` overflows MAX_SAFE_INTEGER,
// so the random component is lost to rounding and ids collide. Instead we keep a
// module-level counter, seeded from the clock and strictly decreasing, which
// stays well within the safe-integer range and is collision-free by construction.
let lastLocalId = 0;

export function makeLocalNumericId(): number {
	const candidate = -Date.now();
	// Use the clock when it has advanced; otherwise decrement so rapid successive
	// calls within the same millisecond still produce unique, ordered ids.
	if (lastLocalId === 0 || candidate < lastLocalId) {
		lastLocalId = candidate;
	} else {
		lastLocalId -= 1;
	}
	return lastLocalId;
}
