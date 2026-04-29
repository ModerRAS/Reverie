import { test, expect } from '@playwright/test';

// Helper function to check for console errors
async function checkForConsoleErrors(page: any): Promise<string[]> {
  const errors: string[] = [];
  page.on('console', msg => {
    if (msg.type() === 'error') {
      errors.push(msg.text());
    }
  });
  return errors;
}

// Helper function to filter out non-critical errors
function filterCriticalErrors(errors: string[]): string[] {
  return errors.filter(e =>
    !e.includes('favicon') &&
    !e.includes('404') &&
    !e.includes('Failed to load resource') &&
    !e.includes('net::ERR')
  );
}

// Helper function to login before accessing protected pages
async function loginBeforeTest(page: any): Promise<void> {
  await page.goto('/login');
  await page.waitForLoadState('networkidle');
  
  // Fill login form
  const usernameInput = page.locator('[data-testid="username-input"]');
  const passwordInput = page.locator('[data-testid="password-input"]');
  const loginButton = page.locator('[data-testid="login-button"]');
  
  // Wait for login form to be visible
  await expect(usernameInput).toBeVisible({ timeout: 10000 });
  await expect(passwordInput).toBeVisible();
  await expect(loginButton).toBeVisible();
  
  // Fill credentials (using default test credentials)
  await usernameInput.fill('admin');
  await passwordInput.fill('admin');
  await loginButton.click();
  
  // Wait for navigation after login
  await page.waitForLoadState('networkidle');
}

