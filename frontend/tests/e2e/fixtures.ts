import { test as base, expect } from '@playwright/test';

const EMPTY_BUNDLE = { schema_version: 1, loans: [] as unknown[] };
const E2E_USER = 'e2e';
const E2E_PASS = 'e2e';

async function apiLoginCookie(
  request: import('@playwright/test').APIRequestContext,
): Promise<string> {
  const res = await request.post('/api/v1/auth/login', {
    data: { username: E2E_USER, password: E2E_PASS },
  });
  if (!res.ok()) {
    throw new Error(`Login failed: ${res.status()} ${await res.text()}`);
  }
  const setCookie = res.headers()['set-cookie'] ?? '';
  return setCookie.split(';')[0] ?? '';
}

export async function loginUi(page: import('@playwright/test').Page) {
  await page.getByLabel('Username').fill(E2E_USER);
  await page.getByLabel('Password').fill(E2E_PASS);
  await page.locator('form button[type="submit"]').click();
  await expect(page.getByRole('button', { name: 'Add loan' })).toBeVisible();
}

export const test = base.extend({
  page: async ({ page, request }, use) => {
    await page.addInitScript(() => {
      localStorage.setItem('dept-tracker-locale', 'en');
    });
    const cookie = await apiLoginCookie(request);
    const res = await request.post('/api/v1/import?confirm=true', {
      data: EMPTY_BUNDLE,
      headers: { Cookie: cookie },
    });
    if (!res.ok()) {
      throw new Error(`Failed to reset database: ${res.status()} ${await res.text()}`);
    }
    await page.goto('/');
    await loginUi(page);
    await use(page);
  },
});

export { expect };
