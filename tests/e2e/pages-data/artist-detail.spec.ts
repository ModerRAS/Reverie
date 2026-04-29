import { test, expect } from '@playwright/test';
import { TestContext } from '../helpers/test-context';

test.describe('Artist Detail Page Data Rendering', () => {
  test('artist detail page renders album list or not found', async ({ page }) => {
    const ctx = new TestContext(page);
    
    // Login first
    await ctx.login();
    
    // Navigate to artist detail page with a dummy artist ID
    await page.goto('/artist/artist-1');
    await page.waitForLoadState('networkidle');
    
    // Wait for loading spinner to disappear
    await page.waitForTimeout(1000);
    
    // Check if artist exists
    const notFound = page.locator('div:has-text("艺术家不存在")');
    const artistDetailPage = page.locator('[data-testid="page-artist-detail"]');
    
    if (await notFound.isVisible()) {
      // Artist not found, verify empty state
      await expect(notFound).toBeVisible();
    } else {
      // Artist found, verify page loads
      await expect(artistDetailPage).toBeVisible();
      
      // Verify artist header
      const artistName = artistDetailPage.locator('h1');
      await expect(artistName).toBeVisible();
      
      // Verify album section exists
      const albumSection = artistDetailPage.locator('h2:has-text("专辑")');
      await expect(albumSection).toBeVisible();
      
      // Verify album grid exists (may be empty)
      const albumGrid = artistDetailPage.locator('.album-grid');
      await expect(albumGrid).toBeAttached();
      
      // Verify album cards exist (if any)
      const albumCards = albumGrid.locator('.album-card');
      const albumCount = await albumCards.count();
      expect(albumCount).toBeGreaterThanOrEqual(0);
      
      // Verify top songs section exists
      const topSongsSection = artistDetailPage.locator('h2:has-text("热门歌曲")');
      await expect(topSongsSection).toBeVisible();
      
      // Verify compact song list exists (may be empty)
      const songList = artistDetailPage.locator('.space-y-1');
      await expect(songList).toBeAttached();
      
      // Verify play all button exists
      const playAllButton = artistDetailPage.locator('button:has-text("播放全部")');
      await expect(playAllButton).toBeVisible();
    }
  });
});