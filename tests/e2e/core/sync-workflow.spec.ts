import { expect, test } from '@playwright/test';

test.describe('Leptos-Sync Core Workflow', () => {
    test.beforeEach(async ({ page }) => {
        // Navigate to the task manager demo
        await page.goto('/examples/task_manager_demo/index.html');

        // Wait for the WASM to load
        await page.waitForLoadState('networkidle');

        // Wait for the app to be ready
        await page.waitForSelector('[data-testid="task-manager"]', { timeout: 10000 });
    });

    test('should load the task manager application', async ({ page }) => {
        // Verify the app loaded successfully
        await expect(page.locator('[data-testid="task-manager"]')).toBeVisible();

        // Verify we can see the task input
        await expect(page.locator('[data-testid="task-input"]')).toBeVisible();

        // Verify we can see the add button
        await expect(page.locator('[data-testid="add-task"]')).toBeVisible();
    });

    test('should create a new task', async ({ page }) => {
        const taskText = 'E2E Test Task';

        // Add a new task
        await page.fill('[data-testid="task-input"]', taskText);
        await page.click('[data-testid="add-task"]');

        // Verify the task was added
        await expect(page.locator('[data-testid="task-list"]')).toContainText(taskText);

        // Verify the input is cleared
        await expect(page.locator('[data-testid="task-input"]')).toHaveValue('');
    });

    test('should toggle task completion', async ({ page }) => {
        const taskText = 'Toggle Test Task';

        // Add a task first
        await page.fill('[data-testid="task-input"]', taskText);
        await page.click('[data-testid="add-task"]');

        // Find the task and toggle it
        const taskItem = page.locator('[data-testid="task-item"]').filter({ hasText: taskText });
        await expect(taskItem).toBeVisible();

        // Click the toggle button
        await taskItem.locator('[data-testid="toggle-task"]').click();

        // Verify the task is marked as completed
        await expect(taskItem).toHaveClass(/completed/);
    });

    test('should delete a task', async ({ page }) => {
        const taskText = 'Delete Test Task';

        // Add a task first
        await page.fill('[data-testid="task-input"]', taskText);
        await page.click('[data-testid="add-task"]');

        // Verify the task exists
        const taskItem = page.locator('[data-testid="task-item"]').filter({ hasText: taskText });
        await expect(taskItem).toBeVisible();

        // Delete the task
        await taskItem.locator('[data-testid="delete-task"]').click();

        // Verify the task is removed
        await expect(taskItem).not.toBeVisible();
    });

    test('should persist tasks in local storage', async ({ page }) => {
        const taskText = 'Persistence Test Task';

        // Add a task
        await page.fill('[data-testid="task-input"]', taskText);
        await page.click('[data-testid="add-task"]');

        // Verify the task exists
        await expect(page.locator('[data-testid="task-list"]')).toContainText(taskText);

        // Reload the page
        await page.reload();
        await page.waitForLoadState('networkidle');
        await page.waitForSelector('[data-testid="task-manager"]', { timeout: 10000 });

        // Verify the task persists
        await expect(page.locator('[data-testid="task-list"]')).toContainText(taskText);
    });

    test('should handle multiple tasks', async ({ page }) => {
        const tasks = ['Task 1', 'Task 2', 'Task 3'];

        // Add multiple tasks
        for (const task of tasks) {
            await page.fill('[data-testid="task-input"]', task);
            await page.click('[data-testid="add-task"]');
        }

        // Verify all tasks are present
        for (const task of tasks) {
            await expect(page.locator('[data-testid="task-list"]')).toContainText(task);
        }

        // Verify task count
        const taskItems = page.locator('[data-testid="task-item"]');
        await expect(taskItems).toHaveCount(3);
    });
});
