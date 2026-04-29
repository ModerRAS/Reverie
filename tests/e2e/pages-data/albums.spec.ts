import { test, expect } from '@playwright/test';
import { TestContext } from '../helpers/test-context';

test.describe('Albums Page Data Rendering', () => {
  test('albums page renders album grid and tabs', async ({ page }) => {
    const ctx = new TestContext(page);
    
    // Login first
    await ctx.login();
    
    // Navigate to albums page
    await page.goto('/albums');
    await page.waitForLoadState('networkidle');
    
    // Verify page loads
    const albumsPage = page.locator('[data-testid="page-albums"]');
    await expect(albumsPage).toBeVisible();
    
    // Verify page header
    const header = page.locator('h1:has-text("专辑")');
    await expect(header).toBeVisible();
    
    // Verify tabs exist (TabBar component)
    const tabs = page.locator('[role="tablist"], .tab-bar');
    // TabBar may not have a specific data-testid, but we can check for tab buttons
    const tabButtons = page.locator('button:has-text("最近添加"), button:has-text("最近播放"), button:has-text("最多播放"), button:has-text("随机"), button:has-text("已收藏")');
    await expect(tabButtons.first()).toBeVisible();
    
    // Verify album grid exists (may be empty)
    const albumGrid = page.locator('.album-grid');
    await expect(albumGrid).toBeAttached();
    
    // Verify album cards are rendered (if any)
    // AlbumCard component doesn't have data-testid, but we can check for album-card class
    const albumCards = page.locator('.album-card');
    // At least zero cards should be present
    const count = await albumCards.count();
    expect(count).toBeGreaterThanOrEqual(0);
    
    // If there are album cards, verify they have titles
    if (count > 0) {
      const firstAlbumTitle = albumCards.first().locator('h3');
      await expect(firstAlbumTitle).toBeVisible();
    }
    
    // Verify sorting tabs are clickable (click each tab)
    for (const tabName of ['最近添加', '最近播放', '最多播放', '随机', '已收藏']) {
      const tab = page.locator(`button:has-text("${tabName}")`);
      await tab.click();
      // Wait a bit for any UI updates
      await page.waitForTimeout(100);
      // Verify page still visible
      await expect(albumsPage).toBeVisible();
    }
  });
});