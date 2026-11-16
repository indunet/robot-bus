import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests', timeout: 30_000, expect: { timeout: 10_000 },
  workers: 1, fullyParallel: false, retries: 0,
  outputDir: 'test-results/browser',
  reporter: [['list'], ['junit', { outputFile: 'test-results/results.xml' }]],
  use: { baseURL: process.env.ROS2_WEB_URL ?? 'http://127.0.0.1:8080',
    browserName: 'chromium', trace: 'retain-on-failure', screenshot: 'only-on-failure' },
});
