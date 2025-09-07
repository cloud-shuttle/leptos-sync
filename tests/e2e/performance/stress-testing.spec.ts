import { expect, test } from '@playwright/test';

test.describe('Stress Testing E2E Tests', () => {
    test('should handle massive task creation', async ({ page }) => {
        await page.goto('/tests/e2e/fixtures/simple-test.html');
        await page.waitForLoadState('networkidle');

        const startTime = Date.now();
        const taskCount = 1000; // Stress test with 1000 tasks

        // Create many tasks rapidly
        for (let i = 0; i < taskCount; i++) {
            await page.fill('.task-input', `Stress Test Task ${i}`);
            await page.click('.add-button');

            // Log progress every 100 tasks
            if (i % 100 === 0) {
                console.log(`Created ${i} tasks...`);
            }
        }

        const endTime = Date.now();
        const duration = endTime - startTime;

        // Verify all tasks were created
        await expect(page.locator('#task-count')).toContainText(taskCount.toString());

        // Performance assertion: should complete within reasonable time
        expect(duration).toBeLessThan(60000); // 60 seconds max

        // Log performance metrics
        console.log(`Created ${taskCount} tasks in ${duration}ms (${duration / taskCount}ms per task)`);

        // Verify the page is still responsive
        await expect(page.locator('.task-input')).toBeVisible();
        await expect(page.locator('.add-button')).toBeVisible();
    });

    test('should handle rapid task operations', async ({ page }) => {
        await page.goto('/tests/e2e/fixtures/simple-test.html');
        await page.waitForLoadState('networkidle');

        // Create initial tasks
        const initialTasks = 100;
        for (let i = 0; i < initialTasks; i++) {
            await page.fill('.task-input', `Rapid Ops Task ${i}`);
            await page.click('.add-button');
        }

        await expect(page.locator('#task-count')).toContainText(initialTasks.toString());

        const startTime = Date.now();

        // Perform rapid operations
        const operations = 500;
        for (let i = 0; i < operations; i++) {
            const taskItems = page.locator('.task-item');
            const taskCount = await taskItems.count();

            if (taskCount > 0) {
                const randomIndex = Math.floor(Math.random() * taskCount);
                const randomTask = taskItems.nth(randomIndex);

                // Randomly choose an operation
                const operation = Math.random();
                if (operation < 0.5) {
                    // Toggle task
                    await randomTask.locator('.toggle-button').click();
                } else {
                    // Delete task
                    await randomTask.locator('.delete-button').click();
                }
            }

            // Add a new task occasionally
            if (i % 10 === 0) {
                await page.fill('.task-input', `Rapid Ops New Task ${i}`);
                await page.click('.add-button');
            }

            // Log progress every 100 operations
            if (i % 100 === 0) {
                console.log(`Performed ${i} operations...`);
            }
        }

        const endTime = Date.now();
        const duration = endTime - startTime;

        // Verify the application is still responsive
        await expect(page.locator('.task-input')).toBeVisible();
        await expect(page.locator('.add-button')).toBeVisible();

        // Performance assertion
        expect(duration).toBeLessThan(120000); // 2 minutes max

        console.log(`Performed ${operations} operations in ${duration}ms`);
    });

    test('should handle memory pressure', async ({ page }) => {
        await page.goto('/tests/e2e/fixtures/simple-test.html');
        await page.waitForLoadState('networkidle');

        // Get initial memory usage
        const initialMemory = await page.evaluate(() => {
            return (performance as any).memory ? (performance as any).memory.usedJSHeapSize : null;
        });

        // Create many tasks to test memory usage
        const taskCount = 500;
        for (let i = 0; i < taskCount; i++) {
            await page.fill('.task-input', `Memory Test Task ${i}`);
            await page.click('.add-button');
        }

        // Get memory usage after creating tasks
        const afterCreateMemory = await page.evaluate(() => {
            return (performance as any).memory ? (performance as any).memory.usedJSHeapSize : null;
        });

        // Perform many operations to test memory management
        for (let i = 0; i < 200; i++) {
            const taskItems = page.locator('.task-item');
            const count = await taskItems.count();

            if (count > 0) {
                // Delete a random task
                const randomIndex = Math.floor(Math.random() * count);
                await taskItems.nth(randomIndex).locator('.delete-button').click();
            }

            // Add a new task
            await page.fill('.task-input', `Memory Ops Task ${i}`);
            await page.click('.add-button');
        }

        // Get memory usage after operations
        const afterOpsMemory = await page.evaluate(() => {
            return (performance as any).memory ? (performance as any).memory.usedJSHeapSize : null;
        });

        // Verify the application is still responsive
        await expect(page.locator('.task-input')).toBeVisible();
        await expect(page.locator('.add-button')).toBeVisible();

        // Log memory usage if available
        if (initialMemory && afterCreateMemory && afterOpsMemory) {
            const memoryIncrease = afterCreateMemory - initialMemory;
            const memoryAfterOps = afterOpsMemory - initialMemory;

            console.log(`Memory usage - Initial: ${initialMemory}, After create: ${afterCreateMemory}, After ops: ${afterOpsMemory}`);
            console.log(`Memory increase: ${memoryIncrease} bytes, After operations: ${memoryAfterOps} bytes`);

            // Memory should not grow excessively
            expect(memoryIncrease).toBeLessThan(100 * 1024 * 1024); // 100MB max increase
        }
    });

    test('should handle concurrent user stress', async ({ browser }) => {
        // Create multiple browser contexts to simulate many users
        const contexts = [];
        const pages = [];

        try {
            // Create 5 concurrent users
            for (let i = 0; i < 5; i++) {
                const context = await browser.newContext();
                const page = await context.newPage();
                contexts.push(context);
                pages.push(page);
            }

            // All users navigate to the app
            await Promise.all(pages.map(page => page.goto('/tests/e2e/fixtures/simple-test.html')));
            await Promise.all(pages.map(page => page.waitForLoadState('networkidle')));

            const startTime = Date.now();

            // All users perform operations concurrently
            const operations = 50;
            for (let i = 0; i < operations; i++) {
                const promises = pages.map(async (page, userIndex) => {
                    // Each user performs different operations
                    if (i % 3 === 0) {
                        // Add task
                        await page.fill('.task-input', `User ${userIndex} Task ${i}`);
                        await page.click('.add-button');
                    } else if (i % 3 === 1) {
                        // Toggle task if available
                        const taskItems = page.locator('.task-item');
                        const count = await taskItems.count();
                        if (count > 0) {
                            await taskItems.first().locator('.toggle-button').click();
                        }
                    } else {
                        // Delete task if available
                        const taskItems = page.locator('.task-item');
                        const count = await taskItems.count();
                        if (count > 0) {
                            await taskItems.first().locator('.delete-button').click();
                        }
                    }
                });

                await Promise.all(promises);

                // Log progress every 10 operations
                if (i % 10 === 0) {
                    console.log(`Completed ${i} concurrent operations...`);
                }
            }

            const endTime = Date.now();
            const duration = endTime - startTime;

            // Verify all users' applications are still responsive
            for (let i = 0; i < pages.length; i++) {
                await expect(pages[i].locator('.task-input')).toBeVisible();
                await expect(pages[i].locator('.add-button')).toBeVisible();
            }

            // Performance assertion
            expect(duration).toBeLessThan(180000); // 3 minutes max

            console.log(`Completed ${operations} concurrent operations across 5 users in ${duration}ms`);

        } finally {
            // Clean up contexts
            await Promise.all(contexts.map(context => context.close()));
        }
    });

    test('should handle network stress', async ({ page, context }) => {
        await page.goto('/tests/e2e/fixtures/simple-test.html');
        await page.waitForLoadState('networkidle');

        // Create some initial tasks
        for (let i = 0; i < 50; i++) {
            await page.fill('.task-input', `Network Stress Task ${i}`);
            await page.click('.add-button');
        }

        await expect(page.locator('#task-count')).toContainText('50');

        const startTime = Date.now();

        // Simulate network stress by rapidly going offline/online
        for (let i = 0; i < 20; i++) {
            // Go offline
            await context.setOffline(true);

            // Perform operations while offline
            await page.fill('.task-input', `Offline Task ${i}`);
            await page.click('.add-button');

            // Go back online
            await context.setOffline(false);

            // Perform operations while online
            await page.fill('.task-input', `Online Task ${i}`);
            await page.click('.add-button');

            // Log progress every 5 cycles
            if (i % 5 === 0) {
                console.log(`Completed ${i} network stress cycles...`);
            }
        }

        const endTime = Date.now();
        const duration = endTime - startTime;

        // Verify the application is still responsive
        await expect(page.locator('.task-input')).toBeVisible();
        await expect(page.locator('.add-button')).toBeVisible();

        // Performance assertion
        expect(duration).toBeLessThan(120000); // 2 minutes max

        console.log(`Completed network stress test in ${duration}ms`);
    });

    test('should handle data corruption recovery', async ({ page }) => {
        await page.goto('/tests/e2e/fixtures/simple-test.html');
        await page.waitForLoadState('networkidle');

        // Create some tasks
        for (let i = 0; i < 100; i++) {
            await page.fill('.task-input', `Corruption Test Task ${i}`);
            await page.click('.add-button');
        }

        await expect(page.locator('#task-count')).toContainText('100');

        // Simulate data corruption by manipulating localStorage
        await page.evaluate(() => {
            // Corrupt the localStorage data
            localStorage.setItem('e2e-test-tasks', 'corrupted-data');
        });

        // Reload the page to test recovery
        await page.reload();
        await page.waitForLoadState('networkidle');

        // Verify the application recovers gracefully
        await expect(page.locator('h1')).toContainText('Simple E2E Test Application');
        await expect(page.locator('.task-input')).toBeVisible();
        await expect(page.locator('.add-button')).toBeVisible();

        // Verify that the application can still function
        await page.fill('.task-input', 'Recovery Test Task');
        await page.click('.add-button');

        await expect(page.locator('#task-count')).toContainText('1');
        await expect(page.locator('#tasks-container')).toContainText('Recovery Test Task');
    });

    test('should handle browser resource limits', async ({ page }) => {
        await page.goto('/tests/e2e/fixtures/simple-test.html');
        await page.waitForLoadState('networkidle');

        // Test with limited resources by creating many tasks
        const taskCount = 2000;
        const startTime = Date.now();

        for (let i = 0; i < taskCount; i++) {
            await page.fill('.task-input', `Resource Limit Task ${i}`);
            await page.click('.add-button');

            // Log progress every 200 tasks
            if (i % 200 === 0) {
                console.log(`Created ${i} tasks...`);
            }
        }

        const endTime = Date.now();
        const duration = endTime - startTime;

        // Verify all tasks were created
        await expect(page.locator('#task-count')).toContainText(taskCount.toString());

        // Test that the application is still responsive
        await expect(page.locator('.task-input')).toBeVisible();
        await expect(page.locator('.add-button')).toBeVisible();

        // Test scrolling performance with many tasks
        await page.locator('.task-item').last().scrollIntoViewIfNeeded();

        // Test interaction performance
        const firstTask = page.locator('.task-item').first();
        await firstTask.locator('.toggle-button').click();

        // Performance assertion
        expect(duration).toBeLessThan(300000); // 5 minutes max

        console.log(`Created ${taskCount} tasks in ${duration}ms (${duration / taskCount}ms per task)`);
    });
});
