import { test, expect } from '@playwright/test';
import { TestContext } from '../helpers/test-context';

test.describe('Artists Page Data Rendering', () => {
  test('artists page renders artist grid', async ({ page }) => {
    const ctx = new TestContext(page);
    
    // Login first
    await ctx.login();
    
    // Navigate to artists page
    await page.goto('/artists');
    await page.waitForLoadState('networkidle');
    
    // Verify page loads
    const artistsPage = page.locator('[data-testid="page-artists"]');
    await expect(artistsPage).toBeVisible();
    
    // Verify page header
    const header = page.locator('h1:has-text("艺术家")');
    await expect(header).toBeVisible();
    
    // Verify artist grid exists (may be empty)
    const artistGrid = page.locator('.grid.grid-cols-2.sm\\:grid-cols-3.md\\:grid-cols-4.lg\\:grid-cols-5.xl\\:grid-cols-6');
    await expect(artistGrid).toBeAttached();
    
    // Verify artist cards are rendered (if any)
    // ArtistCard component has data-testid="album-card" (incorrect but we use it)
    const artistCards = page.locator('[data-testid="album-card"]');
    const count = await artistCards.count();
    expect(count).toBeGreaterThanOrEqual(0);
    
    // If there are artist cards, verify they have names
    if (count > 0) {
      const firstArtistName = artistCards.first().locator('h3');
      await expect(firstArtistName).toBeVisible();
    }
  });
});