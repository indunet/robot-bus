import { test, expect, type Page } from '@playwright/test';

async function counter(page: Page, id: string) {
  return Number(await page.locator('#' + id).textContent());
}

test.beforeEach(async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('#connection')).toHaveText('ready');
  await expect.poll(() => counter(page, 'sequence')).toBeGreaterThan(0);
});

test('topics: ROS telemetry and browser command reach opposite endpoints', async ({ page }) => {
  const before = await counter(page, 'sequence');
  await expect.poll(() => counter(page, 'sequence')).toBeGreaterThan(before);
  const command = `browser-${crypto.randomUUID()}`;
  await page.getByLabel('Command text').fill(command);
  // Topic subscription establishment is asynchronous. Repeat an idempotent
  // command within a bounded window, and assert the ROS-side acknowledgement.
  await expect(async () => {
    await page.getByRole('button', { name: 'Send command', exact: true }).click();
    await expect(page.locator('#last_command')).toHaveText(command, { timeout: 500 });
  }).toPass({ timeout: 10_000 });
});

test('service success changes ROS state', async ({ page }) => {
  const before = await counter(page, 'resets');
  await page.getByRole('button', { name: 'Reset', exact: true }).click();
  await expect(page.locator('#service-result')).toHaveText(`success: reset:${before + 1}`);
  await expect.poll(() => counter(page, 'resets')).toBe(before + 1);
});

test('bridge service timeout is typed and a later request still succeeds', async ({ page }) => {
  const started = Date.now();
  await page.getByRole('button', { name: 'Test timeout', exact: true }).click();
  await expect(page.locator('#service-result')).toHaveText(/^timeout:/);
  expect(Date.now() - started).toBeLessThan(4000); // Earlier than the 5 s WS deadline.
  await page.getByRole('button', { name: 'Reset', exact: true }).click();
  await expect(page.locator('#service-result')).toHaveText(/^success: reset:/);
});

test('action feedback and exact result', async ({ page }) => {
  await page.getByRole('button', { name: 'Start task' }).click();
  await expect(page.locator('#feedback')).not.toBeEmpty();
  await expect(page.locator('#action-result')).toHaveText('success: 0, 1, 1, 2, 3, 5, 8, 13');
});

test('cancel reaches ROS and returns a cancelled outcome', async ({ page }) => {
  const before = await counter(page, 'goals_cancelled');
  await page.getByLabel('Terms').fill('30');
  await page.getByRole('button', { name: 'Start task' }).click();
  await expect(page.locator('#feedback')).not.toBeEmpty();
  await page.getByRole('button', { name: 'Cancel task' }).click();
  await expect(page.locator('#action-result')).toHaveText(/^cancelled:/);
  await expect.poll(() => counter(page, 'goals_cancelled')).toBe(before + 1);
});

test('ROS rejects an invalid goal', async ({ page }) => {
  await page.getByText('Exercise failure paths').click();
  await page.getByRole('button', { name: 'Reject invalid goal' }).click();
  await expect(page.locator('#action-result')).toHaveText(/^rejected:/);
});

test('ROS abort remains distinct from cancellation', async ({ page }) => {
  const before = await counter(page, 'goals_aborted');
  await page.getByText('Exercise failure paths').click();
  await page.getByRole('button', { name: 'Abort on ROS' }).click();
  await expect(page.locator('#action-result')).toHaveText(/^aborted:/);
  await expect.poll(() => counter(page, 'goals_aborted')).toBe(before + 1);
});

test('bridge action deadline also requests cancellation on ROS', async ({ page }) => {
  const before = await counter(page, 'goals_cancelled');
  await page.getByText('Exercise failure paths').click();
  await page.getByRole('button', { name: 'Test action deadline' }).click();
  await expect(page.locator('#action-result')).toHaveText(/^timeout:/);
  await expect.poll(() => counter(page, 'goals_cancelled')).toBe(before + 1);
});
