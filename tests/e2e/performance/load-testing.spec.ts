import { expect, test } from '@playwright/test';

test.describe('Performance and Load Testing', () => {
    test.beforeEach(async ({ page }) => {
        // Navigate to the task manager demo
        await page.goto('/examples/task_manager_demo/index.html');

        // Wait for the WASM to load
        await page.waitForLoadState('networkidle');

        // Wait for the app to be ready
        await page.waitForSelector('[data-testid="task-manager"]', { timeout: 10000 });
    });

    test('should handle rapid task creation', async ({ page }) => {
        const startTime = Date.now();
        const taskCount = 50;

        // Create many tasks rapidly
        for (let i = 0; i < taskCount; i++) {
            await page.fill('[data-testid="task-input"]', `Performance Task ${i}`);
            await page.click('[data-testid="add-task"]');
        }

        const endTime = Date.now();
        const duration = endTime - startTime;

        // Verify all tasks were created
        const taskItems = page.locator('[data-testid="task-item"]');
        await expect(taskItems).toHaveCount(taskCount);

        // Performance assertion: should complete within reasonable time
        expect(duration).toBeLessThan(10000); // 10 seconds max

        // Log performance metrics
        console.log(`Created ${taskCount} tasks in ${duration}ms (${duration / taskCount}ms per task)`);
    });

    test('should maintain performance with large task lists', async ({ page }) => {
        const taskCount = 100;

        // Create a large number of tasks
        for (let i = 0; i < taskCount; i++) {
            await page.fill('[data-testid="task-input"]', `Large List Task ${i}`);
            await page.click('[data-testid="add-task"]');
        }

        // Verify all tasks are present
        const taskItems = page.locator('[data-testid="task-item"]');
        await expect(taskItems).toHaveCount(taskCount);

        // Test scrolling performance
        const taskList = page.locator('[data-testid="task-list"]');
        await taskList.scrollIntoViewIfNeeded();

        // Test interaction performance with large list
        const firstTask = taskItems.first();
        const startTime = Date.now();

        await firstTask.locator('[data-testid="toggle-task"]').click();

        const endTime = Date.now();
        const interactionTime = endTime - startTime;

        // Interaction should be fast even with large lists
        expect(interactionTime).toBeLessThan(1000); // 1 second max

        console.log(`Task interaction time with ${taskCount} tasks: ${interactionTime}ms`);
    });

    test('should handle memory efficiently', async ({ page }) => {
        // Get initial memory usage (if available)
        const initialMemory = await page.evaluate(() => {
            return (performance as any).memory ? (performance as any).memory.usedJSHeapSize : null;
        });

        const taskCount = 200;

        // Create many tasks
        for (let i = 0; i < taskCount; i++) {
            await page.fill('[data-testid="task-input"]', `Memory Test Task ${i}`);
            await page.click('[data-testid="add-task"]');
        }

        // Get memory usage after creating tasks
        const afterCreateMemory = await page.evaluate(() => {
            return (performance as any).memory ? (performance as any).memory.usedJSHeapSize : null;
        });

        // Delete half the tasks
        const taskItems = page.locator('[data-testid="task-item"]');
        const halfCount = Math.floor(taskCount / 2);

        for (let i = 0; i < halfCount; i++) {
            await taskItems.first().locator('[data-testid="delete-task"]').click();
        }

        // Get memory usage after deletion
        const afterDeleteMemory = await page.evaluate(() => {
            return (performance as any).memory ? (performance as any).memory.usedJSHeapSize : null;
        });

        // Verify task count is correct
        await expect(taskItems).toHaveCount(halfCount);

        // Log memory usage if available
        if (initialMemory && afterCreateMemory && afterDeleteMemory) {
            const memoryIncrease = afterCreateMemory - initialMemory;
            const memoryAfterDelete = afterDeleteMemory - initialMemory;

            console.log(`Memory usage - Initial: ${initialMemory}, After create: ${afterCreateMemory}, After delete: ${afterDeleteMemory}`);
            console.log(`Memory increase: ${memoryIncrease} bytes, After deletion: ${memoryAfterDelete} bytes`);

            // Memory should not grow excessively
            expect(memoryIncrease).toBeLessThan(50 * 1024 * 1024); // 50MB max increase
        }
    });

    test('should load quickly', async ({ page }) => {
        const startTime = Date.now();

        // Navigate to the page
        await page.goto('/examples/task_manager_demo/index.html');

        // Wait for the app to be ready
        await page.waitForSelector('[data-testid="task-manager"]', { timeout: 10000 });

        const endTime = Date.now();
        const loadTime = endTime - startTime;

        // App should load within reasonable time
        expect(loadTime).toBeLessThan(5000); // 5 seconds max

        console.log(`App load time: ${loadTime}ms`);
    });

    test('should handle concurrent operations', async ({ page }) => {
        const operations = [];

        // Start multiple operations concurrently
        for (let i = 0; i < 10; i++) {
            operations.push(
                page.fill('[data-testid="task-input"]', `Concurrent Task ${i}`).then(() =>
                    page.click('[data-testid="add-task"]')
                )
            );
        }

        const startTime = Date.now();

        // Wait for all operations to complete
        await Promise.all(operations);

        const endTime = Date.now();
        const duration = endTime - startTime;

        // Verify all tasks were created
        const taskItems = page.locator('[data-testid="task-item"]');
        await expect(taskItems).toHaveCount(10);

        // Concurrent operations should be faster than sequential
        expect(duration).toBeLessThan(5000); // 5 seconds max

        console.log(`Concurrent operations completed in ${duration}ms`);
    });
});
