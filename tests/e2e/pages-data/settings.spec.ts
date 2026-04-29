import { test, expect } from '@playwright/test';
import { TestContext } from '../helpers/test-context';

test.describe('Settings Page Data Rendering', () => {
  test('settings page renders settings sections', async ({ page }) => {
    const ctx = new TestContext(page);
    
    // Login first
    await ctx.login();
    
    // Navigate to settings page
    await page.goto('/settings');
    await page.waitForLoadState('networkidle');
    
    // Verify page loads
    const settingsPage = page.locator('[data-testid="page-settings"]');
    await expect(settingsPage).toBeVisible();
    
    // Verify page header
    const header = page.locator('h1:has-text("设置")');
    await expect(header).toBeVisible();
    
    // Verify general settings section
    const generalSection = page.locator('h2:has-text("通用")');
    await expect(generalSection).toBeVisible();
    
    // Verify theme selector
    const themeSelect = page.locator('select:has(option[value="dark"])');
    await expect(themeSelect).toBeVisible();
    
    // Verify language selector
    const languageSelect = page.locator('select:has(option[value="en"])');
    await expect(languageSelect).toBeVisible();
    
    // Verify playback settings section
    const playbackSection = page.locator('h2:has-text("播放")');
    await expect(playbackSection).toBeVisible();
    
    // Verify fade checkbox
    const fadeCheckbox = page.locator('input[type="checkbox"]').first();
    await expect(fadeCheckbox).toBeVisible();
    
    // Verify about section
    const aboutSection = page.locator('h2:has-text("关于")');
    await expect(aboutSection).toBeVisible();
    
    // Verify version text
    const versionText = page.locator('p:has-text("版本: 0.1.0")');
    await expect(versionText).toBeVisible();
  });
});