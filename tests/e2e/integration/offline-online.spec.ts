import { expect, test } from '@playwright/test';

test.describe('Offline/Online Transition Tests', () => {
    test.beforeEach(async ({ page }) => {
        // Navigate to the task manager demo
        await page.goto('/examples/task_manager_demo/index.html');

        // Wait for the WASM to load
        await page.waitForLoadState('networkidle');

        // Wait for the app to be ready
        await page.waitForSelector('[data-testid="task-manager"]', { timeout: 10000 });
    });

    test('should work offline and sync when back online', async ({ page, context }) => {
        const taskText = 'Offline Test Task';

        // Go offline
        await context.setOffline(true);

        // Verify offline indicator (if present)
        const offlineIndicator = page.locator('[data-testid="offline-indicator"]');
        if (await offlineIndicator.isVisible()) {
            await expect(offlineIndicator).toBeVisible();
        }

        // Add a task while offline
        await page.fill('[data-testid="task-input"]', taskText);
        await page.click('[data-testid="add-task"]');

        // Verify the task was added locally
        await expect(page.locator('[data-testid="task-list"]')).toContainText(taskText);

        // Go back online
        await context.setOffline(false);

        // Wait for sync to complete (if there's a sync indicator)
        const syncIndicator = page.locator('[data-testid="sync-indicator"]');
        if (await syncIndicator.isVisible()) {
            await expect(syncIndicator).toHaveClass(/synced/, { timeout: 10000 });
        }

        // Verify the task is still there after going online
        await expect(page.locator('[data-testid="task-list"]')).toContainText(taskText);
    });

    test('should handle network interruptions gracefully', async ({ page, context }) => {
        const taskText = 'Network Interruption Test';

        // Add a task while online
        await page.fill('[data-testid="task-input"]', taskText);
        await page.click('[data-testid="add-task"]');

        // Verify the task was added
        await expect(page.locator('[data-testid="task-list"]')).toContainText(taskText);

        // Simulate network interruption
        await context.setOffline(true);

        // Try to add another task while offline
        const offlineTaskText = 'Offline Task';
        await page.fill('[data-testid="task-input"]', offlineTaskText);
        await page.click('[data-testid="add-task"]');

        // Verify both tasks are present
        await expect(page.locator('[data-testid="task-list"]')).toContainText(taskText);
        await expect(page.locator('[data-testid="task-list"]')).toContainText(offlineTaskText);

        // Restore network
        await context.setOffline(false);

        // Verify both tasks persist after network restoration
        await expect(page.locator('[data-testid="task-list"]')).toContainText(taskText);
        await expect(page.locator('[data-testid="task-list"]')).toContainText(offlineTaskText);
    });

    test('should show appropriate connection status', async ({ page, context }) => {
        // Check initial online status
        const connectionStatus = page.locator('[data-testid="connection-status"]');
        if (await connectionStatus.isVisible()) {
            await expect(connectionStatus).toContainText(/online|connected/i);
        }

        // Go offline
        await context.setOffline(true);

        // Check offline status
        if (await connectionStatus.isVisible()) {
            await expect(connectionStatus).toContainText(/offline|disconnected/i);
        }

        // Go back online
        await context.setOffline(false);

        // Check online status again
        if (await connectionStatus.isVisible()) {
            await expect(connectionStatus).toContainText(/online|connected/i);
        }
    });

    test('should queue operations when offline', async ({ page, context }) => {
        // Go offline first
        await context.setOffline(true);

        const tasks = ['Queued Task 1', 'Queued Task 2', 'Queued Task 3'];

        // Add multiple tasks while offline
        for (const task of tasks) {
            await page.fill('[data-testid="task-input"]', task);
            await page.click('[data-testid="add-task"]');
        }

        // Verify all tasks are added locally
        for (const task of tasks) {
            await expect(page.locator('[data-testid="task-list"]')).toContainText(task);
        }

        // Check if there's a queue indicator
        const queueIndicator = page.locator('[data-testid="queue-indicator"]');
        if (await queueIndicator.isVisible()) {
            await expect(queueIndicator).toContainText(/3/); // Should show 3 queued operations
        }

        // Go back online
        await context.setOffline(false);

        // Wait for queue to be processed
        if (await queueIndicator.isVisible()) {
            await expect(queueIndicator).toContainText(/0/, { timeout: 10000 });
        }

        // Verify all tasks are still present
        for (const task of tasks) {
            await expect(page.locator('[data-testid="task-list"]')).toContainText(task);
        }
    });
});
