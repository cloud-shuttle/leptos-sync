import { expect, test } from '@playwright/test';

test.describe('Data Migration E2E Tests', () => {
    test('should handle localStorage data migration', async ({ page }) => {
        await page.goto('/tests/e2e/fixtures/simple-test.html');
        await page.waitForLoadState('networkidle');

        // Create some tasks with the current format
        const tasks = ['Migration Task 1', 'Migration Task 2', 'Migration Task 3'];
        for (const task of tasks) {
            await page.fill('.task-input', task);
            await page.click('.add-button');
        }

        await expect(page.locator('#task-count')).toContainText('3');

        // Simulate data migration by updating localStorage format
        await page.evaluate(() => {
            const currentData = localStorage.getItem('e2e-test-tasks');
            if (currentData) {
                const tasks = JSON.parse(currentData);

                // Simulate migration to a new format
                const migratedTasks = tasks.map((task: any) => ({
                    ...task,
                    version: '2.0',
                    migratedAt: new Date().toISOString(),
                    // Add new fields that might be added in a future version
                    tags: [],
                    priority: 'medium'
                }));

                localStorage.setItem('e2e-test-tasks', JSON.stringify(migratedTasks));
            }
        });

        // Reload the page to test migration
        await page.reload();
        await page.waitForLoadState('networkidle');

        // Verify that the migrated data is still accessible
        await expect(page.locator('#task-count')).toContainText('3');
        for (const task of tasks) {
            await expect(page.locator('#tasks-container')).toContainText(task);
        }

        // Verify that new functionality works with migrated data
        await page.fill('.task-input', 'Post-Migration Task');
        await page.click('.add-button');

        await expect(page.locator('#task-count')).toContainText('4');
        await expect(page.locator('#tasks-container')).toContainText('Post-Migration Task');
    });

    test('should handle schema version changes', async ({ page }) => {
        await page.goto('/tests/e2e/fixtures/simple-test.html');
        await page.waitForLoadState('networkidle');

        // Create tasks with old schema
        await page.evaluate(() => {
            const oldSchemaTasks = [
                {
                    id: 1,
                    title: 'Old Schema Task 1',
                    description: 'Task with old schema',
                    completed: false,
                    createdAt: '2023-01-01T00:00:00.000Z'
                    // Missing new fields that might be added
                },
                {
                    id: 2,
                    title: 'Old Schema Task 2',
                    description: 'Another old schema task',
                    completed: true,
                    createdAt: '2023-01-02T00:00:00.000Z'
                }
            ];

            localStorage.setItem('e2e-test-tasks', JSON.stringify(oldSchemaTasks));
        });

        // Reload the page to test schema migration
        await page.reload();
        await page.waitForLoadState('networkidle');

        // Verify that old schema data is still accessible
        await expect(page.locator('#task-count')).toContainText('2');
        await expect(page.locator('#tasks-container')).toContainText('Old Schema Task 1');
        await expect(page.locator('#tasks-container')).toContainText('Old Schema Task 2');

        // Verify that the completed task is marked as completed
        const completedTask = page.locator('.task-item').filter({ hasText: 'Old Schema Task 2' });
        await expect(completedTask).toHaveClass(/completed/);

        // Verify that new tasks can be added
        await page.fill('.task-input', 'New Schema Task');
        await page.click('.add-button');

        await expect(page.locator('#task-count')).toContainText('3');
        await expect(page.locator('#tasks-container')).toContainText('New Schema Task');
    });

    test('should handle data format changes', async ({ page }) => {
        await page.goto('/tests/e2e/fixtures/simple-test.html');
        await page.waitForLoadState('networkidle');

        // Simulate a change in data format (e.g., from array to object)
        await page.evaluate(() => {
            const oldFormatData = [
                {
                    id: 1,
                    title: 'Format Change Task 1',
                    description: 'Task with old format',
                    completed: false,
                    createdAt: '2023-01-01T00:00:00.000Z'
                },
                {
                    id: 2,
                    title: 'Format Change Task 2',
                    description: 'Another old format task',
                    completed: false,
                    createdAt: '2023-01-02T00:00:00.000Z'
                }
            ];

            // Simulate migration from array format to object format
            const newFormatData = {
                version: '2.0',
                tasks: oldFormatData,
                metadata: {
                    createdAt: new Date().toISOString(),
                    migratedFrom: '1.0'
                }
            };

            localStorage.setItem('e2e-test-tasks', JSON.stringify(newFormatData));
        });

        // Reload the page to test format migration
        await page.reload();
        await page.waitForLoadState('networkidle');

        // Verify that the new format data is accessible
        await expect(page.locator('#task-count')).toContainText('2');
        await expect(page.locator('#tasks-container')).toContainText('Format Change Task 1');
        await expect(page.locator('#tasks-container')).toContainText('Format Change Task 2');

        // Verify that new tasks can be added with the new format
        await page.fill('.task-input', 'New Format Task');
        await page.click('.add-button');

        await expect(page.locator('#task-count')).toContainText('3');
        await expect(page.locator('#tasks-container')).toContainText('New Format Task');
    });

    test('should handle corrupted data recovery', async ({ page }) => {
        await page.goto('/tests/e2e/fixtures/simple-test.html');
        await page.waitForLoadState('networkidle');

        // Create some valid tasks first
        await page.fill('.task-input', 'Valid Task 1');
        await page.click('.add-button');
        await page.fill('.task-input', 'Valid Task 2');
        await page.click('.add-button');

        await expect(page.locator('#task-count')).toContainText('2');

        // Simulate data corruption
        await page.evaluate(() => {
            // Corrupt the data in various ways
            localStorage.setItem('e2e-test-tasks', 'invalid-json');
            localStorage.setItem('e2e-test-tasks-backup', '{"tasks":[],"version":"1.0"}');
        });

        // Reload the page to test corruption recovery
        await page.reload();
        await page.waitForLoadState('networkidle');

        // Verify that the application recovers gracefully
        await expect(page.locator('h1')).toContainText('Simple E2E Test Application');
        await expect(page.locator('.task-input')).toBeVisible();
        await expect(page.locator('.add-button')).toBeVisible();

        // Verify that the application can still function
        await page.fill('.task-input', 'Recovery Test Task');
        await page.click('.add-button');

        await expect(page.locator('#task-count')).toContainText('1');
        await expect(page.locator('#tasks-container')).toContainText('Recovery Test Task');
    });

    test('should handle partial data migration', async ({ page }) => {
        await page.goto('/tests/e2e/fixtures/simple-test.html');
        await page.waitForLoadState('networkidle');

        // Simulate partial migration scenario
        await page.evaluate(() => {
            const partialData = [
                {
                    id: 1,
                    title: 'Partially Migrated Task 1',
                    description: 'Task with partial migration',
                    completed: false,
                    createdAt: '2023-01-01T00:00:00.000Z',
                    version: '2.0' // This task was migrated
                },
                {
                    id: 2,
                    title: 'Partially Migrated Task 2',
                    description: 'Task without migration',
                    completed: false,
                    createdAt: '2023-01-02T00:00:00.000Z'
                    // This task was not migrated yet
                }
            ];

            localStorage.setItem('e2e-test-tasks', JSON.stringify(partialData));
        });

        // Reload the page to test partial migration handling
        await page.reload();
        await page.waitForLoadState('networkidle');

        // Verify that both migrated and non-migrated tasks are accessible
        await expect(page.locator('#task-count')).toContainText('2');
        await expect(page.locator('#tasks-container')).toContainText('Partially Migrated Task 1');
        await expect(page.locator('#tasks-container')).toContainText('Partially Migrated Task 2');

        // Verify that new tasks can be added
        await page.fill('.task-input', 'New Task After Partial Migration');
        await page.click('.add-button');

        await expect(page.locator('#task-count')).toContainText('3');
        await expect(page.locator('#tasks-container')).toContainText('New Task After Partial Migration');
    });

    test('should handle migration rollback', async ({ page }) => {
        await page.goto('/tests/e2e/fixtures/simple-test.html');
        await page.waitForLoadState('networkidle');

        // Create some tasks
        await page.fill('.task-input', 'Rollback Test Task 1');
        await page.click('.add-button');
        await page.fill('.task-input', 'Rollback Test Task 2');
        await page.click('.add-button');

        await expect(page.locator('#task-count')).toContainText('2');

        // Simulate migration with rollback capability
        await page.evaluate(() => {
            const currentData = localStorage.getItem('e2e-test-tasks');
            if (currentData) {
                const tasks = JSON.parse(currentData);

                // Create backup before migration
                localStorage.setItem('e2e-test-tasks-backup', currentData);

                // Simulate migration
                const migratedTasks = tasks.map((task: any) => ({
                    ...task,
                    version: '2.0',
                    migratedAt: new Date().toISOString()
                }));

                localStorage.setItem('e2e-test-tasks', JSON.stringify(migratedTasks));
            }
        });

        // Reload the page to test migration
        await page.reload();
        await page.waitForLoadState('networkidle');

        // Verify that the migration worked
        await expect(page.locator('#task-count')).toContainText('2');
        await expect(page.locator('#tasks-container')).toContainText('Rollback Test Task 1');
        await expect(page.locator('#tasks-container')).toContainText('Rollback Test Task 2');

        // Simulate rollback scenario
        await page.evaluate(() => {
            const backupData = localStorage.getItem('e2e-test-tasks-backup');
            if (backupData) {
                localStorage.setItem('e2e-test-tasks', backupData);
                localStorage.removeItem('e2e-test-tasks-backup');
            }
        });

        // Reload the page to test rollback
        await page.reload();
        await page.waitForLoadState('networkidle');

        // Verify that the rollback worked
        await expect(page.locator('#task-count')).toContainText('2');
        await expect(page.locator('#tasks-container')).toContainText('Rollback Test Task 1');
        await expect(page.locator('#tasks-container')).toContainText('Rollback Test Task 2');

        // Verify that the application still functions after rollback
        await page.fill('.task-input', 'Post-Rollback Task');
        await page.click('.add-button');

        await expect(page.locator('#task-count')).toContainText('3');
        await expect(page.locator('#tasks-container')).toContainText('Post-Rollback Task');
    });

    test('should handle migration with data validation', async ({ page }) => {
        await page.goto('/tests/e2e/fixtures/simple-test.html');
        await page.waitForLoadState('networkidle');

        // Simulate migration with data validation
        await page.evaluate(() => {
            const mixedData = [
                {
                    id: 1,
                    title: 'Valid Task',
                    description: 'This task is valid',
                    completed: false,
                    createdAt: '2023-01-01T00:00:00.000Z'
                },
                {
                    id: 2,
                    title: '', // Invalid: empty title
                    description: 'This task has an empty title',
                    completed: false,
                    createdAt: '2023-01-02T00:00:00.000Z'
                },
                {
                    id: 3,
                    title: 'Valid Task 2',
                    description: 'This task is also valid',
                    completed: true,
                    createdAt: '2023-01-03T00:00:00.000Z'
                }
            ];

            localStorage.setItem('e2e-test-tasks', JSON.stringify(mixedData));
        });

        // Reload the page to test migration with validation
        await page.reload();
        await page.waitForLoadState('networkidle');

        // Verify that valid tasks are accessible
        await expect(page.locator('#task-count')).toContainText('2'); // Only valid tasks
        await expect(page.locator('#tasks-container')).toContainText('Valid Task');
        await expect(page.locator('#tasks-container')).toContainText('Valid Task 2');

        // Verify that the invalid task was filtered out
        await expect(page.locator('#tasks-container')).not.toContainText('This task has an empty title');

        // Verify that the completed task is marked as completed
        const completedTask = page.locator('.task-item').filter({ hasText: 'Valid Task 2' });
        await expect(completedTask).toHaveClass(/completed/);

        // Verify that new tasks can be added
        await page.fill('.task-input', 'New Valid Task');
        await page.click('.add-button');

        await expect(page.locator('#task-count')).toContainText('3');
        await expect(page.locator('#tasks-container')).toContainText('New Valid Task');
    });
});
