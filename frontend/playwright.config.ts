import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests/e2e',
  workers: 1,
  timeout: 30_000,
  use: {
    baseURL: 'http://127.0.0.1:8080',
    trace: 'on-first-retry',
  },
  webServer: {
    command:
      'cd ../backend && DATA_DIR=../data-e2e STATIC_DIR=../frontend/build AUTH_USERNAME=e2e AUTH_PASSWORD=e2e cargo run -p api',
    url: 'http://127.0.0.1:8080/api/v1/health',
    reuseExistingServer: !!process.env.CI,
    timeout: 120_000,
  },
});
