import { test, expect } from './fixtures';
import { fillNumber } from './helpers';

test('dashboard empty state and add loan', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Dept Tracker' })).toBeVisible();
  await expect(page.getByText('No loans yet')).toBeVisible();
  await page.getByRole('button', { name: 'Add your first loan' }).click();
  await page.getByLabel('Name').fill('E2E Mortgage');
  await fillNumber(page, '#balance', '150000');
  await fillNumber(page, '#apr', '3.25');
  await fillNumber(page, '#tilgung-euro', '1200');
  const create = page.waitForResponse(
    (r) => r.url().includes('/api/v1/loans') && r.request().method() === 'POST',
  );
  await page.getByRole('button', { name: 'Save loan' }).click();
  expect((await create).ok()).toBeTruthy();
  await expect(page.getByRole('button', { name: /E2E Mortgage/ })).toBeVisible();
  await expect(page.getByText('Total remaining')).toBeVisible();
});
