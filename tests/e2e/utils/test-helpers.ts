import { Page, expect } from '@playwright/test';

export class TestHelpers {
  /**
   * Wait for the app to be fully loaded
   */
  static async waitForAppLoad(page: Page) {
    await page.waitForSelector('h1:has-text("Leptos-Sync Examples")');
    // For file:// URLs, we don't need to wait for network idle
    await page.waitForTimeout(100);
  }

  /**
   * Simulate offline mode
   */
  static async simulateOffline(page: Page) {
    await page.route('**/*', route => route.abort());
  }

  /**
   * Restore online mode
   */
  static async restoreOnline(page: Page) {
    await page.unroute('**/*');
  }

  /**
   * Measure page load performance
   */
  static async measureLoadTime(page: Page): Promise<number> {
    const startTime = Date.now();
    await page.waitForLoadState('networkidle');
    return Date.now() - startTime;
  }

  /**
   * Check accessibility basics
   */
  static async checkAccessibility(page: Page) {
    // Check heading structure
    const headings = page.locator('h1, h2, h3, h4, h5, h6');
    await expect(headings).toBeVisible();
    
    // Check focus management
    await page.keyboard.press('Tab');
    const focusedElement = page.locator(':focus');
    await expect(focusedElement).toBeVisible();
  }

  /**
   * Test responsive design
   */
  static async testResponsive(page: Page, width: number, height: number) {
    await page.setViewportSize({ width, height });
    await expect(page.locator('h1')).toBeVisible();
  }
}
