import { test, expect } from './fixtures';
import { fillNumber } from './helpers';

test('interest section visible for percent tilgung loan', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Add loan' }).click();
  await page.getByLabel('Name').fill('Percent Loan');
  await fillNumber(page, '#balance', '50000');
  await fillNumber(page, '#apr', '3.5');
  await page.getByLabel('Repayment').selectOption('tilgung_percent');
  await fillNumber(page, '#tilgung-pct', '2');
  const create = page.waitForResponse(
    (r) => r.url().includes('/api/v1/loans') && r.request().method() === 'POST',
  );
  await page.getByRole('button', { name: 'Save loan' }).click();
  expect((await create).ok()).toBeTruthy();
  await page.locator('article').filter({ hasText: 'Percent Loan' }).getByTestId('loan-expand').click();
  await expect(page.locator('article').filter({ hasText: 'Percent Loan' }).locator('.loan-details')).toBeVisible();
  await expect(page.getByRole('region', { name: 'Interest summary' })).toBeVisible();
  await expect(page.getByText('Paid to date')).toBeVisible();
});
