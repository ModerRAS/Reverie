import { test, expect } from '@playwright/test';

// NOTE: This smoke test requires data-testid attributes to be added to Dioxus components.
// If data-testid attributes don't exist yet, this test will use CSS selectors as fallback.

test.describe('Smoke Tests', () => {
  test('home page loads without errors', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });

    await page.goto('/home');
    await page.waitForLoadState('networkidle');

    // If data-testid exists, use it; otherwise just check page loaded
    const homePage = page.locator('[data-testid="page-home"]');
    if (await homePage.count() > 0) {
      await expect(homePage).toBeVisible();
    }

    // Filter out non-critical errors (favicon, etc.)
    const criticalErrors = errors.filter(e =>
      !e.includes('favicon') &&
      !e.includes('404')
    );
    expect(criticalErrors).toHaveLength(0);
  });

  test('login page has form fields', async ({ page }) => {
    await page.goto('/login');

    // Try data-testid first, fall back to CSS selectors
    const usernameInput = page.locator('[data-testid="username-input"]');
    const passwordInput = page.locator('[data-testid="password-input"]');
    const loginButton = page.locator('[data-testid="login-button"]');

    // If data-testid not found, use CSS fallback
    if (await usernameInput.count() === 0) {
      await expect(page.locator('input[type="text"], input[name="username"]')).toBeVisible();
    } else {
      await expect(usernameInput).toBeVisible();
      await expect(passwordInput).toBeVisible();
      await expect(loginButton).toBeVisible();
    }
  });

  test('server ping endpoint responds', async ({ page }) => {
    const response = await page.request.get('/rest/ping?f=json');
    expect(response.ok()).toBeTruthy();

    const body = await response.json();
    expect(body['subsonic-response']).toBeDefined();
    expect(body['subsonic-response'].status).toBe('ok');
  });
});
