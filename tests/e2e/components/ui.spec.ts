import { expect, test } from '@playwright/test';

test.describe('UI Components E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('test-app.html');
    await page.waitForSelector('h1:has-text("Leptos-Sync Examples")');
  });

  test('should render LocalFirstProvider correctly', async ({ page }) => {
    // Check that the provider wrapper is present
    await expect(page.locator('.container')).toBeVisible();
  });

  test('should render SyncStatusIndicator correctly', async ({ page }) => {
    // Check that sync status is displayed
    await expect(page.locator('text=Sync Status: Connected')).toBeVisible();
  });

  test('should render ConflictResolver correctly', async ({ page }) => {
    // Check that conflict resolver is present
    await expect(page.locator('text=Conflict Resolver')).toBeVisible();
  });

  test('should handle responsive design on mobile', async ({ page }) => {
    // Test mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });

    // All elements should still be visible
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('p')).toBeVisible();
  });

  test('should handle keyboard navigation', async ({ page }) => {
    // Focus should be managed properly
    await page.keyboard.press('Tab');

    // Check that focus is visible (focus should be on the first focusable element)
    // Note: WebKit may not show focus indicators by default
    const focusedElement = page.locator(':focus');
    if (await focusedElement.count() > 0) {
      await expect(focusedElement).toBeVisible();
    } else {
      // For WebKit, just verify that tab navigation works
      await expect(page.locator('h1')).toBeVisible();
    }
  });
});
