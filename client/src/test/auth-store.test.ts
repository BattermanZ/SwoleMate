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
			jsonResponse({
				status: 'ok',
				user: { id: 1, username: 'alice', role: 'user', must_change_password: false }
			})
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

	it('initializes user scope from stored auth user before refresh', async () => {
		localStorage.setItem(
			'auth.lastUser',
			JSON.stringify({
				id: 7,
				username: 'bob',
				role: 'user',
				must_change_password: false
			})
		);

		vi.resetModules();
		vi.doMock('$app/environment', () => ({ browser: true }));
		await import('$lib/auth');
		const { getActiveUserId, scopedKey } = await import('$lib/auth/scope');

		expect(getActiveUserId()).toBe('7');
		// Scope is initialised from the stored user, so scoped keys are prefixed.
		expect(scopedKey('today.plannedTemplate')).toBe('u7:today.plannedTemplate');
		vi.doUnmock('$app/environment');
	});
});
