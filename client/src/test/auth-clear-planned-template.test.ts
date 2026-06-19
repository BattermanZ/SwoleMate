import { beforeEach, describe, expect, it, vi } from 'vitest';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'content-type': 'application/json' }
	});
}

describe('logout clears planned-template residue (audit #6)', () => {
	beforeEach(() => {
		localStorage.clear();
		vi.resetModules();
	});

	it('removes the scoped today.plannedTemplate key on logout', async () => {
		vi.stubGlobal('indexedDB', undefined);
		vi.doMock('$app/environment', () => ({ browser: true }));
		const { auth } = await import('$lib/auth');
		const { persistPlannedTemplate } = await import(
			'$lib/today/controller/actions/plannedTemplate'
		);
		const { kvGet } = await import('$lib/offline/storage');

		const loginFetch = vi.fn(async () =>
			jsonResponse({
				status: 'ok',
				user: { id: 1, username: 'alice', role: 'user', must_change_password: false }
			})
		);
		await auth.login('alice', 'pw', loginFetch as unknown as typeof fetch);

		await persistPlannedTemplate(99, [{ name: 'Bench', notes: '' } as never]);
		expect(await kvGet('u1:today.plannedTemplate')).toBeTruthy();

		await auth.logout(vi.fn(async () => jsonResponse({ status: 'ok' })) as unknown as typeof fetch);

		await vi.waitFor(async () => expect(await kvGet('u1:today.plannedTemplate')).toBeNull());
		vi.doUnmock('$app/environment');
		vi.unstubAllGlobals();
	});
});
