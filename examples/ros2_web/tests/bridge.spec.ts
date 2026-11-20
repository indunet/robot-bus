import { test, expect, type Page } from '@playwright/test';

async function counter(page: Page, id: string) {
  return Number(await page.locator('#' + id).textContent());
}

async function rosCounters(page: Page): Promise<Record<string, number>> {
  return JSON.parse(await page.locator('#telemetry').textContent() ?? '{}');
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
  const before = await rosCounters(page);
  const result = page.locator('#service-result');
  await page.getByRole('button', { name: 'Test timeout', exact: true }).click();
  await expect(result).toHaveText('timeout: timed out waiting for ROS future');
  await expect.poll(() => rosCounters(page)).toMatchObject({
    slow_started: before.slow_started + 1,
    slow_completed: before.slow_completed + 1,
  });
  await expect(result).toHaveText('timeout: timed out waiting for ROS future');
  await page.getByRole('button', { name: 'Reset', exact: true }).click();
  await expect(result).toHaveText(`success: reset:${before.resets + 1}`);
});

test('action feedback and exact result', async ({ page }) => {
  const before = await rosCounters(page);
  const feedback = page.locator('#feedback');
  await feedback.evaluate(element => {
    const observer = new MutationObserver(() => {
      if (element.textContent && document.getElementById('action-result')?.textContent === 'running') {
        element.setAttribute('data-feedback-while-running', element.textContent);
      }
    });
    observer.observe(element, { childList: true });
  });
  await page.getByRole('button', { name: 'Start task' }).click();
  await expect(feedback).toHaveAttribute('data-feedback-while-running', /^0(?:, \d+)*$/);
  await expect(page.locator('#action-result')).toHaveText('success: 0, 1, 1, 2, 3, 5, 8, 13');
  await expect.poll(() => rosCounters(page)).toMatchObject({
    goals_started: before.goals_started + 1,
    goals_succeeded: before.goals_succeeded + 1,
  });
});

test('cancel reaches ROS and returns a cancelled outcome', async ({ page }) => {
  const before = await rosCounters(page);
  await page.getByLabel('Terms').fill('30');
  await page.getByRole('button', { name: 'Start task' }).click();
  await expect(page.locator('#feedback')).not.toBeEmpty();
  await page.getByRole('button', { name: 'Cancel task' }).click();
  await expect(page.locator('#action-result')).toHaveText(/^cancelled:/);
  await expect.poll(() => rosCounters(page)).toMatchObject({
    goals_started: before.goals_started + 1,
    goals_cancelled: before.goals_cancelled + 1,
    goals_succeeded: before.goals_succeeded,
  });
});

test('ROS rejects an invalid goal', async ({ page }) => {
  const before = await rosCounters(page);
  await page.getByText('Exercise failure paths').click();
  await page.getByRole('button', { name: 'Reject invalid goal' }).click();
  await expect(page.locator('#action-result')).toHaveText(/^rejected:/);
  await expect.poll(() => rosCounters(page)).toMatchObject({
    goals_rejected: before.goals_rejected + 1,
    goals_started: before.goals_started,
  });
  await expect(page.locator('#feedback')).toBeEmpty();
});

test('ROS abort remains distinct from cancellation', async ({ page }) => {
  const before = await counter(page, 'goals_aborted');
  await page.getByText('Exercise failure paths').click();
  await page.getByRole('button', { name: 'Abort on ROS' }).click();
  await expect(page.locator('#action-result')).toHaveText(/^aborted:/);
  await expect.poll(() => counter(page, 'goals_aborted')).toBe(before + 1);
});

test('bridge action deadline also requests cancellation on ROS', async ({ page }) => {
  const before = await rosCounters(page);
  await page.getByText('Exercise failure paths').click();
  await page.getByRole('button', { name: 'Test action deadline' }).click();
  await expect(page.locator('#action-result')).toHaveText('timeout: timed out waiting for ROS future');
  await expect.poll(() => rosCounters(page)).toMatchObject({
    goals_started: before.goals_started + 1,
    goals_cancelled: before.goals_cancelled + 1,
  });
});
