import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

function okResponse(status = 200): Response {
	return new Response('{}', { status, headers: { 'content-type': 'application/json' } });
}

describe('logger', () => {
	beforeEach(() => {
		vi.useFakeTimers();
		localStorage.clear();
	});

	afterEach(() => {
		vi.restoreAllMocks();
		vi.useRealTimers();
	});

	it('does not send queued logs while remote logging is disabled', async () => {
		vi.resetModules();
		const fetchMock = vi.fn(async () => okResponse());
		vi.stubGlobal('fetch', fetchMock);
		const { logger } = await import('$lib/logger');

		logger.warn('test', 'warn');
		await vi.advanceTimersByTimeAsync(1_100);

		expect(fetchMock).not.toHaveBeenCalled();
	});

	it('sends warn/error logs when remote logging is enabled', async () => {
		vi.resetModules();
		const fetchMock = vi.fn(async () => okResponse());
		vi.stubGlobal('fetch', fetchMock);
		const { logger } = await import('$lib/logger');

		logger.setRemoteEnabled(true);
		logger.info('test', 'info skipped remotely');
		logger.warn('test', 'warn sent');
		logger.error('test', 'error sent');

		await vi.advanceTimersByTimeAsync(1_100);

		expect(fetchMock).toHaveBeenCalledTimes(1);
		const [, init] = fetchMock.mock.calls[0] as unknown as [
			RequestInfo | URL,
			RequestInit | undefined
		];
		const body = JSON.parse(String(init?.body ?? '[]'));
		expect(body.map((entry: { level: string }) => entry.level)).toEqual(['warn', 'error']);
	});

	it('caps remote queue at 500 entries', async () => {
		vi.resetModules();
		const fetchMock = vi.fn(async () => okResponse());
		vi.stubGlobal('fetch', fetchMock);
		const { logger } = await import('$lib/logger');

		logger.setRemoteEnabled(true);
		for (let i = 0; i < 620; i += 1) {
			logger.warn('cap', `entry ${i}`);
		}

		await vi.advanceTimersByTimeAsync(1_100);

		const [, init] = fetchMock.mock.calls[0] as unknown as [
			RequestInfo | URL,
			RequestInit | undefined
		];
		const body = JSON.parse(String(init?.body ?? '[]'));
		expect(body).toHaveLength(500);
		expect(body[0]?.message).toBe('entry 120');
	});
});
