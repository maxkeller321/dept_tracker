import { test, expect } from './fixtures';
import { fillNumber } from './helpers';
import path from 'path';
import fs from 'fs';
import os from 'os';

test('export and import JSON round-trip', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Add loan' }).click();
  await page.getByLabel('Name').fill('Backup Test');
  await fillNumber(page, '#balance', '5000');
  await fillNumber(page, '#apr', '2.5');
  await fillNumber(page, '#tilgung-euro', '100');
  const create = page.waitForResponse(
    (r) => r.url().includes('/api/v1/loans') && r.request().method() === 'POST',
  );
  await page.getByRole('button', { name: 'Save loan' }).click();
  expect((await create).ok()).toBeTruthy();
  await expect(page.getByRole('button', { name: /Backup Test/ })).toHaveCount(1);

  await page.getByRole('button', { name: 'Settings' }).click();
  const dlPromise = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Export JSON backup' }).click();
  const dl = await dlPromise;
  const tmp = path.join(os.tmpdir(), 'dept-tracker-e2e.json');
  await dl.saveAs(tmp);
  const json = JSON.parse(fs.readFileSync(tmp, 'utf-8'));
  expect(json.schema_version).toBe(1);

  await page.getByRole('button', { name: 'Settings' }).click();
  page.on('dialog', (d) => d.accept());
  await page.locator('input[type="file"]').setInputFiles(tmp);
  await expect(page.getByText('Import complete')).toBeVisible({ timeout: 10_000 });
  await expect(page.getByRole('button', { name: /Backup Test/ })).toHaveCount(1);
});
