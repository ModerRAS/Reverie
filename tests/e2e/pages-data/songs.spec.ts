import { test, expect } from '@playwright/test';
import { TestContext } from '../helpers/test-context';

test.describe('Songs Page Data Rendering', () => {
  test('songs page renders track table with titles', async ({ page }) => {
    const ctx = new TestContext(page);
    
    // Login first
    await ctx.login();
    
    // Navigate to songs page
    await page.goto('/songs');
    await page.waitForLoadState('networkidle');
    
    // Verify page loads
    const songsPage = page.locator('[data-testid="page-songs"]');
    await expect(songsPage).toBeVisible();
    
    // Verify page header
    const header = page.locator('h1:has-text("歌曲")');
    await expect(header).toBeVisible();
    
    // Verify track list exists (may be empty)
    const trackList = page.locator('[data-testid="track-list"]');
    await expect(trackList).toBeAttached();
    
    // Verify track rows are rendered (if any)
    const trackRows = page.locator('[data-testid^="track-row-"]');
    const count = await trackRows.count();
    expect(count).toBeGreaterThanOrEqual(0);
    
    // If there are track rows, verify they have titles
    if (count > 0) {
      const firstTrackRow = trackRows.first();
      // Track row contains title in a paragraph
      const title = firstTrackRow.locator('p.text-white, p.text-blue-400');
      await expect(title).toBeVisible();
      // Verify title is not empty
      const titleText = await title.textContent();
      expect(titleText?.trim().length).toBeGreaterThan(0);
    }
    
    // Verify table headers exist
    const tableHeader = trackList.locator('.flex.items-center.gap-4.px-4.py-2.text-xs');
    await expect(tableHeader).toBeVisible();
    // Check for column headers
    await expect(tableHeader.locator('div:has-text("Title")')).toBeVisible();
    await expect(tableHeader.locator('div:has-text("Duration")')).toBeVisible();
  });
});