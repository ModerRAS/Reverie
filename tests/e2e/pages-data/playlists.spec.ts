import { test, expect } from '@playwright/test';
import { TestContext } from '../helpers/test-context';

test.describe('Playlists Page Data Rendering', () => {
  test('playlists page renders playlist cards with titles', async ({ page }) => {
    const ctx = new TestContext(page);
    
    // Login first
    await ctx.login();
    
    // Navigate to playlists page
    await page.goto('/playlists');
    await page.waitForLoadState('networkidle');
    
    // Verify page loads
    const playlistsPage = page.locator('[data-testid="page-playlists"]');
    await expect(playlistsPage).toBeVisible();
    
    // Verify page header
    const header = page.locator('h1:has-text("歌单")');
    await expect(header).toBeVisible();
    
    // Verify playlist grid exists
    const playlistGrid = page.locator('.album-grid');
    await expect(playlistGrid).toBeAttached();
    
    // Verify playlist cards are rendered (mock data returns 8 playlists)
    const playlistCards = page.locator('[data-testid="playlist-card"]');
    const count = await playlistCards.count();
    expect(count).toBe(8); // mock::playlists(8)
    
    // Verify each playlist card has a title
    for (let i = 0; i < count; i++) {
      const card = playlistCards.nth(i);
      const title = card.locator('h3');
      await expect(title).toBeVisible();
      const titleText = await title.textContent();
      expect(titleText).toMatch(/^Playlist \d+$/);
    }
    
    // Verify create playlist button exists
    const createButton = page.locator('button:has-text("新建歌单")');
    await expect(createButton).toBeVisible();
  });
});