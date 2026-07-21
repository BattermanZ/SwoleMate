import { beforeEach, describe, expect, it, vi } from 'vitest';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'content-type': 'application/json' }
	});
}

// Regression tests for audit findings:
//  - F-CRIT-1: a 401 mid-workout must NOT wipe unsynced offline session records.
//  - F-HIGH-4: switching accounts must preserve the departing user's sessions and
//    must not touch a different user's data.
describe('auth preserves unsynced offline sessions (F-CRIT-1 / F-HIGH-4)', () => {
	beforeEach(() => {
		localStorage.clear();
		vi.resetModules();
	});

	async function seedSession(userId: number, sessionId: number, status: string) {
		const { setActiveUserId } = await import('$lib/auth/scope');
		const { sessionKeyForId, saveOfflineSession } = await import('$lib/offline/todaySessions');
		setActiveUserId(userId);
		const key = sessionKeyForId(sessionId);
		await saveOfflineSession({
			key,
			status: status as never,
			updated_at: new Date(0).toISOString(),
			session: { id: sessionId } as never
		});
		return key;
	}

	it('keeps in-progress session records when a 401 arrives mid-workout', async () => {
		vi.stubGlobal('indexedDB', undefined);
		vi.doMock('$app/environment', () => ({ browser: true }));

		const { auth } = await import('$lib/auth');
		const { kvGet } = await import('$lib/offline/storage');

		await auth.login(
			'alice',
			'pw',
			vi.fn(async () =>
				jsonResponse({
					status: 'ok',
					user: { id: 1, username: 'alice', role: 'user', must_change_password: false }
				})
			) as unknown as typeof fetch
		);

		const key = await seedSession(1, 5, 'in_progress');
		expect(await kvGet(key)).toBeTruthy();

		// Backend session lapses -> next call returns 401.
		await auth.refresh(vi.fn(async () => jsonResponse({ message: 'Unauthorized' }, 401)) as never);

		// The unsynced session must survive the 401.
		await vi.waitFor(async () => expect(await kvGet(key)).toBeTruthy());

		vi.doUnmock('$app/environment');
		vi.unstubAllGlobals();
	});

	it('preserves another user\'s session and only clears the departing user on switch', async () => {
		vi.stubGlobal('indexedDB', undefined);
		vi.doMock('$app/environment', () => ({ browser: true }));

		const { auth } = await import('$lib/auth');
		const { kvGet } = await import('$lib/offline/storage');
		const { persistPlannedTemplate } = await import(
			'$lib/today/controller/actions/plannedTemplate'
		);

		// User 1 signs in and ends a workout offline (pending_sync, never uploaded),
		// plus has a read-only planned-template cache.
		await auth.login(
			'alice',
			'pw',
			vi.fn(async () =>
				jsonResponse({
					status: 'ok',
					user: { id: 1, username: 'alice', role: 'user', must_change_password: false }
				})
			) as unknown as typeof fetch
		);
		const aliceSession = await seedSession(1, 9, 'pending_sync');
		const { setActiveUserId } = await import('$lib/auth/scope');
		setActiveUserId(1);
		await persistPlannedTemplate(9, [{ name: 'Bench', notes: '' } as never]);
		expect(await kvGet('u1:today.plannedTemplate')).toBeTruthy();

		// User 2 signs in on the same device.
		await auth.login(
			'bob',
			'pw',
			vi.fn(async () =>
				jsonResponse({
					status: 'ok',
					user: { id: 2, username: 'bob', role: 'user', must_change_password: false }
				})
			) as unknown as typeof fetch
		);

		// Alice's unsynced workout survives; her read-only cache is cleared.
		expect(await kvGet(aliceSession)).toBeTruthy();
		expect(await kvGet('u1:today.plannedTemplate')).toBeNull();

		vi.doUnmock('$app/environment');
		vi.unstubAllGlobals();
	});
});
