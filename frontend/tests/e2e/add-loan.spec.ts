import { test, expect } from './fixtures';
import { fillNumber } from './helpers';

test('add loan from header button', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Add loan' }).click();
  await page.getByLabel('Name').fill('Car Loan');
  await fillNumber(page, '#balance', '8000');
  await fillNumber(page, '#apr', '4.5');
  await fillNumber(page, '#tilgung-euro', '350');
  const create = page.waitForResponse(
    (r) => r.url().includes('/api/v1/loans') && r.request().method() === 'POST',
  );
  await page.getByRole('button', { name: 'Save loan' }).click();
  expect((await create).ok()).toBeTruthy();
  await expect(page.getByRole('button', { name: /Car Loan/ })).toBeVisible();
});
