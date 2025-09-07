import { expect, test } from '@playwright/test';

test.describe('Simple E2E Tests', () => {
    test.beforeEach(async ({ page }) => {
        // Navigate to our simple test application
        await page.goto('/tests/e2e/fixtures/simple-test.html');

        // Wait for the page to load
        await page.waitForLoadState('networkidle');
    });

    test('should load the simple test application', async ({ page }) => {
        // Verify the app loaded successfully
        await expect(page.locator('h1')).toContainText('Simple E2E Test Application');

        // Verify we can see the task input
        await expect(page.locator('.task-input')).toBeVisible();

        // Verify we can see the add button
        await expect(page.locator('.add-button')).toBeVisible();
    });

    test('should create a new task', async ({ page }) => {
        const taskText = 'E2E Test Task';

        // Add a new task
        await page.fill('.task-input', taskText);
        await page.click('.add-button');

        // Verify the task was added
        await expect(page.locator('#task-count')).toContainText('1');
        await expect(page.locator('#tasks-container')).toContainText(taskText);

        // Verify the input is cleared
        await expect(page.locator('.task-input')).toHaveValue('');
    });

    test('should handle multiple tasks', async ({ page }) => {
        const tasks = ['Task 1', 'Task 2', 'Task 3'];

        // Add multiple tasks
        for (const task of tasks) {
            await page.fill('.task-input', task);
            await page.click('.add-button');
        }

        // Verify task count
        await expect(page.locator('#task-count')).toContainText('3');

        // Verify all tasks are present
        for (const task of tasks) {
            await expect(page.locator('#tasks-container')).toContainText(task);
        }
    });

    test('should toggle task completion', async ({ page }) => {
        const taskText = 'Toggle Test Task';

        // Add a task first
        await page.fill('.task-input', taskText);
        await page.click('.add-button');

        // Find the task and toggle it
        const taskItem = page.locator('.task-item').filter({ hasText: taskText });
        await expect(taskItem).toBeVisible();

        // Click the toggle button
        await taskItem.locator('.toggle-button').click();

        // Verify the task is marked as completed
        await expect(taskItem).toHaveClass(/completed/);
    });

    test('should delete a task', async ({ page }) => {
        const taskText = 'Delete Test Task';

        // Add a task first
        await page.fill('.task-input', taskText);
        await page.click('.add-button');

        // Verify the task exists
        const taskItem = page.locator('.task-item').filter({ hasText: taskText });
        await expect(taskItem).toBeVisible();

        // Delete the task
        await taskItem.locator('.delete-button').click();

        // Verify the task is removed
        await expect(taskItem).not.toBeVisible();
        await expect(page.locator('#task-count')).toContainText('0');
    });

    test('should persist tasks in local storage', async ({ page }) => {
        const taskText = 'Persistence Test Task';

        // Add a task
        await page.fill('.task-input', taskText);
        await page.click('.add-button');

        // Verify the task exists
        await expect(page.locator('#task-count')).toContainText('1');

        // Reload the page
        await page.reload();
        await page.waitForLoadState('networkidle');

        // Verify the task persists
        await expect(page.locator('#task-count')).toContainText('1');
        await expect(page.locator('#tasks-container')).toContainText(taskText);
    });

    test('should display manager info correctly', async ({ page }) => {
        // Add a task
        await page.fill('.task-input', 'Info Test Task');
        await page.click('.add-button');

        // Check manager info section
        await expect(page.locator('#total-tasks')).toContainText('1');
        await expect(page.locator('#is-empty')).toContainText('false');
        await expect(page.locator('#user-id')).toContainText('test-user-123');
    });

    test('should handle keyboard input', async ({ page }) => {
        const taskText = 'Keyboard Test Task';

        // Add a task using Enter key
        await page.fill('.task-input', taskText);
        await page.press('.task-input', 'Enter');

        // Verify the task was added
        await expect(page.locator('#task-count')).toContainText('1');
        await expect(page.locator('#tasks-container')).toContainText(taskText);
    });
});
