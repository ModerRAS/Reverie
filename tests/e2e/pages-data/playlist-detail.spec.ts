import { test, expect } from '@playwright/test';
import { TestContext } from '../helpers/test-context';

test.describe('Playlist Detail Page Data Rendering', () => {
  test('playlist detail page renders track list', async ({ page }) => {
    const ctx = new TestContext(page);
    
    // Login first
    await ctx.login();
    
    // Navigate to playlist detail page with a dummy playlist ID
    // Mock data returns playlist with 15 songs
    await page.goto('/playlist/playlist-1');
    await page.waitForLoadState('networkidle');
    
    // Wait for loading spinner to disappear
    await page.waitForTimeout(1000);
    
    // Verify page loads
    const playlistDetailPage = page.locator('[data-testid="page-playlist-detail"]');
    await expect(playlistDetailPage).toBeVisible();
    
    // Verify playlist header
    const playlistTitle = playlistDetailPage.locator('h1');
    await expect(playlistTitle).toBeVisible();
    const titleText = await playlistTitle.textContent();
    expect(titleText).toBe('Playlist 1');
    
    // Verify track list exists
    const trackList = playlistDetailPage.locator('[data-testid="track-list"]');
    await expect(trackList).toBeVisible();
    
    // Verify track rows exist (mock returns 15 songs)
    const trackRows = trackList.locator('[data-testid^="track-row-"]');
    const trackCount = await trackRows.count();
    expect(trackCount).toBe(15); // mock::playlist_detail returns 15 songs
    
    // Verify first track has title
    const firstTrackTitle = trackRows.first().locator('p.text-white, p.text-blue-400');
    await expect(firstTrackTitle).toBeVisible();
    const firstTitleText = await firstTrackTitle.textContent();
    expect(firstTitleText).toBe('Playlist Song 1');
    
    // Verify play button exists
    const playButton = playlistDetailPage.locator('button:has(svg path[d="M8 5v14l11-7z"])');
    await expect(playButton).toBeVisible();
    
    // Verify owner info
    const ownerInfo = playlistDetailPage.locator('span:has-text("demo")');
    await expect(ownerInfo).toBeVisible();
  });
});