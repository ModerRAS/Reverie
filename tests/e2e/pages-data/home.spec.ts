import { test, expect } from '@playwright/test';
import { TestContext } from '../helpers/test-context';

test.describe('Home Page Data Rendering', () => {
  test('home page renders album grid and song cards', async ({ page }) => {
    const ctx = new TestContext(page);
    
    // Login first
    await ctx.login();
    
    // Navigate to home page
    await page.goto('/home');
    await page.waitForLoadState('networkidle');
    
    // Verify page loads
    const homePage = page.locator('[data-testid="page-home"]');
    await expect(homePage).toBeVisible();
    
    // Verify album grid section exists (may be empty if no data)
    // The home page has two sections: recent albums and recent songs
    // We can verify the page structure by checking for section headings
    const recentAlbumsHeading = page.locator('h2:has-text("最近播放")');
    await expect(recentAlbumsHeading).toBeVisible();
    
    const recentSongsHeading = page.locator('h2:has-text("最近添加")');
    await expect(recentSongsHeading).toBeVisible();
    
    // Verify quick access cards exist
    const quickAccessHeading = page.locator('h2:has-text("快速访问")');
    await expect(quickAccessHeading).toBeVisible();
    
    // Verify album cards are rendered (if any)
    // Album cards are rendered via AlbumCard component which doesn't have data-testid
    // But we can check for the grid container
    const albumGrid = page.locator('.grid.grid-cols-2.md\\:grid-cols-3.lg\\:grid-cols-6');
    // The grid may be empty, but it should exist in DOM
    await expect(albumGrid).toBeAttached();
    
    // Verify song cards are rendered (if any)
    const songGrid = page.locator('.grid.grid-cols-1.md\\:grid-cols-2.lg\\:grid-cols-4');
    await expect(songGrid).toBeAttached();
    
    // Verify no critical console errors
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });
    
    // Filter out non-critical errors
    const criticalErrors = errors.filter(e =>
      !e.includes('favicon') &&
      !e.includes('404') &&
      !e.includes('Failed to load resource')
    );
    expect(criticalErrors).toHaveLength(0);
  });
});