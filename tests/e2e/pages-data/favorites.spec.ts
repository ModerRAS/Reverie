import { test, expect } from '@playwright/test';
import { TestContext } from '../helpers/test-context';

test.describe('Favorites Page Data Rendering', () => {
  test('favorites page renders tabs and content', async ({ page }) => {
    const ctx = new TestContext(page);
    
    // Login first
    await ctx.login();
    
    // Navigate to favorites page
    await page.goto('/favorites');
    await page.waitForLoadState('networkidle');
    
    // Verify page loads
    const favoritesPage = page.locator('[data-testid="page-favorites"]');
    await expect(favoritesPage).toBeVisible();
    
    // Verify page header
    const header = page.locator('h1:has-text("收藏")');
    await expect(header).toBeVisible();
    
    // Verify tabs exist
    const songsTab = page.locator('button:has-text("歌曲")');
    const albumsTab = page.locator('button:has-text("专辑")');
    const artistsTab = page.locator('button:has-text("艺术家")');
    await expect(songsTab).toBeVisible();
    await expect(albumsTab).toBeVisible();
    await expect(artistsTab).toBeVisible();
    
    // Default tab is songs (index 0)
    // Verify track list exists (mock data returns 10 songs)
    const trackList = page.locator('[data-testid="track-list"]');
    await expect(trackList).toBeVisible();
    
    // Verify track rows
    const trackRows = page.locator('[data-testid^="track-row-"]');
    const songCount = await trackRows.count();
    expect(songCount).toBe(10); // mock::favorites() returns songs(10)
    
    // Click albums tab
    await albumsTab.click();
    await page.waitForTimeout(100);
    
    // Verify album grid appears
    const albumGrid = page.locator('.album-grid');
    await expect(albumGrid).toBeVisible();
    
    // Verify album cards (mock returns 6 albums)
    const albumCards = page.locator('.album-card');
    const albumCount = await albumCards.count();
    expect(albumCount).toBe(6); // mock::favorites() returns albums(6)
    
    // Click artists tab
    await artistsTab.click();
    await page.waitForTimeout(100);
    
    // Verify artist grid appears
    const artistGrid = page.locator('.grid.grid-cols-2.sm\\:grid-cols-3.md\\:grid-cols-4.lg\\:grid-cols-5.xl\\:grid-cols-6');
    await expect(artistGrid).toBeVisible();
    
    // Verify artist cards (mock returns 4 artists)
    const artistCards = page.locator('[data-testid="album-card"]'); // ArtistCard uses data-testid="album-card"
    const artistCount = await artistCards.count();
    expect(artistCount).toBe(4); // mock::favorites() returns artists(4)
  });
});