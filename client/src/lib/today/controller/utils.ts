export function getErrorMessage(e: unknown): string {
	if (e instanceof Error) return e.message;
	return 'Something went wrong';
}

export function isNetworkFailure(e: unknown): boolean {
	if (typeof navigator !== 'undefined' && navigator.onLine === false) return true;
	// A request that timed out or was aborted (see the request timeout in api.ts) is
	// a connectivity problem, not a bug — treat it as offline so the write is queued
	// and retried on reconnect rather than surfaced as an error (F-MED-7).
	if (
		typeof DOMException !== 'undefined' &&
		e instanceof DOMException &&
		(e.name === 'TimeoutError' || e.name === 'AbortError')
	) {
		return true;
	}
	// A failed fetch throws a TypeError, but so do plenty of genuine bugs
	// ("x is not a function", undefined access). Matching every TypeError flips
	// the app into offline mode on real errors and strands data as "pending".
	// Match the fetch-specific failure messages instead — they cover every
	// browser's wording (Chrome "Failed to fetch", Firefox "NetworkError…",
	// Safari "Load failed") — and let other errors propagate.
	const message = e instanceof Error ? e.message : String(e);
	return /failed to fetch|networkerror|load failed|connection|timed out|the operation was aborted/i.test(
		message
	);
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
