import { expect, test } from '@playwright/test';

test.beforeEach(async ({ page }) => {
	await page.route('**/api/**', async (route) => {
		const request = route.request();
		const method = request.method();
		const url = request.url();

		if (method === 'POST' || method === 'PUT' || method === 'PATCH' || method === 'DELETE') {
			return route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify({ ok: true })
			});
		}

		if (url.includes('/api/logs/init')) {
			return route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify({ ok: true })
			});
		}

		return route.fulfill({
			status: 200,
			contentType: 'application/json',
			body: JSON.stringify([])
		});
	});
});

test('loads Today and can start a session', async ({ page }) => {
	await page.goto('/');

	await expect(page.getByRole('heading', { name: 'Today' })).toBeVisible();
	await page.getByRole('button', { name: 'Start session' }).click();
	await expect(page.getByText('Session notes')).toBeVisible();
	await expect(page.getByRole('button', { name: 'End session' })).toBeVisible();
});

test('theme toggle persists across reload', async ({ page }) => {
	await page.goto('/');

	const toggle = page.getByRole('button', { name: /toggle dark mode/i });
	await toggle.click();
	await expect(page.locator('html')).toHaveClass(/dark/);

	await page.reload();
	await expect(page.locator('html')).toHaveClass(/dark/);
});
