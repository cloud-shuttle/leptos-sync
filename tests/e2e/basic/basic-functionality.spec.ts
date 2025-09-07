import { expect, test } from '@playwright/test';

test.describe('Basic Functionality Tests', () => {
    test.beforeEach(async ({ page }) => {
        // Navigate to the task manager demo
        await page.goto('/examples/task_manager_demo/dist/index.html');

        // Wait for the WASM to load
        await page.waitForLoadState('networkidle');

        // Wait for the app to be ready - look for the main heading
        await page.waitForSelector('h1:has-text("Collaborative Task Manager Demo")', { timeout: 10000 });
    });

    test('should load the task manager application', async ({ page }) => {
        // Verify the app loaded successfully
        await expect(page.locator('h1')).toContainText('Collaborative Task Manager Demo');

        // Verify we can see the task input
        await expect(page.locator('input[placeholder="Task title..."]')).toBeVisible();

        // Verify we can see the add button
        await expect(page.locator('button:has-text("Add Task")')).toBeVisible();
    });

    test('should create a new task', async ({ page }) => {
        const taskText = 'E2E Test Task';

        // Add a new task
        await page.fill('input[placeholder="Task title..."]', taskText);
        await page.click('button:has-text("Add Task")');

        // Verify the task was added by checking the task count
        await expect(page.locator('h3:has-text("Tasks")')).toContainText('Tasks (1)');

        // Verify the input is cleared
        await expect(page.locator('input[placeholder="Task title..."]')).toHaveValue('');
    });

    test('should handle multiple tasks', async ({ page }) => {
        const tasks = ['Task 1', 'Task 2', 'Task 3'];

        // Add multiple tasks
        for (const task of tasks) {
            await page.fill('input[placeholder="Task title..."]', task);
            await page.click('button:has-text("Add Task")');
        }

        // Verify task count
        await expect(page.locator('h3:has-text("Tasks")')).toContainText('Tasks (3)');

        // Verify all tasks are present in the task list
        for (const task of tasks) {
            await expect(page.locator('.tasks-list')).toContainText(task);
        }
    });

    test('should filter tasks by priority', async ({ page }) => {
        // Add tasks with different priorities
        await page.fill('input[placeholder="Task title..."]', 'High Priority Task');
        await page.selectOption('select', 'High');
        await page.click('button:has-text("Add Task")');

        await page.fill('input[placeholder="Task title..."]', 'Low Priority Task');
        await page.selectOption('select', 'Low');
        await page.click('button:has-text("Add Task")');

        // Filter by high priority
        await page.selectOption('.filters select:first-of-type', 'High Priority');

        // Verify only high priority task is shown
        await expect(page.locator('.tasks-list')).toContainText('High Priority Task');
        await expect(page.locator('.tasks-list')).not.toContainText('Low Priority Task');
    });

    test('should merge dummy tasks', async ({ page }) => {
        // Add one task first
        await page.fill('input[placeholder="Task title..."]', 'Original Task');
        await page.click('button:has-text("Add Task")');

        // Verify initial count
        await expect(page.locator('h3:has-text("Tasks")')).toContainText('Tasks (1)');

        // Merge dummy tasks
        await page.click('button:has-text("Merge Dummy Tasks")');

        // Verify count increased
        await expect(page.locator('h3:has-text("Tasks")')).toContainText('Tasks (3)');

        // Verify dummy tasks are present
        await expect(page.locator('.tasks-list')).toContainText('Dummy Task 1');
        await expect(page.locator('.tasks-list')).toContainText('Dummy Task 2');
    });

    test('should persist tasks in local storage', async ({ page }) => {
        const taskText = 'Persistence Test Task';

        // Add a task
        await page.fill('input[placeholder="Task title..."]', taskText);
        await page.click('button:has-text("Add Task")');

        // Verify the task exists
        await expect(page.locator('h3:has-text("Tasks")')).toContainText('Tasks (1)');

        // Reload the page
        await page.reload();
        await page.waitForLoadState('networkidle');
        await page.waitForSelector('h1:has-text("Collaborative Task Manager Demo")', { timeout: 10000 });

        // Verify the task persists
        await expect(page.locator('h3:has-text("Tasks")')).toContainText('Tasks (1)');
        await expect(page.locator('.tasks-list')).toContainText(taskText);
    });

    test('should display manager info correctly', async ({ page }) => {
        // Add a task
        await page.fill('input[placeholder="Task title..."]', 'Info Test Task');
        await page.click('button:has-text("Add Task")');

        // Check manager info section
        await expect(page.locator('.manager-info')).toContainText('Total Tasks: 1');
        await expect(page.locator('.manager-info')).toContainText('Is Empty: false');
        await expect(page.locator('.manager-info')).toContainText('User ID:');
    });
});
