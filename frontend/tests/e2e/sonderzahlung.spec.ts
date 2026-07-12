import { test, expect } from './fixtures';
import { fillNumber } from './helpers';

test('record extra payment on expanded loan', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Add loan' }).click();
  await page.getByLabel('Name').fill('Extra Pay Test');
  await fillNumber(page, '#balance', '10000');
  await fillNumber(page, '#apr', '5');
  await fillNumber(page, '#tilgung-euro', '200');
  const create = page.waitForResponse(
    (r) => r.url().includes('/api/v1/loans') && r.request().method() === 'POST',
  );
  await page.getByRole('button', { name: 'Save loan' }).click();
  expect((await create).ok()).toBeTruthy();
  const card = page.locator('article').filter({ hasText: 'Extra Pay Test' });
  await card.getByTestId('loan-expand').click();
  await expect(card.locator('.loan-details')).toBeVisible();
  await expect(card.getByText('Applied as of today.')).toBeVisible();
  await fillNumber(page, '#sonder-extra-amount', '500');
  const extra = page.waitForResponse(
    (r) => r.url().includes('/sonderzahlungen/immediate') && r.request().method() === 'POST',
  );
  await card.getByRole('button', { name: 'Apply now' }).click();
  const extraResp = await extra;
  expect(extraResp.ok()).toBeTruthy();
  const body = extraResp.request().postDataJSON() as {
    recalculate_from_past?: boolean;
    paid_at?: string;
  };
  expect(body.recalculate_from_past).toBeFalsy();
  await expect(page.getByRole('button', { name: /Extra Pay Test/ })).toBeVisible();
});

test('schedule future extra payment from plan row', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Add loan' }).click();
  await page.getByLabel('Name').fill('Schedule Extra Test');
  await fillNumber(page, '#balance', '10000');
  await fillNumber(page, '#apr', '5');
  await fillNumber(page, '#tilgung-euro', '200');
  const create = page.waitForResponse(
    (r) => r.url().includes('/api/v1/loans') && r.request().method() === 'POST',
  );
  await page.getByRole('button', { name: 'Save loan' }).click();
  expect((await create).ok()).toBeTruthy();
  const card = page.locator('article').filter({ hasText: 'Schedule Extra Test' });
  await card.getByTestId('loan-expand').click();

  const future = new Date();
  future.setMonth(future.getMonth() + 3);
  const futureIso = future.toISOString().slice(0, 10);

  await fillNumber(page, '#sonder-schedule-amount', '250');
  await page.locator('#sonder-schedule-date').fill(futureIso);
  const scheduled = page.waitForResponse(
    (r) => r.url().includes('/sonderzahlungen/scheduled') && r.request().method() === 'POST',
  );
  await card.getByRole('button', { name: 'Schedule' }).click();
  const scheduledResp = await scheduled;
  expect(scheduledResp.ok()).toBeTruthy();
  await expect(card.getByText('Upcoming extra payments')).toBeVisible();
});
