import { test, expect } from '@playwright/test';

test.describe('Login Page Data Rendering', () => {
  test('login page renders form with inputs', async ({ page }) => {
    // Navigate to login page
    await page.goto('/login');
    await page.waitForLoadState('networkidle');
    
    // Verify page loads
    const loginPage = page.locator('[data-testid="page-login"]');
    await expect(loginPage).toBeVisible();
    
    // Verify logo and title
    const logo = page.locator('h1:has-text("Reverie")');
    await expect(logo).toBeVisible();
    
    // Verify form exists
    const form = page.locator('form');
    await expect(form).toBeVisible();
    
    // Verify server URL input
    const serverUrlInput = page.locator('input[placeholder="http://localhost:4533/rest"]');
    await expect(serverUrlInput).toBeVisible();
    await expect(serverUrlInput).toHaveValue('http://localhost:4533/rest');
    
    // Verify username input
    const usernameInput = page.locator('[data-testid="username-input"]');
    await expect(usernameInput).toBeVisible();
    await expect(usernameInput).toHaveAttribute('placeholder', '请输入用户名');
    
    // Verify password input
    const passwordInput = page.locator('[data-testid="password-input"]');
    await expect(passwordInput).toBeVisible();
    await expect(passwordInput).toHaveAttribute('placeholder', '请输入密码');
    
    // Verify login button
    const loginButton = page.locator('[data-testid="login-button"]');
    await expect(loginButton).toBeVisible();
    await expect(loginButton).toBeEnabled();
    
    // Verify footer text
    const footer = page.locator('p:has-text("Reverie 音乐服务器 • 基于 Rust & Dioxus 构建")');
    await expect(footer).toBeVisible();
  });
});