test.describe('Page Smoke Tests', () => {
  // Public pages (no login required)
  
  test('Login page loads', async ({ page }) => {
    const errors = await checkForConsoleErrors(page);
    
    await page.goto('/login');
    await page.waitForLoadState('networkidle');
    
    // Check for login page data-testid
    const loginPage = page.locator('[data-testid="page-login"]');
    await expect(loginPage).toBeVisible({ timeout: 10000 });
    
    // Check for console errors
    const criticalErrors = filterCriticalErrors(errors);
    expect(criticalErrors).toHaveLength(0);
  });
  
  // Protected pages (login required)
  
  test('Home page loads', async ({ page }) => {
    const errors = await checkForConsoleErrors(page);
    
    // Login first
    await loginBeforeTest(page);
    
    // Navigate to home page
    await page.goto('/home');
    await page.waitForLoadState('networkidle');
    
    // Check for home page data-testid
    const homePage = page.locator('[data-testid="page-home"]');
    await expect(homePage).toBeVisible({ timeout: 10000 });
    
    // Check for console errors
    const criticalErrors = filterCriticalErrors(errors);
    expect(criticalErrors).toHaveLength(0);
  });
  
  test('Albums page loads', async ({ page }) => {
    const errors = await checkForConsoleErrors(page);
    
    // Login first
    await loginBeforeTest(page);
    
    // Navigate to albums page
    await page.goto('/albums');
    await page.waitForLoadState('networkidle');
    
    // Check for albums page data-testid
    const albumsPage = page.locator('[data-testid="page-albums"]');
    await expect(albumsPage).toBeVisible({ timeout: 10000 });
    
    // Check for console errors
    const criticalErrors = filterCriticalErrors(errors);
    expect(criticalErrors).toHaveLength(0);
  });
  
  test('Artists page loads', async ({ page }) => {
    const errors = await checkForConsoleErrors(page);
    
    // Login first
    await loginBeforeTest(page);
    
    // Navigate to artists page
    await page.goto('/artists');
    await page.waitForLoadState('networkidle');
    
    // Check for artists page data-testid
    const artistsPage = page.locator('[data-testid="page-artists"]');
    await expect(artistsPage).toBeVisible({ timeout: 10000 });
    
    // Check for console errors
    const criticalErrors = filterCriticalErrors(errors);
    expect(criticalErrors).toHaveLength(0);
  });
  
  test('Songs page loads', async ({ page }) => {
    const errors = await checkForConsoleErrors(page);
    
    // Login first
    await loginBeforeTest(page);
    
    // Navigate to songs page
    await page.goto('/songs');
    await page.waitForLoadState('networkidle');
    
    // Check for songs page data-testid
    const songsPage = page.locator('[data-testid="page-songs"]');
    await expect(songsPage).toBeVisible({ timeout: 10000 });
    
    // Check for console errors
    const criticalErrors = filterCriticalErrors(errors);
    expect(criticalErrors).toHaveLength(0);
  });
  
  test('Playlists page loads', async ({ page }) => {
    const errors = await checkForConsoleErrors(page);
    
    // Login first
    await loginBeforeTest(page);
    
    // Navigate to playlists page
    await page.goto('/playlists');
    await page.waitForLoadState('networkidle');
    
    // Check for playlists page data-testid
    const playlistsPage = page.locator('[data-testid="page-playlists"]');
    await expect(playlistsPage).toBeVisible({ timeout: 10000 });
    
    // Check for console errors
    const criticalErrors = filterCriticalErrors(errors);
    expect(criticalErrors).toHaveLength(0);
  });
  
  test('Favorites page loads', async ({ page }) => {
    const errors = await checkForConsoleErrors(page);
    
    // Login first
    await loginBeforeTest(page);
    
    // Navigate to favorites page
    await page.goto('/favorites');
    await page.waitForLoadState('networkidle');
    
    // Check for favorites page data-testid
    const favoritesPage = page.locator('[data-testid="page-favorites"]');
    await expect(favoritesPage).toBeVisible({ timeout: 10000 });
    
    // Check for console errors
    const criticalErrors = filterCriticalErrors(errors);
    expect(criticalErrors).toHaveLength(0);
  });
  
  test('Search page loads', async ({ page }) => {
    const errors = await checkForConsoleErrors(page);
    
    // Login first
    await loginBeforeTest(page);
    
    // Navigate to search page
    await page.goto('/search');
    await page.waitForLoadState('networkidle');
    
    // Check for search page data-testid
    const searchPage = page.locator('[data-testid="page-search"]');
    await expect(searchPage).toBeVisible({ timeout: 10000 });
    
    // Check for console errors
    const criticalErrors = filterCriticalErrors(errors);
    expect(criticalErrors).toHaveLength(0);
  });
  
  test('Settings page loads', async ({ page }) => {
    const errors = await checkForConsoleErrors(page);
    
    // Login first
    await loginBeforeTest(page);
    
    // Navigate to settings page
    await page.goto('/settings');
    await page.waitForLoadState('networkidle');
    
    // Check for settings page data-testid
    const settingsPage = page.locator('[data-testid="page-settings"]');
    await expect(settingsPage).toBeVisible({ timeout: 10000 });
    
    // Check for console errors
    const criticalErrors = filterCriticalErrors(errors);
    expect(criticalErrors).toHaveLength(0);
  });
  
  // Detail pages (require navigation from list pages)
  
  test('Album detail page loads', async ({ page }) => {
    const errors = await checkForConsoleErrors(page);
    
    // Login first
    await loginBeforeTest(page);
    
    // Navigate to albums page first
    await page.goto('/albums');
    await page.waitForLoadState('networkidle');
    
    // Click on first album to navigate to detail page
    const albumLink = page.locator('[data-testid="album-link"]').first();
    if (await albumLink.count() > 0) {
      await albumLink.click();
      await page.waitForLoadState('networkidle');
      
      // Check for album detail page data-testid
      const albumDetailPage = page.locator('[data-testid="page-album-detail"]');
      await expect(albumDetailPage).toBeVisible({ timeout: 10000 });
    } else {
      // If no albums exist, navigate directly to a sample album detail page
      await page.goto('/albums/1');
      await page.waitForLoadState('networkidle');
      
      // Check for album detail page data-testid
      const albumDetailPage = page.locator('[data-testid="page-album-detail"]');
      await expect(albumDetailPage).toBeVisible({ timeout: 10000 });
    }
    
    // Check for console errors
    const criticalErrors = filterCriticalErrors(errors);
    expect(criticalErrors).toHaveLength(0);
  });
  
  test('Artist detail page loads', async ({ page }) => {
    const errors = await checkForConsoleErrors(page);
    
    // Login first
    await loginBeforeTest(page);
    
    // Navigate to artists page first
    await page.goto('/artists');
    await page.waitForLoadState('networkidle');
    
    // Click on first artist to navigate to detail page
    const artistLink = page.locator('[data-testid="artist-link"]').first();
    if (await artistLink.count() > 0) {
      await artistLink.click();
      await page.waitForLoadState('networkidle');
      
      // Check for artist detail page data-testid
      const artistDetailPage = page.locator('[data-testid="page-artist-detail"]');
      await expect(artistDetailPage).toBeVisible({ timeout: 10000 });
    } else {
      // If no artists exist, navigate directly to a sample artist detail page
      await page.goto('/artists/1');
      await page.waitForLoadState('networkidle');
      
      // Check for artist detail page data-testid
      const artistDetailPage = page.locator('[data-testid="page-artist-detail"]');
      await expect(artistDetailPage).toBeVisible({ timeout: 10000 });
    }
    
    // Check for console errors
    const criticalErrors = filterCriticalErrors(errors);
    expect(criticalErrors).toHaveLength(0);
  });
  
  test('Playlist detail page loads', async ({ page }) => {
    const errors = await checkForConsoleErrors(page);
    
    // Login first
    await loginBeforeTest(page);
    
    // Navigate to playlists page first
    await page.goto('/playlists');
    await page.waitForLoadState('networkidle');
    
    // Click on first playlist to navigate to detail page
    const playlistLink = page.locator('[data-testid="playlist-link"]').first();
    if (await playlistLink.count() > 0) {
      await playlistLink.click();
      await page.waitForLoadState('networkidle');
      
      // Check for playlist detail page data-testid
      const playlistDetailPage = page.locator('[data-testid="page-playlist-detail"]');
      await expect(playlistDetailPage).toBeVisible({ timeout: 10000 });
    } else {
      // If no playlists exist, navigate directly to a sample playlist detail page
      await page.goto('/playlists/1');
      await page.waitForLoadState('networkidle');
      
      // Check for playlist detail page data-testid
      const playlistDetailPage = page.locator('[data-testid="page-playlist-detail"]');
      await expect(playlistDetailPage).toBeVisible({ timeout: 10000 });
    }
    
    // Check for console errors
    const criticalErrors = filterCriticalErrors(errors);
    expect(criticalErrors).toHaveLength(0);
  });
});