import { expect, test } from '@playwright/test';

test.describe('Cross-Browser Compatibility', () => {
    test.beforeEach(async ({ page }) => {
        // Navigate to the task manager demo
        await page.goto('/examples/task_manager_demo/index.html');

        // Wait for the WASM to load
        await page.waitForLoadState('networkidle');

        // Wait for the app to be ready
        await page.waitForSelector('[data-testid="task-manager"]', { timeout: 10000 });
    });

    test('should work consistently across browsers', async ({ page, browserName }) => {
        const taskText = `Cross-browser test for ${browserName}`;

        // Test basic functionality
        await page.fill('[data-testid="task-input"]', taskText);
        await page.click('[data-testid="add-task"]');

        // Verify task was added
        await expect(page.locator('[data-testid="task-list"]')).toContainText(taskText);

        // Test task interaction
        const taskItem = page.locator('[data-testid="task-item"]').filter({ hasText: taskText });
        await taskItem.locator('[data-testid="toggle-task"]').click();

        // Verify toggle worked
        await expect(taskItem).toHaveClass(/completed/);

        // Test deletion
        await taskItem.locator('[data-testid="delete-task"]').click();

        // Verify deletion worked
        await expect(taskItem).not.toBeVisible();

        console.log(`✅ ${browserName} compatibility test passed`);
    });

    test('should handle WASM loading consistently', async ({ page, browserName }) => {
        // Check that WASM loaded successfully
        const wasmLoaded = await page.evaluate(() => {
            return typeof window !== 'undefined' &&
                typeof (window as any).wasm_bindgen !== 'undefined';
        });

        expect(wasmLoaded).toBe(true);

        // Check that the app is functional
        await expect(page.locator('[data-testid="task-manager"]')).toBeVisible();

        console.log(`✅ ${browserName} WASM loading test passed`);
    });

    test('should handle local storage consistently', async ({ page, browserName }) => {
        const taskText = `Storage test for ${browserName}`;

        // Add a task
        await page.fill('[data-testid="task-input"]', taskText);
        await page.click('[data-testid="add-task"]');

        // Verify task exists
        await expect(page.locator('[data-testid="task-list"]')).toContainText(taskText);

        // Check local storage directly
        const localStorageData = await page.evaluate(() => {
            const data = [];
            for (let i = 0; i < localStorage.length; i++) {
                const key = localStorage.key(i);
                if (key && key.includes('leptos-sync')) {
                    data.push({ key, value: localStorage.getItem(key) });
                }
            }
            return data;
        });

        // Should have some local storage data
        expect(localStorageData.length).toBeGreaterThan(0);

        // Reload and verify persistence
        await page.reload();
        await page.waitForLoadState('networkidle');
        await page.waitForSelector('[data-testid="task-manager"]', { timeout: 10000 });

        // Task should persist
        await expect(page.locator('[data-testid="task-list"]')).toContainText(taskText);

        console.log(`✅ ${browserName} local storage test passed`);
    });

    test('should handle IndexedDB consistently', async ({ page, browserName }) => {
        // Check IndexedDB availability
        const indexedDBAvailable = await page.evaluate(() => {
            return typeof indexedDB !== 'undefined';
        });

        expect(indexedDBAvailable).toBe(true);

        // Test IndexedDB operations
        const indexedDBWorking = await page.evaluate(async () => {
            try {
                const request = indexedDB.open('test-db', 1);
                return new Promise((resolve) => {
                    request.onsuccess = () => {
                        request.result.close();
                        resolve(true);
                    };
                    request.onerror = () => resolve(false);
                });
            } catch (e) {
                return false;
            }
        });

        expect(indexedDBWorking).toBe(true);

        console.log(`✅ ${browserName} IndexedDB test passed`);
    });

    test('should handle WebSocket connections consistently', async ({ page, browserName }) => {
        // Check WebSocket availability
        const webSocketAvailable = await page.evaluate(() => {
            return typeof WebSocket !== 'undefined';
        });

        expect(webSocketAvailable).toBe(true);

        // Test WebSocket connection (if sync is enabled)
        const syncStatus = page.locator('[data-testid="sync-status"]');
        if (await syncStatus.isVisible()) {
            // Wait for sync to be ready
            await expect(syncStatus).toContainText(/connected|ready/i, { timeout: 10000 });
        }

        console.log(`✅ ${browserName} WebSocket test passed`);
    });

    test('should handle mobile viewport consistently', async ({ page, browserName }) => {
        // Test mobile viewport if this is a mobile test
        if (browserName.includes('Mobile') || browserName.includes('Safari')) {
            // Check that the UI is responsive
            const taskManager = page.locator('[data-testid="task-manager"]');
            await expect(taskManager).toBeVisible();

            // Test touch interactions
            await page.fill('[data-testid="task-input"]', 'Mobile test task');
            await page.click('[data-testid="add-task"]');

            // Verify task was added
            await expect(page.locator('[data-testid="task-list"]')).toContainText('Mobile test task');

            // Test touch interactions on task items
            const taskItem = page.locator('[data-testid="task-item"]').first();
            await taskItem.locator('[data-testid="toggle-task"]').click();

            console.log(`✅ ${browserName} mobile viewport test passed`);
        } else {
            console.log(`⏭️ Skipping mobile test for ${browserName}`);
        }
    });
});
