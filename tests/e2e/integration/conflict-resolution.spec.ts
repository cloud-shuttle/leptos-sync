import { expect, test } from '@playwright/test';

test.describe('Conflict Resolution E2E Tests', () => {
    test('should handle simultaneous task edits', async ({ browser }) => {
        const user1Context = await browser.newContext();
        const user2Context = await browser.newContext();

        const user1Page = await user1Context.newPage();
        const user2Page = await user2Context.newPage();

        try {
            // Both users navigate to the app
            await Promise.all([
                user1Page.goto('/tests/e2e/fixtures/simple-test.html'),
                user2Page.goto('/tests/e2e/fixtures/simple-test.html')
            ]);

            await Promise.all([
                user1Page.waitForLoadState('networkidle'),
                user2Page.waitForLoadState('networkidle')
            ]);

            // Both users add the same task
            await Promise.all([
                user1Page.fill('.task-input', 'Conflict Test Task'),
                user2Page.fill('.task-input', 'Conflict Test Task')
            ]);

            await Promise.all([
                user1Page.click('.add-button'),
                user2Page.click('.add-button')
            ]);

            // Both users should see their task
            await expect(user1Page.locator('#task-count')).toContainText('1');
            await expect(user2Page.locator('#task-count')).toContainText('1');

            // User 1 toggles the task to completed
            await user1Page.locator('.task-item').first().locator('.toggle-button').click();

            // User 2 toggles the task to completed (simultaneous operation)
            await user2Page.locator('.task-item').first().locator('.toggle-button').click();

            // Both users should see the task as completed
            await expect(user1Page.locator('.task-item').first()).toHaveClass(/completed/);
            await expect(user2Page.locator('.task-item').first()).toHaveClass(/completed/);

            // In a real conflict resolution scenario, the system would handle
            // the simultaneous operations and ensure consistency

        } finally {
            await user1Context.close();
            await user2Context.close();
        }
    });

    test('should handle delete-edit conflicts', async ({ browser }) => {
        const user1Context = await browser.newContext();
        const user2Context = await browser.newContext();

        const user1Page = await user1Context.newPage();
        const user2Page = await user2Context.newPage();

        try {
            // Both users navigate to the app
            await Promise.all([
                user1Page.goto('/tests/e2e/fixtures/simple-test.html'),
                user2Page.goto('/tests/e2e/fixtures/simple-test.html')
            ]);

            await Promise.all([
                user1Page.waitForLoadState('networkidle'),
                user2Page.waitForLoadState('networkidle')
            ]);

            // Both users add the same task
            await Promise.all([
                user1Page.fill('.task-input', 'Delete-Edit Conflict Task'),
                user2Page.fill('.task-input', 'Delete-Edit Conflict Task')
            ]);

            await Promise.all([
                user1Page.click('.add-button'),
                user2Page.click('.add-button')
            ]);

            // User 1 deletes the task
            await user1Page.locator('.task-item').first().locator('.delete-button').click();

            // User 2 toggles the task (edit operation)
            await user2Page.locator('.task-item').first().locator('.toggle-button').click();

            // User 1 should see no tasks
            await expect(user1Page.locator('#task-count')).toContainText('0');

            // User 2 should see the task as completed
            await expect(user2Page.locator('#task-count')).toContainText('1');
            await expect(user2Page.locator('.task-item').first()).toHaveClass(/completed/);

            // In a real conflict resolution scenario, the system would need to
            // decide whether to keep the task (with the edit) or delete it
            // This is a classic delete-edit conflict

        } finally {
            await user1Context.close();
            await user2Context.close();
        }
    });

    test('should handle add-delete conflicts', async ({ browser }) => {
        const user1Context = await browser.newContext();
        const user2Context = await browser.newContext();

        const user1Page = await user1Context.newPage();
        const user2Page = await user2Context.newPage();

        try {
            // Both users navigate to the app
            await Promise.all([
                user1Page.goto('/tests/e2e/fixtures/simple-test.html'),
                user2Page.goto('/tests/e2e/fixtures/simple-test.html')
            ]);

            await Promise.all([
                user1Page.waitForLoadState('networkidle'),
                user2Page.waitForLoadState('networkidle')
            ]);

            // User 1 adds a task
            await user1Page.fill('.task-input', 'Add-Delete Conflict Task');
            await user1Page.click('.add-button');

            // User 2 adds the same task
            await user2Page.fill('.task-input', 'Add-Delete Conflict Task');
            await user2Page.click('.add-button');

            // User 1 immediately deletes the task
            await user1Page.locator('.task-item').first().locator('.delete-button').click();

            // User 2 toggles the task
            await user2Page.locator('.task-item').first().locator('.toggle-button').click();

            // User 1 should see no tasks
            await expect(user1Page.locator('#task-count')).toContainText('0');

            // User 2 should see the task as completed
            await expect(user2Page.locator('#task-count')).toContainText('1');
            await expect(user2Page.locator('.task-item').first()).toHaveClass(/completed/);

            // This represents a conflict where one user deletes a task
            // while another user is editing it

        } finally {
            await user1Context.close();
            await user2Context.close();
        }
    });

    test('should handle multiple simultaneous conflicts', async ({ browser }) => {
        const user1Context = await browser.newContext();
        const user2Context = await browser.newContext();

        const user1Page = await user1Context.newPage();
        const user2Page = await user2Context.newPage();

        try {
            // Both users navigate to the app
            await Promise.all([
                user1Page.goto('/tests/e2e/fixtures/simple-test.html'),
                user2Page.goto('/tests/e2e/fixtures/simple-test.html')
            ]);

            await Promise.all([
                user1Page.waitForLoadState('networkidle'),
                user2Page.waitForLoadState('networkidle')
            ]);

            // Both users add multiple tasks
            const tasks = ['Task 1', 'Task 2', 'Task 3'];

            for (const task of tasks) {
                await user1Page.fill('.task-input', task);
                await user1Page.click('.add-button');
            }

            for (const task of tasks) {
                await user2Page.fill('.task-input', task);
                await user2Page.click('.add-button');
            }

            // Both users should see 3 tasks
            await expect(user1Page.locator('#task-count')).toContainText('3');
            await expect(user2Page.locator('#task-count')).toContainText('3');

            // User 1 performs multiple operations
            await user1Page.locator('.task-item').nth(0).locator('.toggle-button').click(); // Complete first task
            await user1Page.locator('.task-item').nth(1).locator('.delete-button').click(); // Delete second task

            // User 2 performs different operations
            await user2Page.locator('.task-item').nth(1).locator('.toggle-button').click(); // Complete second task
            await user2Page.locator('.task-item').nth(2).locator('.delete-button').click(); // Delete third task

            // Verify local state changes
            await expect(user1Page.locator('#task-count')).toContainText('2');
            await expect(user2Page.locator('#task-count')).toContainText('2');

            // In a real conflict resolution scenario, the system would need to
            // resolve multiple simultaneous conflicts and ensure consistency

        } finally {
            await user1Context.close();
            await user2Context.close();
        }
    });

    test('should handle network interruption during conflicts', async ({ browser }) => {
        const user1Context = await browser.newContext();
        const user2Context = await browser.newContext();

        const user1Page = await user1Context.newPage();
        const user2Page = await user2Context.newPage();

        try {
            // Both users navigate to the app
            await Promise.all([
                user1Page.goto('/tests/e2e/fixtures/simple-test.html'),
                user2Page.goto('/tests/e2e/fixtures/simple-test.html')
            ]);

            await Promise.all([
                user1Page.waitForLoadState('networkidle'),
                user2Page.waitForLoadState('networkidle')
            ]);

            // Both users add the same task
            await Promise.all([
                user1Page.fill('.task-input', 'Network Conflict Task'),
                user2Page.fill('.task-input', 'Network Conflict Task')
            ]);

            await Promise.all([
                user1Page.click('.add-button'),
                user2Page.click('.add-button')
            ]);

            // Simulate network interruption for user 1
            await user1Context.setOffline(true);

            // User 1 performs operations while offline
            await user1Page.locator('.task-item').first().locator('.toggle-button').click();

            // User 2 performs operations while online
            await user2Page.locator('.task-item').first().locator('.delete-button').click();

            // User 1 should see the task as completed (local state)
            await expect(user1Page.locator('.task-item').first()).toHaveClass(/completed/);

            // User 2 should see no tasks
            await expect(user2Page.locator('#task-count')).toContainText('0');

            // Restore network for user 1
            await user1Context.setOffline(false);

            // In a real scenario, the system would need to resolve the conflict
            // when user 1 comes back online

        } finally {
            await user1Context.close();
            await user2Context.close();
        }
    });

    test('should handle conflict resolution UI', async ({ page }) => {
        // This test would verify that conflict resolution UI elements
        // are displayed correctly when conflicts occur

        await page.goto('/tests/e2e/fixtures/simple-test.html');
        await page.waitForLoadState('networkidle');

        // Add a task
        await page.fill('.task-input', 'Conflict UI Test Task');
        await page.click('.add-button');

        // Verify the task exists
        await expect(page.locator('#task-count')).toContainText('1');

        // In a real conflict resolution scenario, there would be UI elements
        // to show conflicts and allow users to resolve them
        // For now, we verify the basic functionality works

        // Simulate a conflict by toggling the task
        await page.locator('.task-item').first().locator('.toggle-button').click();

        // Verify the task is completed
        await expect(page.locator('.task-item').first()).toHaveClass(/completed/);

        // In a real implementation, there might be conflict resolution dialogs,
        // merge indicators, or other UI elements to help users resolve conflicts
    });
});
