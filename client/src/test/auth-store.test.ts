import { beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'content-type': 'application/json' }
	});
}

describe('auth store', () => {
	beforeEach(() => {
		localStorage.clear();
	});

	it('marks unauthenticated on 401', async () => {
		vi.resetModules();
		const { auth } = await import('$lib/auth');

		const mockFetch = vi.fn(async () => jsonResponse({ message: 'Unauthorized' }, 401));
		await auth.refresh(mockFetch as unknown as typeof fetch);

		expect(get(auth.state).status).toBe('unauthenticated');
		expect(get(auth.state).user).toBe(null);
	});

	it('keeps user and marks offline on network errors', async () => {
		vi.resetModules();
		const { auth } = await import('$lib/auth');

		const loginFetch = vi.fn(async () =>
			jsonResponse({ status: 'ok', user: { id: 1, username: 'alice', role: 'user' } })
		);
		await auth.login('alice', 'pw', loginFetch as unknown as typeof fetch);

		const offlineFetch = vi.fn(async () => {
			throw new TypeError('Failed to fetch');
		});
		await auth.refresh(offlineFetch as unknown as typeof fetch);

		const state = get(auth.state);
		expect(state.status).toBe('authenticated');
		expect(state.user?.username).toBe('alice');
		expect(state.offline).toBe(true);
	});
});
