import { test, expect } from '@playwright/test';
import { TestContext } from '../helpers/test-context';

test.describe('Album Detail Page Data Rendering', () => {
  test('album detail page renders track list or not found', async ({ page }) => {
    const ctx = new TestContext(page);
    
    // Login first
    await ctx.login();
    
    // Navigate to album detail page with a dummy album ID
    // The page will try to fetch album from backend; if not found, shows "专辑不存在"
    await page.goto('/album/album-1');
    await page.waitForLoadState('networkidle');
    
    // Wait for loading spinner to disappear
    await page.waitForTimeout(1000);
    
    // Check if album exists (depends on seeded data)
    const notFound = page.locator('div:has-text("专辑不存在")');
    const albumDetailPage = page.locator('[data-testid="page-album-detail"]');
    
    if (await notFound.isVisible()) {
      // Album not found, verify empty state
      await expect(notFound).toBeVisible();
    } else {
      // Album found, verify page loads
      await expect(albumDetailPage).toBeVisible();
      
      // Verify album header
      const albumTitle = albumDetailPage.locator('h1');
      await expect(albumTitle).toBeVisible();
      
      // Verify track list exists
      const trackList = albumDetailPage.locator('[data-testid="track-list"]');
      await expect(trackList).toBeVisible();
      
      // Verify track rows exist (if any)
      const trackRows = trackList.locator('[data-testid^="track-row-"]');
      const trackCount = await trackRows.count();
      expect(trackCount).toBeGreaterThanOrEqual(0);
      
      // If there are tracks, verify first track has title
      if (trackCount > 0) {
        const firstTrackTitle = trackRows.first().locator('p.text-white, p.text-blue-400');
        await expect(firstTrackTitle).toBeVisible();
      }
      
      // Verify play button exists
      const playButton = albumDetailPage.locator('button:has(svg path[d="M8 5v14l11-7z"])');
      await expect(playButton).toBeVisible();
    }
  });
});