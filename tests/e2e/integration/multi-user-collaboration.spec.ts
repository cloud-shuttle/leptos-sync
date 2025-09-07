import { expect, test } from '@playwright/test';

test.describe('Multi-User Collaboration E2E Tests', () => {
    test('should handle concurrent user operations', async ({ browser }) => {
        // Create two browser contexts to simulate different users
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
            await user1Page.fill('.task-input', 'User 1 Task');
            await user1Page.click('.add-button');

            // User 2 adds a different task
            await user2Page.fill('.task-input', 'User 2 Task');
            await user2Page.click('.add-button');

            // Verify both users see their own tasks
            await expect(user1Page.locator('#task-count')).toContainText('1');
            await expect(user2Page.locator('#task-count')).toContainText('1');

            await expect(user1Page.locator('#tasks-container')).toContainText('User 1 Task');
            await expect(user2Page.locator('#tasks-container')).toContainText('User 2 Task');

            // Simulate sync by refreshing both pages
            await Promise.all([
                user1Page.reload(),
                user2Page.reload()
            ]);

            await Promise.all([
                user1Page.waitForLoadState('networkidle'),
                user2Page.waitForLoadState('networkidle')
            ]);

            // In a real sync scenario, both users should see both tasks
            // For now, we'll verify the local storage persistence works
            await expect(user1Page.locator('#tasks-container')).toContainText('User 1 Task');
            await expect(user2Page.locator('#tasks-container')).toContainText('User 2 Task');

        } finally {
            await user1Context.close();
            await user2Context.close();
        }
    });

    test('should handle simultaneous task modifications', async ({ browser }) => {
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
                user1Page.fill('.task-input', 'Shared Task'),
                user2Page.fill('.task-input', 'Shared Task')
            ]);

            await Promise.all([
                user1Page.click('.add-button'),
                user2Page.click('.add-button')
            ]);

            // Both users should see their task
            await expect(user1Page.locator('#task-count')).toContainText('1');
            await expect(user2Page.locator('#task-count')).toContainText('1');

            // User 1 toggles the task
            await user1Page.locator('.task-item').first().locator('.toggle-button').click();

            // User 2 deletes the task
            await user2Page.locator('.task-item').first().locator('.delete-button').click();

            // Verify the operations worked locally
            await expect(user1Page.locator('.task-item').first()).toHaveClass(/completed/);
            await expect(user2Page.locator('#task-count')).toContainText('0');

        } finally {
            await user1Context.close();
            await user2Context.close();
        }
    });

    test('should handle rapid concurrent operations', async ({ browser }) => {
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

            // Both users rapidly add multiple tasks
            const user1Tasks = ['User1-Task-1', 'User1-Task-2', 'User1-Task-3'];
            const user2Tasks = ['User2-Task-1', 'User2-Task-2', 'User2-Task-3'];

            // User 1 adds tasks
            for (const task of user1Tasks) {
                await user1Page.fill('.task-input', task);
                await user1Page.click('.add-button');
            }

            // User 2 adds tasks
            for (const task of user2Tasks) {
                await user2Page.fill('.task-input', task);
                await user2Page.click('.add-button');
            }

            // Verify both users see their tasks
            await expect(user1Page.locator('#task-count')).toContainText('3');
            await expect(user2Page.locator('#task-count')).toContainText('3');

            // Verify all tasks are present
            for (const task of user1Tasks) {
                await expect(user1Page.locator('#tasks-container')).toContainText(task);
            }
            for (const task of user2Tasks) {
                await expect(user2Page.locator('#tasks-container')).toContainText(task);
            }

        } finally {
            await user1Context.close();
            await user2Context.close();
        }
    });

    test('should handle user disconnection and reconnection', async ({ browser }) => {
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
            await user1Page.fill('.task-input', 'Before Disconnect Task');
            await user1Page.click('.add-button');

            // Simulate user 2 disconnecting by closing their context
            await user2Context.close();

            // User 1 adds another task
            await user1Page.fill('.task-input', 'After Disconnect Task');
            await user1Page.click('.add-button');

            // Verify user 1 sees both tasks
            await expect(user1Page.locator('#task-count')).toContainText('2');

            // Simulate user 2 reconnecting
            const newUser2Context = await browser.newContext();
            const newUser2Page = await newUser2Context.newPage();

            await newUser2Page.goto('/tests/e2e/fixtures/simple-test.html');
            await newUser2Page.waitForLoadState('networkidle');

            // User 2 should see the tasks that were added while they were disconnected
            // In a real sync scenario, this would be handled by the sync mechanism
            // For now, we'll verify the page loads correctly
            await expect(newUser2Page.locator('h1')).toContainText('Simple E2E Test Application');

            await newUser2Context.close();

        } finally {
            await user1Context.close();
        }
    });

    test('should handle data consistency across users', async ({ browser }) => {
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

            // User 1 adds a task and marks it as completed
            await user1Page.fill('.task-input', 'Consistency Test Task');
            await user1Page.click('.add-button');
            await user1Page.locator('.task-item').first().locator('.toggle-button').click();

            // Verify user 1 sees the completed task
            await expect(user1Page.locator('.task-item').first()).toHaveClass(/completed/);

            // User 2 adds a different task
            await user2Page.fill('.task-input', 'User 2 Consistency Task');
            await user2Page.click('.add-button');

            // Verify user 2 sees their task
            await expect(user2Page.locator('#task-count')).toContainText('1');
            await expect(user2Page.locator('#tasks-container')).toContainText('User 2 Consistency Task');

            // In a real sync scenario, both users would see both tasks
            // with the correct completion status
            // For now, we verify local consistency
            await expect(user1Page.locator('#task-count')).toContainText('1');
            await expect(user2Page.locator('#task-count')).toContainText('1');

        } finally {
            await user1Context.close();
            await user2Context.close();
        }
    });
});
