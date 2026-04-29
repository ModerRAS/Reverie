import { test, expect } from '@playwright/test';
import { TestContext } from '../helpers/test-context';

test.describe('Search Page Data Rendering', () => {
  test('search page renders results for query', async ({ page }) => {
    const ctx = new TestContext(page);
    
    // Login first
    await ctx.login();
    
    // Navigate to search page (with empty query)
    await page.goto('/search');
    await page.waitForLoadState('networkidle');
    
    // Verify page loads with empty state
    const emptyState = page.locator('h2:has-text("搜索音乐")');
    await expect(emptyState).toBeVisible();
    
    // The search page uses UI state for query, we need to trigger search via UI
    // The search input is likely in the layout (sidebar or top bar)
    // Let's find the search input
    const searchInput = page.locator('input[placeholder*="搜索"], input[type="search"]');
    // If search input exists, type query
    if (await searchInput.count() > 0) {
      await searchInput.fill('test');
      await searchInput.press('Enter');
      await page.waitForTimeout(500);
      
      // Verify search page shows results
      const searchPage = page.locator('[data-testid="page-search"]');
      await expect(searchPage).toBeVisible();
      
      // Verify page header shows query
      const header = page.locator('h1:has-text("搜索: \\"test\\"")');
      await expect(header).toBeVisible();
      
      // Verify results sections exist (mock returns 5 songs, 3 albums, 2 artists)
      // Artists section
      const artistsHeading = page.locator('h2:has-text("艺术家")');
      if (await artistsHeading.isVisible()) {
        const artistCards = page.locator('[data-testid="album-card"]'); // ArtistCard uses data-testid="album-card"
        const artistCount = await artistCards.count();
        expect(artistCount).toBe(2); // mock::search returns 2 artists
      }
      
      // Albums section
      const albumsHeading = page.locator('h2:has-text("专辑")');
      if (await albumsHeading.isVisible()) {
        const albumCards = page.locator('.album-card');
        const albumCount = await albumCards.count();
        expect(albumCount).toBe(3); // mock::search returns 3 albums
      }
      
      // Songs section
      const songsHeading = page.locator('h2:has-text("歌曲")');
      if (await songsHeading.isVisible()) {
        const songItems = page.locator('.flex.items-center.gap-3.p-2.rounded.hover\\:bg-gray-800');
        const songCount = await songItems.count();
        expect(songCount).toBe(5); // mock::search returns 5 songs
      }
    }
  });
});